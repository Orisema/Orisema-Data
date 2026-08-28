use async_trait::async_trait;
use rustdb_core::driver::{ConnectionConfig, DatabaseDriver, DriverError, SslMode};
use sqlx::{
    ConnectOptions, Connection, PgConnection,
    postgres::{PgConnectOptions, PgSslMode},
};
use std::time::Duration;

pub struct PostgresDriver {
    pub connect_timeout: Duration,
}
impl PostgresDriver {
    async fn connect(&self, config: &ConnectionConfig) -> Result<PgConnection, DriverError> {
        let options = PgConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .database(&config.database_name)
            .username(&config.username)
            .password(&config.password)
            .ssl_mode(match config.ssl_mode {
                SslMode::Disable => PgSslMode::Disable,
                SslMode::VerifyFull => PgSslMode::VerifyFull,
            })
            .application_name("rustdb")
            .disable_statement_logging();
        tokio::time::timeout(self.connect_timeout, PgConnection::connect_with(&options))
            .await
            .map_err(|_| DriverError::Timeout)?
            .map_err(|_| DriverError::Connection)
    }
}
#[async_trait]
impl DatabaseDriver for PostgresDriver {
    async fn test_connection(&self, config: &ConnectionConfig) -> Result<(), DriverError> {
        let mut conn = self.connect(config).await?;
        tokio::time::timeout(
            self.connect_timeout,
            sqlx::query("SELECT 1").execute(&mut conn),
        )
        .await
        .map_err(|_| DriverError::Timeout)?
        .map_err(|_| DriverError::Connection)?;
        Ok(())
    }
}
