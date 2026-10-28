use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use futures_util::TryStreamExt;
use orisema_data_core::driver::{
    ColumnInfo, ConnectionConfig, DatabaseDriver, DriverError, QueryResult, ResultColumn, SslMode,
    TableInfo,
};
use serde_json::Value;
use sqlx::{
    Column, ConnectOptions, Connection, Executor, MySqlConnection, Row, Statement, TypeInfo,
    ValueRef,
    mysql::{MySqlConnectOptions, MySqlSslMode},
};
use std::time::Duration;

pub struct MySqlDriver {
    pub connect_timeout: Duration,
    pub query_timeout: Duration,
}

impl MySqlDriver {
    async fn connect(&self, config: &ConnectionConfig) -> Result<MySqlConnection, DriverError> {
        let options = MySqlConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .database(&config.database_name)
            .username(&config.username)
            .password(&config.password)
            .ssl_mode(match config.ssl_mode {
                SslMode::Disable => MySqlSslMode::Disabled,
                SslMode::VerifyFull => MySqlSslMode::VerifyIdentity,
            })
            .disable_statement_logging();
        tokio::time::timeout(
            self.connect_timeout,
            MySqlConnection::connect_with(&options),
        )
        .await
        .map_err(|_| DriverError::Timeout)?
        .map_err(|_| DriverError::Connection)
    }
}

#[async_trait]
impl DatabaseDriver for MySqlDriver {
    async fn test_connection(&self, config: &ConnectionConfig) -> Result<(), DriverError> {
        let mut conn = self.connect(config).await?;
        tokio::time::timeout(
            self.connect_timeout,
            sqlx::query("SELECT 1").execute(&mut conn),
        )
        .await
        .map_err(|_| DriverError::Timeout)?
        .map_err(db_error)?;
        Ok(())
    }

    async fn list_schemas(&self, config: &ConnectionConfig) -> Result<Vec<String>, DriverError> {
        let mut conn = self.connect(config).await?;
        tokio::time::timeout(
            self.query_timeout,
            sqlx::query_scalar::<_, Vec<u8>>("SELECT schema_name FROM information_schema.schemata WHERE schema_name NOT IN ('information_schema','mysql','performance_schema','sys') ORDER BY schema_name LIMIT 10000")
                .fetch_all(&mut conn),
        )
        .await
        .map_err(|_| DriverError::Timeout)?
        .map_err(db_error)
        .and_then(|rows| rows.into_iter().map(|value| String::from_utf8(value).map_err(|_| DriverError::Query)).collect())
    }

    async fn list_tables(
        &self,
        config: &ConnectionConfig,
        schema: &str,
    ) -> Result<Vec<TableInfo>, DriverError> {
        let mut conn = self.connect(config).await?;
        let rows: Vec<(Vec<u8>, Vec<u8>)> = tokio::time::timeout(
            self.query_timeout,
            sqlx::query_as("SELECT table_name, table_type FROM information_schema.tables WHERE table_schema=? ORDER BY table_name LIMIT 10000")
                .bind(schema)
                .fetch_all(&mut conn),
        )
        .await
        .map_err(|_| DriverError::Timeout)?
        .map_err(db_error)?;
        Ok(rows
            .into_iter()
            .map(|(name, kind)| TableInfo {
                name: String::from_utf8_lossy(&name).into_owned(),
                kind: if kind == b"VIEW" {
                    "view".into()
                } else {
                    "table".into()
                },
            })
            .collect())
    }

