use async_trait::async_trait;
use futures_util::TryStreamExt;
use orisema_data_core::driver::{
    ColumnInfo, ConnectionConfig, DatabaseDriver, DriverError, QueryResult, ResultColumn, SslMode,
    TableInfo,
};
use serde_json::Value;
use sqlx::{
    Column, ConnectOptions, Connection, Executor, PgConnection, Row, Statement, TypeInfo, ValueRef,
    postgres::{PgConnectOptions, PgSslMode},
};
use std::time::Duration;

pub struct PostgresDriver {
    pub connect_timeout: Duration,
    pub query_timeout: Duration,
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
            .options([
                (
                    "statement_timeout",
                    self.query_timeout.as_millis().to_string(),
                ),
                (
                    "idle_in_transaction_session_timeout",
                    self.query_timeout.as_millis().to_string(),
                ),
            ])
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
    async fn list_schemas(&self, config: &ConnectionConfig) -> Result<Vec<String>, DriverError> {
        let mut conn = self.connect(config).await?;
        tokio::time::timeout(self.query_timeout, sqlx::query_scalar("SELECT schema_name::text FROM information_schema.schemata WHERE schema_name NOT LIKE 'pg_%' AND schema_name <> 'information_schema' ORDER BY schema_name LIMIT 10000").fetch_all(&mut conn)).await.map_err(|_|DriverError::Timeout)?.map_err(db_error)
    }
    async fn list_tables(
        &self,
        config: &ConnectionConfig,
        schema: &str,
    ) -> Result<Vec<TableInfo>, DriverError> {
        let mut conn = self.connect(config).await?;
        let rows: Vec<(String,String)> = tokio::time::timeout(self.query_timeout, sqlx::query_as("SELECT table_name::text, table_type::text FROM information_schema.tables WHERE table_schema=$1 ORDER BY table_name LIMIT 10000").bind(schema).fetch_all(&mut conn)).await.map_err(|_|DriverError::Timeout)?.map_err(db_error)?;
        Ok(rows
            .into_iter()
            .map(|(name, kind)| TableInfo {
                name,
                kind: if kind == "VIEW" {
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
        let rows: Vec<(String,String,String)> = tokio::time::timeout(self.query_timeout, sqlx::query_as("SELECT column_name::text, data_type::text, is_nullable::text FROM information_schema.columns WHERE table_schema=$1 AND table_name=$2 ORDER BY ordinal_position LIMIT 10000").bind(schema).bind(table).fetch_all(&mut conn)).await.map_err(|_|DriverError::Timeout)?.map_err(db_error)?;
        Ok(rows
            .into_iter()
            .map(|(name, data_type, nullable)| ColumnInfo {
                name,
                data_type,
                nullable: nullable == "YES",
            })
            .collect())
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
        let mut conn = self.connect(config).await?;
        // Dedicated connection is dropped on timeout; it is never returned to a pool.
        tokio::time::timeout(self.query_timeout, async {
            let mut tx = conn.begin().await.map_err(db_error)?;
            if config.read_only {
                sqlx::query("SET TRANSACTION READ ONLY")
                    .execute(&mut *tx)
                    .await
                    .map_err(db_error)?;
            }
            let prepared = (&mut *tx).prepare(sql).await.map_err(db_error)?;
            let columns = prepared
                .columns()
                .iter()
                .map(|c| ResultColumn {
                    name: c.name().into(),
                    data_type: c.type_info().name().to_lowercase(),
                })
                .collect();
            // A cursor caps work transferred for SELECT without rewriting user SQL.
            let command = if is_select {
                (&mut *tx)
                    .execute(format!("DECLARE rustdb_result NO SCROLL CURSOR FOR {sql}").as_str())
                    .await
                    .map_err(db_error)?;
                format!("FETCH FORWARD {} FROM rustdb_result", max_rows + 1)
            } else {
                sql.to_owned()
            };
            let mut stream = (&mut *tx).fetch_many(command.as_str());
            let mut rows = Vec::new();
            let mut bytes = 0;
            let mut affected_rows = 0;
            let mut truncated = false;
            while let Some(item) = stream.try_next().await.map_err(db_error)? {
                match item {
                    sqlx::Either::Left(result) => {
                        if !is_select {
                            affected_rows = result.rows_affected();
                        }
                    }
                    sqlx::Either::Right(row) => {
                        if rows.len() >= max_rows {
                            truncated = true;
                            continue;
                        }
                        let mut values = Vec::with_capacity(row.len());
                        for (index, column) in row.columns().iter().enumerate() {
                            bytes += 32; // Include NULLs and per-cell allocation/JSON overhead.
                            if bytes > 8 * 1024 * 1024 {
                                return Err(DriverError::ResultTooLarge);
                            }
                            let raw = row.try_get_raw(index).map_err(db_error)?;
                            if raw.is_null() {
                                values.push(Value::Null);
                                continue;
                            }
                            let text = raw.as_str().map_err(|_| DriverError::Query)?;
                            bytes += text.len().saturating_mul(6); // Worst-case JSON escaping.
                            if bytes > 8 * 1024 * 1024 {
                                return Err(DriverError::ResultTooLarge);
                            }
                            let value = match column.type_info().name() {
                                "BOOL" => Value::Bool(text == "t"),
                                "INT2" | "INT4" => text
                                    .parse::<i32>()
                                    .map(Value::from)
                                    .map_err(|_| DriverError::Query)?,
                                // Exact numbers, JSON, arrays and extension types retain their PostgreSQL text representation.
                                _ => Value::String(text.into()),
                            };
                            values.push(value);
                        }
                        rows.push(values);
                    }
                }
            }
            drop(stream);
            tx.commit().await.map_err(db_error)?;
            Ok(QueryResult {
                columns,
                row_count: rows.len(),
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

fn db_error(error: sqlx::Error) -> DriverError {
    if error
        .as_database_error()
        .and_then(|e| e.code())
        .is_some_and(|code| code == "57014")
    {
        DriverError::Timeout
    } else {
        DriverError::Query
    }
}
