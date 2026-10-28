use crate::{
    auth::{CurrentUser, Role, require},
    error::AppError,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct ConnectionPolicy {
    pub connection_id: Uuid,
    pub allowed_roles: Vec<Role>,
    pub can_write: bool,
    pub max_rows: usize,
    pub timeout_seconds: u64,
    pub prohibited_commands: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyInput {
    pub allowed_roles: Vec<Role>,
    pub can_write: bool,
    pub max_rows: usize,
    pub timeout_seconds: u64,
    #[serde(default)]
    pub prohibited_commands: Vec<String>,
}

#[derive(FromRow)]
struct PolicyRecord {
    connection_id: Uuid,
    allowed_roles: Vec<String>,
    can_write: bool,
    max_rows: i32,
    timeout_seconds: i32,
    prohibited_commands: Vec<String>,
}

pub async fn get(
    pool: &PgPool,
    connection_id: Uuid,
    server_rows: usize,
    server_timeout: u64,
) -> Result<ConnectionPolicy, AppError> {
    let row=sqlx::query_as::<_,PolicyRecord>("SELECT connection_id,allowed_roles,can_write,max_rows,timeout_seconds,prohibited_commands FROM connection_policies WHERE connection_id=$1")
        .bind(connection_id).fetch_optional(pool).await?;
    match row {
        Some(r) => Ok(ConnectionPolicy {
            connection_id: r.connection_id,
            allowed_roles: r
                .allowed_roles
                .iter()
                .map(|v| Role::parse(v))
                .collect::<Result<_, _>>()?,
            can_write: r.can_write,
            max_rows: r.max_rows as usize,
            timeout_seconds: r.timeout_seconds as u64,
            prohibited_commands: r.prohibited_commands,
        }),
        None => Ok(ConnectionPolicy {
            connection_id,
            allowed_roles: vec![Role::Admin],
            can_write: false,
            max_rows: server_rows,
            timeout_seconds: server_timeout,
            prohibited_commands: vec![],
        }),
    }
}

pub async fn save(
    pool: &PgPool,
    actor: &CurrentUser,
    connection_id: Uuid,
    input: PolicyInput,
) -> Result<ConnectionPolicy, AppError> {
    require(actor.role.can_manage_connections())?;
    validate(&input)?;
    let roles: Vec<&str> = input.allowed_roles.iter().map(|r| r.as_str()).collect();
    let commands: Vec<String> = input
        .prohibited_commands
        .iter()
        .map(|v| v.trim().to_ascii_uppercase())
        .collect();
    sqlx::query("INSERT INTO connection_policies(connection_id,allowed_roles,can_write,max_rows,timeout_seconds,prohibited_commands,updated_by) VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT(connection_id) DO UPDATE SET allowed_roles=$2,can_write=$3,max_rows=$4,timeout_seconds=$5,prohibited_commands=$6,updated_by=$7,updated_at=now()")
        .bind(connection_id).bind(roles).bind(input.can_write).bind(input.max_rows as i32).bind(input.timeout_seconds as i32).bind(&commands).bind(actor.id).execute(pool).await?;
    Ok(ConnectionPolicy {
        connection_id,
        allowed_roles: input.allowed_roles,
        can_write: input.can_write,
        max_rows: input.max_rows,
        timeout_seconds: input.timeout_seconds,
        prohibited_commands: commands,
    })
}

pub fn permits(policy: &ConnectionPolicy, user: &CurrentUser) -> bool {
    policy.allowed_roles.contains(&user.role)
}

pub fn enforce(
    policy: &ConnectionPolicy,
    user: &CurrentUser,
    sql: &str,
    requested_rows: usize,
) -> Result<(usize, std::time::Duration), AppError> {
    require(permits(policy, user))?;
    let allow_write = policy.can_write && user.role.can_write();
    let is_select = crate::policy::validate(sql, !allow_write)?;
    let command = if is_select {
        "SELECT"
    } else {
        sql.split_whitespace().next().unwrap_or("")
    };
    if policy
        .prohibited_commands
        .iter()
        .any(|v| v.eq_ignore_ascii_case(command))
    {
        return Err(AppError::Forbidden);
    }
    Ok((
        requested_rows.min(policy.max_rows),
        std::time::Duration::from_secs(policy.timeout_seconds),
    ))
}

fn validate(input: &PolicyInput) -> Result<(), AppError> {
    if input.allowed_roles.is_empty()
        || input.max_rows == 0
        || input.max_rows > 10000
        || input.timeout_seconds == 0
        || input.timeout_seconds > 300
    {
        return Err(AppError::Validation(
            "Invalid connection policy limits or roles",
        ));
    }
    for command in &input.prohibited_commands {
        if !matches!(
            command.trim().to_ascii_uppercase().as_str(),
            "SELECT" | "INSERT" | "UPDATE" | "DELETE"
        ) {
            return Err(AppError::Validation(
                "Prohibited commands must use SELECT, INSERT, UPDATE or DELETE",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn user(role: Role) -> CurrentUser {
        CurrentUser {
            id: Uuid::nil(),
            email: "u@example.test".into(),
            display_name: "U".into(),
            role,
            active: true,
        }
    }
    #[test]
    fn policy_allows_and_denies_roles_writes_limits_and_commands() {
        let policy = ConnectionPolicy {
            connection_id: Uuid::nil(),
            allowed_roles: vec![Role::Editor, Role::Viewer],
            can_write: true,
            max_rows: 50,
            timeout_seconds: 2,
            prohibited_commands: vec!["DELETE".into()],
        };
        assert_eq!(
            enforce(&policy, &user(Role::Viewer), "SELECT 1", 100)
                .unwrap()
                .0,
            50
        );
        assert!(enforce(&policy, &user(Role::Viewer), "UPDATE x SET y=1", 10).is_err());
        assert!(enforce(&policy, &user(Role::Editor), "DELETE FROM x", 10).is_err());
        assert!(enforce(&policy, &user(Role::Admin), "SELECT 1", 10).is_err());
    }
}
