use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Deliberately no Debug or Serialize: this value contains a plaintext secret.
pub struct ConnectionConfig {
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub username: String,
    pub password: String,
    pub read_only: bool,
    pub ssl_mode: SslMode,
}

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SslMode {
    Disable,
    #[default]
    VerifyFull,
}

#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("Database connection failed; check host, credentials and TLS settings")]
    Connection,
    #[error("Database operation timed out")]
    Timeout,
    #[error("Query rejected by the database; check SQL, permissions and column names")]
    Query,
    #[error(
        "Only one SELECT, INSERT, UPDATE or DELETE statement is supported; read-only connections only allow SELECT"
    )]
    Policy,
    #[error("Result exceeds the response size limit")]
    ResultTooLarge,
}

#[async_trait]
pub trait DatabaseDriver: Send + Sync {
    async fn test_connection(&self, config: &ConnectionConfig) -> Result<(), DriverError>;
}