    async fn list_columns(
        &self,
        config: &ConnectionConfig,
        schema: &str,
        table: &str,
    ) -> Result<Vec<ColumnInfo>, DriverError> {
        let mut conn = self.connect(config).await?;
        let rows: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)> = tokio::time::timeout(
            self.query_timeout,
            sqlx::query_as("SELECT column_name, column_type, is_nullable FROM information_schema.columns WHERE table_schema=? AND table_name=? ORDER BY ordinal_position LIMIT 10000")
                .bind(schema)
                .bind(table)
                .fetch_all(&mut conn),
        )
        .await
        .map_err(|_| DriverError::Timeout)?
        .map_err(db_error)?;
        Ok(rows
            .into_iter()
            .map(|(name, data_type, nullable)| ColumnInfo {
                name: String::from_utf8_lossy(&name).into_owned(),
                data_type: String::from_utf8_lossy(&data_type).into_owned(),
                nullable: nullable == b"YES",
            })
            .collect())
    }

    async fn execute_query(
        &self,
        config: &ConnectionConfig,
        sql: &str,
        max_rows: usize,
    ) -> Result<QueryResult, DriverError> {
        let _is_select = crate::policy::validate(sql, config.read_only)?;
        if !(1..=10000).contains(&max_rows) {
            return Err(DriverError::Policy);
        }
        let start = std::time::Instant::now();
        let mut conn = self.connect(config).await?;
        tokio::time::timeout(self.query_timeout, async {
            if config.read_only {
                sqlx::query("SET TRANSACTION READ ONLY")
                    .execute(&mut conn)
                    .await
                    .map_err(db_error)?;
            }
            let mut tx = conn.begin().await.map_err(db_error)?;
            let prepared = (&mut *tx).prepare(sql).await.map_err(db_error)?;
            let mut columns: Vec<ResultColumn> = prepared
                .columns()
                .iter()
                .map(|column| ResultColumn {
                    name: column.name().into(),
                    data_type: column.type_info().name().to_lowercase(),
                })
                .collect();
            let mut stream = (&mut *tx).fetch_many(sql);
            let mut rows = Vec::new();
            let mut affected_rows = 0;
            let mut bytes = 0usize;
            let mut truncated = false;
            while let Some(item) = stream.try_next().await.map_err(db_error)? {
                match item {
                    sqlx::Either::Left(result) => affected_rows = result.rows_affected(),
                    sqlx::Either::Right(row) => {
                        if columns.is_empty() {
                            columns = row
                                .columns()
                                .iter()
                                .map(|column| ResultColumn {
                                    name: column.name().into(),
                                    data_type: column.type_info().name().to_lowercase(),
                                })
                                .collect();
                        }
                        if rows.len() >= max_rows {
                            truncated = true;
                            break;
                        }
                        let mut values = Vec::with_capacity(row.len());
                        for index in 0..row.len() {
                            bytes += 32;
                            if bytes > 8 * 1024 * 1024 {
                                return Err(DriverError::ResultTooLarge);
                            }
                            let raw = row.try_get_raw(index).map_err(db_error)?;
                            let value = if raw.is_null() {
                                Value::Null
                            } else {
                                mysql_value(&row, index)?
                            };
                            bytes += value.to_string().len().saturating_mul(6);
                            if bytes > 8 * 1024 * 1024 {
                                return Err(DriverError::ResultTooLarge);
                            }
                            values.push(value);
                        }
                        rows.push(values);
                    }
                }
            }
            drop(stream);
            tx.commit().await.map_err(db_error)?;
            Ok(QueryResult {
                row_count: rows.len(),
                columns,
                rows,
                affected_rows,
                execution_time_ms: start.elapsed().as_millis(),
                truncated,
            })
        })
        .await
        .map_err(|_| DriverError::Timeout)?
    }
}

fn db_error(_: sqlx::Error) -> DriverError {
    DriverError::Query
}

