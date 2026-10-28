use crate::error::AppError;
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use uuid::Uuid;

const COOKIE_NAME: &str = "rustdb_session";
type LoginAttempts = Arc<Mutex<HashMap<[u8; 32], (u32, Instant)>>>;

#[derive(Clone)]
pub struct Auth {
    pool: PgPool,
    secure_cookie: bool,
    bootstrap_token_hash: [u8; 32],
    login_attempts: LoginAttempts,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Admin,
    Editor,
    Viewer,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Editor => "editor",
            Self::Viewer => "viewer",
        }
    }
    pub fn parse(value: &str) -> Result<Self, AppError> {
        match value {
            "admin" => Ok(Self::Admin),
            "editor" => Ok(Self::Editor),
            "viewer" => Ok(Self::Viewer),
            _ => Err(AppError::Validation("Invalid role")),
        }
    }
    pub fn can_manage_users(self) -> bool {
        self == Self::Admin
    }
    pub fn can_manage_connections(self) -> bool {
        self == Self::Admin
    }
    pub fn can_execute(self) -> bool {
        matches!(self, Self::Admin | Self::Editor | Self::Viewer)
    }
    pub fn can_write(self) -> bool {
        matches!(self, Self::Admin | Self::Editor)
    }
    pub fn can_view_audit(self) -> bool {
        self == Self::Admin
    }
}

#[derive(Clone, Serialize)]
pub struct CurrentUser {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub role: Role,
    pub active: bool,
}

