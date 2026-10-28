use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use futures_util::TryStreamExt;
use orisema_data_core::driver::{
    ColumnInfo, ConnectionConfig, DatabaseDriver, DriverError, QueryResult, ResultColumn, SslMode,
    TableInfo,
};
use serde_json::Value;
use std::time::Duration;
use tiberius::{AuthMethod, Client, ColumnData, Config, EncryptionLevel, QueryItem};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

type SqlServerClient = Client<Compat<TcpStream>>;

pub struct SqlServerDriver {
    pub connect_timeout: Duration,
    pub query_timeout: Duration,
}

impl SqlServerDriver {
    async fn connect(&self, config: &ConnectionConfig) -> Result<SqlServerClient, DriverError> {
        let mut options = Config::new();
        options.host(&config.host);
        options.port(config.port);
        options.database(&config.database_name);
        options.application_name("rustdb");
        options.authentication(AuthMethod::sql_server(&config.username, &config.password));
        options.readonly(config.read_only);
        if matches!(config.ssl_mode, SslMode::Disable) {
            options.encryption(EncryptionLevel::NotSupported);
        }
        let tcp =
            tokio::time::timeout(self.connect_timeout, TcpStream::connect(options.get_addr()))
                .await
                .map_err(|_| DriverError::Timeout)?
                .map_err(|_| DriverError::Connection)?;
        tcp.set_nodelay(true).map_err(|_| DriverError::Connection)?;
        tokio::time::timeout(
            self.connect_timeout,
            Client::connect(options, tcp.compat_write()),
        )
        .await
        .map_err(|_| DriverError::Timeout)?
        .map_err(|_| DriverError::Connection)
    }
}

#[async_trait]
impl DatabaseDriver for SqlServerDriver {
    async fn test_connection(&self, config: &ConnectionConfig) -> Result<(), DriverError> {
        let mut client = self.connect(config).await?;
        tokio::time::timeout(self.connect_timeout, client.simple_query("SELECT 1"))
            .await
            .map_err(|_| DriverError::Timeout)?
            .map_err(|_| DriverError::Connection)?
            .into_row()
            .await
            .map_err(|_| DriverError::Connection)?;
        Ok(())
    }

    async fn list_schemas(&self, config: &ConnectionConfig) -> Result<Vec<String>, DriverError> {
        let mut client = self.connect(config).await?;
        let rows = tokio::time::timeout(
            self.query_timeout,
            client.query("SELECT TOP (10000) name FROM sys.schemas WHERE name NOT IN ('sys','INFORMATION_SCHEMA') ORDER BY name", &[]),
        ).await.map_err(|_| DriverError::Timeout)?.map_err(db_error)?
            .into_first_result().await.map_err(db_error)?;
        rows.into_iter()
            .map(|row| {
                row.try_get::<&str, _>(0)
                    .map(|v| v.unwrap_or_default().to_owned())
                    .map_err(db_error)
            })
            .collect()
    }

    async fn list_tables(
        &self,
        config: &ConnectionConfig,
        schema: &str,
    ) -> Result<Vec<TableInfo>, DriverError> {
        let mut client = self.connect(config).await?;
        let rows = tokio::time::timeout(
            self.query_timeout,
            client.query("SELECT TOP (10000) t.name, CASE WHEN t.type = 'V' THEN 'view' ELSE 'table' END FROM sys.objects t JOIN sys.schemas s ON s.schema_id=t.schema_id WHERE s.name=@P1 AND t.type IN ('U','V') ORDER BY t.name", &[&schema]),
        ).await.map_err(|_| DriverError::Timeout)?.map_err(db_error)?
            .into_first_result().await.map_err(db_error)?;
        rows.into_iter()
            .map(|row| {
                Ok(TableInfo {
                    name: row
                        .try_get::<&str, _>(0)
                        .map_err(db_error)?
                        .unwrap_or_default()
                        .to_owned(),
                    kind: row
                        .try_get::<&str, _>(1)
                        .map_err(db_error)?
                        .unwrap_or("table")
                        .to_owned(),
                })
            })
            .collect()
    }

