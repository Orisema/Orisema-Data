use async_trait::async_trait;
use orisema_data_core::driver::{
    ColumnInfo, ConnectionConfig, DatabaseDriver, DriverError, QueryResult, TableInfo,
};
use std::sync::Arc;

pub struct MultiDriver {
    pub postgres: Arc<dyn DatabaseDriver>,
    pub mysql: Arc<dyn DatabaseDriver>,
    pub sqlserver: Arc<dyn DatabaseDriver>,
}

impl MultiDriver {
    fn select(&self, config: &ConnectionConfig) -> Result<&Arc<dyn DatabaseDriver>, DriverError> {
        match config.database_type.as_str() {
            "postgresql" => Ok(&self.postgres),
            "mysql" | "mariadb" => Ok(&self.mysql),
            "sqlserver" => Ok(&self.sqlserver),
            _ => Err(DriverError::Connection),
        }
    }
}

#[async_trait]
impl DatabaseDriver for MultiDriver {
    async fn test_connection(&self, config: &ConnectionConfig) -> Result<(), DriverError> {
        self.select(config)?.test_connection(config).await
    }
    async fn list_schemas(&self, config: &ConnectionConfig) -> Result<Vec<String>, DriverError> {
        self.select(config)?.list_schemas(config).await
    }
    async fn list_tables(
        &self,
        config: &ConnectionConfig,
        schema: &str,
    ) -> Result<Vec<TableInfo>, DriverError> {
        self.select(config)?.list_tables(config, schema).await
    }
    async fn list_columns(
        &self,
        config: &ConnectionConfig,
        schema: &str,
        table: &str,
    ) -> Result<Vec<ColumnInfo>, DriverError> {
        self.select(config)?
            .list_columns(config, schema, table)
            .await
    }
    async fn execute_query(
        &self,
        config: &ConnectionConfig,
        sql: &str,
        max_rows: usize,
    ) -> Result<QueryResult, DriverError> {
        self.select(config)?
            .execute_query(config, sql, max_rows)
            .await
    }
}