#[derive(FromRow)]
struct UserRecord {
    id: Uuid,
    email: String,
    display_name: String,
    password_hash: String,
    role: String,
    active: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapInput {
    pub bootstrap_token: String,
    #[serde(flatten)]
    pub user: UserInput,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserInput {
    pub email: String,
    pub display_name: String,
    pub password: Option<String>,
    pub role: Role,
    #[serde(default = "active_default")]
    pub active: bool,
}
fn active_default() -> bool {
    true
}

#[derive(Serialize, FromRow)]
pub struct AuditView {
    pub id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub event_type: String,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub success: bool,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn require_session(
    State(service): State<Arc<crate::service::Service>>,
    mut request: Request,
    next: Next,
) -> Response {
    match service.auth.authenticate(request.headers()).await {
        Ok(user) => {
            request.extensions_mut().insert(user);
            next.run(request).await
        }
        Err(error) => error.into_response(),
    }
}

impl Auth {
    pub fn new(pool: PgPool, secure_cookie: bool, bootstrap_token: &str) -> Self {
        Self {
            pool,
            secure_cookie,
            bootstrap_token_hash: Sha256::digest(bootstrap_token.as_bytes()).into(),
            login_attempts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn bootstrap_required(&self) -> Result<bool, AppError> {
        Ok(sqlx::query_scalar::<_, i64>("SELECT count(*) FROM users")
            .fetch_one(&self.pool)
            .await?
            == 0)
    }

    pub async fn bootstrap(&self, token: &str, input: UserInput) -> Result<CurrentUser, AppError> {
        let supplied: [u8; 32] = Sha256::digest(token.as_bytes()).into();
        if supplied != self.bootstrap_token_hash {
            return Err(AppError::Unauthorized);
        }
        let mut tx = self.pool.begin().await?;
        sqlx::query("LOCK TABLE users IN EXCLUSIVE MODE")
            .execute(&mut *tx)
            .await?;
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
            .fetch_one(&mut *tx)
            .await?;
        if count != 0 {
            return Err(AppError::Forbidden);
        }
        validate_user_input(&input, true)?;
        if input.role != Role::Admin {
            return Err(AppError::Validation("First user must be an admin"));
        }
        let id = Uuid::new_v4();
        let password = input
            .password
            .as_deref()
            .ok_or(AppError::Validation("Password required"))?;
        let hash = hash_password(password)?;
        sqlx::query("INSERT INTO users(id,email,display_name,password_hash,role,active) VALUES($1,lower($2),$3,$4,'admin',true)")
            .bind(id).bind(&input.email).bind(&input.display_name).bind(hash).execute(&mut *tx).await?;
        audit_tx(
            &mut tx,
            Some(id),
            "user.bootstrap",
            "user",
            Some(id),
            true,
            serde_json::json!({"role":"admin"}),
        )
        .await?;
        tx.commit().await?;
        Ok(CurrentUser {
            id,
            email: input.email.to_lowercase(),
            display_name: input.display_name,
            role: Role::Admin,
            active: true,
        })
    }

    pub async fn login(&self, input: LoginInput) -> Result<(CurrentUser, HeaderValue), AppError> {
        let login_key: [u8; 32] =
            Sha256::digest(input.email.trim().to_ascii_lowercase().as_bytes()).into();
        if self.login_limited(login_key)? {
            self.audit(
                None,
                "auth.login_rate_limited",
                "user",
                None,
                false,
                serde_json::json!({}),
            )
            .await?;
            return Err(AppError::RateLimited);
        }
        let record = sqlx::query_as::<_, UserRecord>("SELECT id,email,display_name,password_hash,role,active FROM users WHERE email=lower($1)")
            .bind(input.email.trim()).fetch_optional(&self.pool).await?;
        let valid = record
            .as_ref()
            .is_some_and(|u| u.active && verify_password(&input.password, &u.password_hash));
        if !valid {
            self.record_login_failure(login_key)?;
            self.audit(
                record.as_ref().map(|u| u.id),
                "auth.login_failed",
                "user",
                record.as_ref().map(|u| u.id),
                false,
                serde_json::json!({}),
            )
            .await?;
            return Err(AppError::Unauthorized);
        }
        self.clear_login_failures(login_key)?;
        let record = record.expect("validated record");
        let user = current_user(record)?;
        let mut token = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut token);
        let encoded = URL_SAFE_NO_PAD.encode(token);
        let digest = Sha256::digest(token).to_vec();
        sqlx::query("DELETE FROM sessions WHERE expires_at <= now()")
            .execute(&self.pool)
            .await?;
        sqlx::query("INSERT INTO sessions(token_hash,user_id,expires_at) VALUES($1,$2,now()+interval '12 hours')")
            .bind(digest).bind(user.id).execute(&self.pool).await?;
        self.audit(
            Some(user.id),
            "auth.login",
            "user",
            Some(user.id),
            true,
            serde_json::json!({}),
        )
        .await?;
        let secure = if self.secure_cookie { "; Secure" } else { "" };
        let cookie = format!(
            "{COOKIE_NAME}={encoded}; Path=/; HttpOnly; SameSite=Strict; Max-Age=43200{secure}"
        );
        Ok((
            user,
            HeaderValue::from_str(&cookie).map_err(|_| AppError::Internal)?,
        ))
    }

    pub async fn authenticate(&self, headers: &HeaderMap) -> Result<CurrentUser, AppError> {
        let token = cookie_token(headers).ok_or(AppError::Unauthorized)?;
        let raw = URL_SAFE_NO_PAD
            .decode(token)
            .map_err(|_| AppError::Unauthorized)?;
        if raw.len() != 32 {
            return Err(AppError::Unauthorized);
        }
        let digest = Sha256::digest(raw).to_vec();
        let record = sqlx::query_as::<_, UserRecord>("SELECT u.id,u.email,u.display_name,u.password_hash,u.role,u.active FROM sessions s JOIN users u ON u.id=s.user_id WHERE s.token_hash=$1 AND s.expires_at>now()")
            .bind(digest).fetch_optional(&self.pool).await?.ok_or(AppError::Unauthorized)?;
        if !record.active {
            return Err(AppError::Unauthorized);
        }
        current_user(record)
    }

    pub async fn logout(
        &self,
        headers: &HeaderMap,
        user: &CurrentUser,
    ) -> Result<HeaderValue, AppError> {
        if let Some(token) = cookie_token(headers).and_then(|v| URL_SAFE_NO_PAD.decode(v).ok()) {
            sqlx::query("DELETE FROM sessions WHERE token_hash=$1")
                .bind(Sha256::digest(token).to_vec())
                .execute(&self.pool)
                .await?;
        }
        self.audit(
            Some(user.id),
            "auth.logout",
            "user",
            Some(user.id),
            true,
            serde_json::json!({}),
        )
        .await?;
        Ok(HeaderValue::from_static(
            "rustdb_session=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0",
        ))
    }

    pub async fn list_users(&self, actor: &CurrentUser) -> Result<Vec<CurrentUser>, AppError> {
        require(actor.role.can_manage_users())?;
        let rows: Vec<(Uuid, String, String, String, bool)> =
            sqlx::query_as("SELECT id,email,display_name,role,active FROM users ORDER BY email")
                .fetch_all(&self.pool)
                .await?;
        rows.into_iter()
            .map(|(id, email, display_name, role, active)| {
                Ok(CurrentUser {
                    id,
                    email,
                    display_name,
                    role: Role::parse(&role)?,
                    active,
                })
            })
            .collect()
    }

    pub async fn create_user(
        &self,
        actor: &CurrentUser,
        input: UserInput,
    ) -> Result<CurrentUser, AppError> {
        require(actor.role.can_manage_users())?;
        validate_user_input(&input, true)?;
        let id = Uuid::new_v4();
        let hash = hash_password(
            input
                .password
                .as_deref()
                .ok_or(AppError::Validation("Password required"))?,
        )?;
        sqlx::query("INSERT INTO users(id,email,display_name,password_hash,role,active) VALUES($1,lower($2),$3,$4,$5,$6)")
            .bind(id).bind(&input.email).bind(&input.display_name).bind(hash).bind(input.role.as_str()).bind(input.active).execute(&self.pool).await?;
        self.audit(
            Some(actor.id),
            "user.create",
            "user",
            Some(id),
            true,
            serde_json::json!({"role":input.role.as_str()}),
        )
        .await?;
        Ok(CurrentUser {
            id,
            email: input.email.to_lowercase(),
            display_name: input.display_name,
            role: input.role,
            active: input.active,
        })
    }

    pub async fn update_user(
        &self,
        actor: &CurrentUser,
        id: Uuid,
        input: UserInput,
    ) -> Result<CurrentUser, AppError> {
        require(actor.role.can_manage_users())?;
        validate_user_input(&input, false)?;
        if actor.id == id && (!input.active || input.role != Role::Admin) {
            return Err(AppError::Validation(
                "An admin cannot remove their own access",
            ));
        }
        let hash = match input.password.as_deref() {
            Some(password) => Some(hash_password(password)?),
            None => None,
        };
        let updated = sqlx::query("UPDATE users SET email=lower($2),display_name=$3,role=$4,active=$5,password_hash=COALESCE($6,password_hash),updated_at=now() WHERE id=$1")
            .bind(id).bind(&input.email).bind(&input.display_name).bind(input.role.as_str()).bind(input.active).bind(hash).execute(&self.pool).await?;
        if updated.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        sqlx::query("DELETE FROM sessions WHERE user_id=$1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        self.audit(
            Some(actor.id),
            "user.update",
            "user",
            Some(id),
            true,
            serde_json::json!({"role":input.role.as_str(),"active":input.active}),
        )
        .await?;
        Ok(CurrentUser {
            id,
            email: input.email.to_lowercase(),
            display_name: input.display_name,
            role: input.role,
            active: input.active,
        })
    }

    pub async fn delete_user(&self, actor: &CurrentUser, id: Uuid) -> Result<(), AppError> {
        require(actor.role.can_manage_users())?;
        if actor.id == id {
            return Err(AppError::Validation(
                "An admin cannot delete their own user",
            ));
        }
        let result = sqlx::query("DELETE FROM users WHERE id=$1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        self.audit(
            Some(actor.id),
            "user.delete",
            "user",
            Some(id),
            true,
            serde_json::json!({}),
        )
        .await
    }

    fn login_limited(&self, key: [u8; 32]) -> Result<bool, AppError> {
        let mut attempts = self.login_attempts.lock().map_err(|_| AppError::Internal)?;
        let now = Instant::now();
        attempts.retain(|_, (_, started)| now.duration_since(*started) < Duration::from_secs(900));
        Ok(attempts.get(&key).is_some_and(|(count, _)| *count >= 5))
    }

    fn record_login_failure(&self, key: [u8; 32]) -> Result<(), AppError> {
        let mut attempts = self.login_attempts.lock().map_err(|_| AppError::Internal)?;
        let entry = attempts.entry(key).or_insert((0, Instant::now()));
        entry.0 = entry.0.saturating_add(1);
        Ok(())
    }

    fn clear_login_failures(&self, key: [u8; 32]) -> Result<(), AppError> {
        self.login_attempts
            .lock()
            .map_err(|_| AppError::Internal)?
            .remove(&key);
        Ok(())
    }

    pub async fn list_audit(
        &self,
        actor: &CurrentUser,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditView>, AppError> {
        require(actor.role.can_view_audit())?;
        Ok(sqlx::query_as("SELECT id,actor_user_id,event_type,target_type,target_id,success,metadata,created_at FROM audit_logs ORDER BY created_at DESC LIMIT $1 OFFSET $2")
            .bind(limit).bind(offset).fetch_all(&self.pool).await?)
    }

    pub async fn audit(
        &self,
        actor: Option<Uuid>,
        event: &str,
        target: &str,
        target_id: Option<Uuid>,
        success: bool,
        metadata: serde_json::Value,
    ) -> Result<(), AppError> {
        sqlx::query("INSERT INTO audit_logs(id,actor_user_id,event_type,target_type,target_id,success,metadata) VALUES($1,$2,$3,$4,$5,$6,$7)")
            .bind(Uuid::new_v4()).bind(actor).bind(event).bind(target).bind(target_id).bind(success).bind(metadata).execute(&self.pool).await?;
        Ok(())
    }
}

pub fn require(condition: bool) -> Result<(), AppError> {
    if condition {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

fn cookie_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .map(str::trim)
        .find_map(|part| part.strip_prefix(&format!("{COOKIE_NAME}=")))
}
fn current_user(record: UserRecord) -> Result<CurrentUser, AppError> {
    Ok(CurrentUser {
        id: record.id,
        email: record.email,
        display_name: record.display_name,
        role: Role::parse(&record.role)?,
        active: record.active,
    })
}
fn validate_user_input(input: &UserInput, creating: bool) -> Result<(), AppError> {
    if input.email.trim().len() < 3
        || input.email.len() > 254
        || !input.email.contains('@')
        || input.email.chars().any(char::is_control)
    {
        return Err(AppError::Validation("Invalid email"));
    }
    if input.display_name.trim().is_empty()
        || input.display_name.len() > 128
        || input.display_name.chars().any(char::is_control)
    {
        return Err(AppError::Validation("Invalid display name"));
    }
    if creating && input.password.is_none() {
        return Err(AppError::Validation("Password required"));
    }
    if let Some(p) = &input.password
        && (p.len() < 12 || p.len() > 1024)
    {
        return Err(AppError::Validation(
            "Password must have 12 to 1024 characters",
        ));
    }
    Ok(())
}
fn hash_password(password: &str) -> Result<String, AppError> {
    Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .map(|v| v.to_string())
        .map_err(|_| AppError::Internal)
}
fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash).ok().is_some_and(|parsed| {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    })
}
async fn audit_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    actor: Option<Uuid>,
    event: &str,
    target: &str,
    target_id: Option<Uuid>,
    success: bool,
    metadata: serde_json::Value,
) -> Result<(), AppError> {
    sqlx::query("INSERT INTO audit_logs(id,actor_user_id,event_type,target_type,target_id,success,metadata) VALUES($1,$2,$3,$4,$5,$6,$7)")
        .bind(Uuid::new_v4()).bind(actor).bind(event).bind(target).bind(target_id).bind(success).bind(metadata).execute(&mut **tx).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn passwords_are_hashed_and_roles_enforce_permissions() -> Result<(), AppError> {
        let hash = hash_password("a-secure-test-password")?;
        assert_ne!(hash, "a-secure-test-password");
        assert!(hash.starts_with("$argon2"));
        assert!(verify_password("a-secure-test-password", &hash));
        assert!(!verify_password("wrong-password", &hash));
        assert!(Role::Admin.can_manage_users());
        assert!(!Role::Editor.can_manage_users());
        assert!(Role::Editor.can_write());
        assert!(!Role::Viewer.can_write());
        Ok(())
    }
}