    async fn list_columns(
        &self,
        config: &ConnectionConfig,
        schema: &str,
        table: &str,
    ) -> Result<Vec<ColumnInfo>, DriverError> {
        let mut client = self.connect(config).await?;
        let rows = tokio::time::timeout(
            self.query_timeout,
            client.query("SELECT TOP (10000) c.name, ty.name, c.is_nullable FROM sys.columns c JOIN sys.objects o ON o.object_id=c.object_id JOIN sys.schemas s ON s.schema_id=o.schema_id JOIN sys.types ty ON ty.user_type_id=c.user_type_id WHERE s.name=@P1 AND o.name=@P2 ORDER BY c.column_id", &[&schema, &table]),
        ).await.map_err(|_| DriverError::Timeout)?.map_err(db_error)?
            .into_first_result().await.map_err(db_error)?;
        rows.into_iter()
            .map(|row| {
                Ok(ColumnInfo {
                    name: row
                        .try_get::<&str, _>(0)
                        .map_err(db_error)?
                        .unwrap_or_default()
                        .to_owned(),
                    data_type: row
                        .try_get::<&str, _>(1)
                        .map_err(db_error)?
                        .unwrap_or_default()
                        .to_owned(),
                    nullable: row
                        .try_get::<bool, _>(2)
                        .map_err(db_error)?
                        .unwrap_or(false),
                })
            })
            .collect()
    }

    async fn execute_query(
        &self,
        config: &ConnectionConfig,
        sql: &str,
        max_rows: usize,
    ) -> Result<QueryResult, DriverError> {
        let is_select = crate::policy::validate(sql, config.read_only)?;
        if !(1..=10000).contains(&max_rows) {
            return Err(DriverError::Policy);
        }
        let start = std::time::Instant::now();
        let mut client = self.connect(config).await?;
        tokio::time::timeout(self.query_timeout, async {
            if !is_select {
                let result = client.execute(sql, &[]).await.map_err(db_error)?;
                return Ok(QueryResult {
                    columns: vec![],
                    rows: vec![],
                    row_count: 0,
                    affected_rows: result.total(),
                    execution_time_ms: start.elapsed().as_millis(),
                    truncated: false,
                });
            }
            let mut stream = client.query(sql, &[]).await.map_err(db_error)?;
            let mut columns = Vec::new();
            let mut rows = Vec::new();
            let mut bytes = 0usize;
            let mut truncated = false;
            while let Some(item) = stream.try_next().await.map_err(db_error)? {
                match item {
                    QueryItem::Metadata(metadata) if columns.is_empty() => {
                        columns = metadata
                            .columns()
                            .iter()
                            .map(|column| ResultColumn {
                                name: column.name().to_owned(),
                                data_type: format!("{:?}", column.column_type()).to_lowercase(),
                            })
                            .collect();
                    }
                    QueryItem::Row(row) => {
                        if rows.len() >= max_rows {
                            truncated = true;
                            break;
                        }
                        let mut values = Vec::with_capacity(row.columns().len());
                        for (_, value) in row.cells() {
                            bytes += 32;
                            let value = sqlserver_value(value);
                            bytes += value.to_string().len().saturating_mul(6);
                            if bytes > 8 * 1024 * 1024 {
                                return Err(DriverError::ResultTooLarge);
                            }
                            values.push(value);
                        }
                        rows.push(values);
                    }
                    _ => {}
                }
            }
            Ok(QueryResult {
                columns,
                row_count: rows.len(),
                rows,
                affected_rows: 0,
                execution_time_ms: start.elapsed().as_millis(),
                truncated,
            })
        })
        .await
        .map_err(|_| DriverError::Timeout)?
    }
}

