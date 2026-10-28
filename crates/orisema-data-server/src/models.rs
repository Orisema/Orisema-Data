use crate::error::AppError;
use orisema_data_core::driver::SslMode;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectionInput {
    pub name: String,
    pub database_type: String,
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub username: String,
    pub password: Option<String>,
    #[serde(default = "read_only_default")]
    pub read_only: bool,
    #[serde(default)]
    pub ssl_mode: SslMode,
}
fn read_only_default() -> bool {
    true
}
impl ConnectionInput {
    pub fn validate(&self, creating: bool) -> Result<(), AppError> {
        for (value, max) in [
            (&self.name, 128),
            (&self.host, 253),
            (&self.database_name, 128),
            (&self.username, 128),
        ] {
            if value.trim().is_empty() || value.len() > max || value.chars().any(char::is_control) {
                return Err(AppError::Validation(
                    "Connection fields are empty, too long or contain control characters",
                ));
            }
        }
        if !matches!(
            self.database_type.as_str(),
            "postgresql" | "mysql" | "mariadb" | "sqlserver"
        ) || self.port == 0
        {
            return Err(AppError::Validation(
                "Database type must be postgresql, mysql, mariadb or sqlserver and the port must be 1–65535",
            ));
        }
        if self.host.contains(['/', '\\', '@', ' ', '?', '#']) {
            return Err(AppError::Validation(
                "Host must be an IP address or hostname, not a URL or socket path",
            ));
        }
        if creating && self.password.is_none() {
            return Err(AppError::Validation(
                "Password is required on creation (empty is allowed for local trust authentication)",
            ));
        }
        if self
            .password
            .as_ref()
            .is_some_and(|p| p.len() > 4096 || p.contains('\0'))
        {
            return Err(AppError::Validation(
                "Invalid password length or NUL character",
            ));
        }
        Ok(())
    }
}

#[derive(FromRow)]
pub struct ConnectionRecord {
    pub id: Uuid,
    pub name: String,
    pub database_type: String,
    pub host: String,
    pub port: i32,
    pub database_name: String,
    pub username: String,
    pub encrypted_password: String,
    pub read_only: bool,
    pub ssl_mode: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct ConnectionView {
    pub id: Uuid,
    pub name: String,
    pub database_type: String,
    pub host: String,
    pub port: i32,
    pub database_name: String,
    pub username: String,
    pub has_password: bool,
    pub read_only: bool,
    pub ssl_mode: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
impl From<ConnectionRecord> for ConnectionView {
    fn from(r: ConnectionRecord) -> Self {
        Self {
            id: r.id,
            name: r.name,
            database_type: r.database_type,
            host: r.host,
            port: r.port,
            database_name: r.database_name,
            username: r.username,
            has_password: true,
            read_only: r.read_only,
            ssl_mode: r.ssl_mode,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validation_defaults_and_password_semantics() -> Result<(), Box<dyn std::error::Error>> {
        let mut input: ConnectionInput = serde_json::from_value(
            serde_json::json!({"name":"Local", "database_type":"postgresql", "host":"localhost", "port":5432, "database_name":"test", "username":"reader"}),
        )?;
        assert!(input.read_only);
        assert!(input.validate(true).is_err());
        assert!(input.validate(false).is_ok());
        input.password = Some(String::new());
        assert!(input.validate(true).is_ok());
        for database_type in ["mysql", "mariadb", "sqlserver"] {
            input.database_type = database_type.into();
            assert!(input.validate(true).is_ok());
        }
        input.database_type = "unsupported".into();
        assert!(input.validate(true).is_err());
        input.database_type = "postgresql".into();
        input.host = "postgres://secret@host".into();
        assert!(input.validate(false).is_err());
        input.host = "localhost".into();
        input.port = 0;
        assert!(input.validate(false).is_err());
        Ok(())
    }
}