fn mysql_value(row: &sqlx::mysql::MySqlRow, index: usize) -> Result<Value, DriverError> {
    let type_name = row.column(index).type_info().name().to_ascii_uppercase();
    match type_name.as_str() {
        "BOOLEAN" | "BOOL" => row
            .try_get::<bool, _>(index)
            .map(Value::Bool)
            .map_err(db_error),
        "TINYINT" => row
            .try_get::<i8, _>(index)
            .map(Value::from)
            .map_err(db_error),
        "TINYINT UNSIGNED" => row
            .try_get::<u8, _>(index)
            .map(Value::from)
            .map_err(db_error),
        "SMALLINT" => row
            .try_get::<i16, _>(index)
            .map(Value::from)
            .map_err(db_error),
        "SMALLINT UNSIGNED" => row
            .try_get::<u16, _>(index)
            .map(Value::from)
            .map_err(db_error),
        "INT" | "MEDIUMINT" => row
            .try_get::<i32, _>(index)
            .map(Value::from)
            .map_err(db_error),
        "INT UNSIGNED" | "MEDIUMINT UNSIGNED" => row
            .try_get::<u32, _>(index)
            .map(Value::from)
            .map_err(db_error),
        "BIGINT" => row
            .try_get::<i64, _>(index)
            .map(|v| Value::String(v.to_string()))
            .map_err(db_error),
        "BIGINT UNSIGNED" => row
            .try_get::<u64, _>(index)
            .map(|v| Value::String(v.to_string()))
            .map_err(db_error),
        "FLOAT" => row
            .try_get::<f32, _>(index)
            .map(|v| Value::String(v.to_string()))
            .map_err(db_error),
        "DOUBLE" => row
            .try_get::<f64, _>(index)
            .map(|v| Value::String(v.to_string()))
            .map_err(db_error),
        "DECIMAL" => row
            .try_get::<rust_decimal::Decimal, _>(index)
            .map(|v| Value::String(v.to_string()))
            .map_err(db_error),
        "JSON" => row
            .try_get::<Value, _>(index)
            .map(|v| Value::String(v.to_string()))
            .map_err(db_error),
        "DATE" => row
            .try_get::<chrono::NaiveDate, _>(index)
            .map(|v| Value::String(v.to_string()))
            .map_err(db_error),
        "TIME" => row
            .try_get::<chrono::NaiveTime, _>(index)
            .map(|v| Value::String(v.to_string()))
            .map_err(db_error),
        "DATETIME" | "TIMESTAMP" => row
            .try_get::<chrono::NaiveDateTime, _>(index)
            .map(|v| Value::String(v.to_string()))
            .map_err(db_error),
        "BINARY" | "VARBINARY" | "BLOB" => row
            .try_get::<Vec<u8>, _>(index)
            .map(|v| Value::String(BASE64.encode(v)))
            .map_err(db_error),
        _ => row
            .try_get::<String, _>(index)
            .map(Value::String)
            .map_err(db_error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn real_mysql_or_mariadb_driver() -> Result<(), Box<dyn std::error::Error>> {
        let Ok(host) = std::env::var("RUSTDB_MYSQL_TEST_HOST") else {
            return Ok(());
        };
        let mut config = ConnectionConfig {
            database_type: std::env::var("RUSTDB_MYSQL_TEST_TYPE")
                .unwrap_or_else(|_| "mysql".into()),
            host,
            port: std::env::var("RUSTDB_MYSQL_TEST_PORT")?.parse()?,
            database_name: std::env::var("RUSTDB_MYSQL_TEST_DATABASE")?,
            username: std::env::var("RUSTDB_MYSQL_TEST_USERNAME")?,
            password: std::env::var("RUSTDB_MYSQL_TEST_PASSWORD").unwrap_or_default(),
            read_only: true,
            ssl_mode: SslMode::Disable,
        };
        let driver = MySqlDriver {
            connect_timeout: Duration::from_secs(5),
            query_timeout: Duration::from_secs(2),
        };
        driver
            .test_connection(&config)
            .await
            .map_err(|_| "test_connection")?;
        let mut setup = driver
            .connect(&config)
            .await
            .map_err(|_| "setup_connection")?;
        sqlx::query("DROP TABLE IF EXISTS rustdb_v04_driver_test")
            .execute(&mut setup)
            .await?;
        sqlx::query("CREATE TABLE rustdb_v04_driver_test (id INT PRIMARY KEY, label VARCHAR(64), amount DECIMAL(10,2), INDEX idx_label(label))").execute(&mut setup).await?;
        sqlx::query("INSERT INTO rustdb_v04_driver_test VALUES (1,'one',1.25),(2,'two',2.50),(3,'three',3.75)").execute(&mut setup).await?;
        assert!(
            driver
                .list_schemas(&config)
                .await
                .map_err(|_| "list_schemas")?
                .contains(&config.database_name)
        );
        assert!(
            driver
                .list_tables(&config, &config.database_name)
                .await
                .map_err(|_| "list_tables")?
                .iter()
                .any(|table| table.name == "rustdb_v04_driver_test")
        );
        let columns = driver
            .list_columns(&config, &config.database_name, "rustdb_v04_driver_test")
            .await
            .map_err(|_| "list_columns")?;
        assert_eq!(columns.len(), 3);
        let result = driver
            .execute_query(
                &config,
                "SELECT id,label,amount FROM rustdb_v04_driver_test ORDER BY id",
                2,
            )
            .await?;
        assert_eq!(result.row_count, 2);
        assert!(result.truncated);
        assert!(
            driver
                .execute_query(
                    &config,
                    "UPDATE rustdb_v04_driver_test SET label='blocked'",
                    10
                )
                .await
                .is_err()
        );
        config.read_only = false;
        let changed = driver
            .execute_query(
                &config,
                "UPDATE rustdb_v04_driver_test SET label='changed' WHERE id=1",
                10,
            )
            .await?;
        assert_eq!(changed.affected_rows, 1);
        let mut cleanup = driver.connect(&config).await?;
        sqlx::query("DROP TABLE rustdb_v04_driver_test")
            .execute(&mut cleanup)
            .await?;
        Ok(())
    }
}