fn sqlserver_value(value: &ColumnData<'_>) -> Value {
    match value {
        ColumnData::U8(Some(v)) => Value::from(*v),
        ColumnData::I16(Some(v)) => Value::from(*v),
        ColumnData::I32(Some(v)) => Value::from(*v),
        ColumnData::I64(Some(v)) => Value::String(v.to_string()),
        ColumnData::F32(Some(v)) => Value::String(v.to_string()),
        ColumnData::F64(Some(v)) => Value::String(v.to_string()),
        ColumnData::Bit(Some(v)) => Value::Bool(*v),
        ColumnData::String(Some(v)) => Value::String(v.to_string()),
        ColumnData::Guid(Some(v)) => Value::String(v.to_string()),
        ColumnData::Binary(Some(v)) => Value::String(BASE64.encode(v.as_ref())),
        ColumnData::Numeric(Some(v)) => Value::String(v.to_string()),
        ColumnData::Xml(Some(v)) => Value::String(v.to_string()),
        ColumnData::DateTime(Some(v)) => Value::String(format!("{v:?}")),
        ColumnData::SmallDateTime(Some(v)) => Value::String(format!("{v:?}")),
        ColumnData::Time(Some(v)) => Value::String(format!("{v:?}")),
        ColumnData::Date(Some(v)) => Value::String(format!("{v:?}")),
        ColumnData::DateTime2(Some(v)) => Value::String(format!("{v:?}")),
        ColumnData::DateTimeOffset(Some(v)) => Value::String(format!("{v:?}")),
        _ => Value::Null,
    }
}

fn db_error(_: tiberius::error::Error) -> DriverError {
    DriverError::Query
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn real_sqlserver_driver() -> Result<(), Box<dyn std::error::Error>> {
        let Ok(host) = std::env::var("RUSTDB_SQLSERVER_TEST_HOST") else {
            return Ok(());
        };
        let mut config = ConnectionConfig {
            database_type: "sqlserver".into(),
            host,
            port: std::env::var("RUSTDB_SQLSERVER_TEST_PORT")?.parse()?,
            database_name: std::env::var("RUSTDB_SQLSERVER_TEST_DATABASE")?,
            username: std::env::var("RUSTDB_SQLSERVER_TEST_USERNAME")?,
            password: std::env::var("RUSTDB_SQLSERVER_TEST_PASSWORD")?,
            read_only: true,
            ssl_mode: SslMode::Disable,
        };
        let driver = SqlServerDriver {
            connect_timeout: Duration::from_secs(10),
            query_timeout: Duration::from_secs(3),
        };
        driver
            .test_connection(&config)
            .await
            .map_err(|_| "test_connection")?;
        let mut setup = driver
            .connect(&config)
            .await
            .map_err(|_| "setup_connection")?;
        setup.simple_query("IF OBJECT_ID('dbo.rustdb_v04_driver_test', 'U') IS NOT NULL DROP TABLE dbo.rustdb_v04_driver_test").await?.into_results().await?;
        setup.simple_query("CREATE TABLE dbo.rustdb_v04_driver_test (id INT PRIMARY KEY, label NVARCHAR(64)); INSERT INTO dbo.rustdb_v04_driver_test VALUES (1,N'one'),(2,N'two'),(3,N'three')").await?.into_results().await?;
        assert!(
            driver
                .list_schemas(&config)
                .await?
                .iter()
                .any(|schema| schema == "dbo")
        );
        assert!(
            driver
                .list_tables(&config, "dbo")
                .await?
                .iter()
                .any(|table| table.name == "rustdb_v04_driver_test")
        );
        assert_eq!(
            driver
                .list_columns(&config, "dbo", "rustdb_v04_driver_test")
                .await?
                .len(),
            2
        );
        let result = driver
            .execute_query(
                &config,
                "SELECT id,label FROM dbo.rustdb_v04_driver_test ORDER BY id",
                2,
            )
            .await?;
        assert_eq!(result.row_count, 2);
        assert!(result.truncated);
        assert!(
            driver
                .execute_query(
                    &config,
                    "UPDATE dbo.rustdb_v04_driver_test SET label='blocked'",
                    10
                )
                .await
                .is_err()
        );
        config.read_only = false;
        let changed = driver
            .execute_query(
                &config,
                "UPDATE dbo.rustdb_v04_driver_test SET label='changed' WHERE id=1",
                10,
            )
            .await?;
        assert_eq!(changed.affected_rows, 1);
        let mut cleanup = driver.connect(&config).await?;
        cleanup
            .simple_query("DROP TABLE dbo.rustdb_v04_driver_test")
            .await?
            .into_results()
            .await?;
        Ok(())
    }
}
