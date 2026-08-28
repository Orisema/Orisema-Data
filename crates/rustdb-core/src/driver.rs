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
    async fn list_schemas(&self, config: &ConnectionConfig) -> Result<Vec<String>, DriverError>;
    async fn list_tables(
        &self,
        config: &ConnectionConfig,
        schema: &str,
    ) -> Result<Vec<TableInfo>, DriverError>;
    async fn list_columns(
        &self,
        config: &ConnectionConfig,
        schema: &str,
        table: &str,
    ) -> Result<Vec<ColumnInfo>, DriverError>;
    async fn execute_query(
        &self,
        config: &ConnectionConfig,
        sql: &str,
        max_rows: usize,
    ) -> Result<QueryResult, DriverError>;
}

#[derive(Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub kind: String,
}
#[derive(Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub nullable: bool,
}
#[derive(Serialize, Deserialize)]
pub struct ResultColumn {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: String,
}
#[derive(Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<ResultColumn>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
    pub affected_rows: u64,
    pub execution_time_ms: u128,
    pub truncated: bool,
}
