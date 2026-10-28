use crate::{
    crypto::Credentials,
    error::AppError,
    models::{ConnectionInput, ConnectionView},
    repository::Repository,
};
use orisema_data_core::driver::{ConnectionConfig, DatabaseDriver, DriverError, QueryResult, SslMode};
use std::{collections::HashMap, sync::Arc};
use tokio::{sync::Mutex, task::AbortHandle};
use uuid::Uuid;

pub struct Service {
    pub repository: Repository,
    pub credentials: Credentials,
    pub driver: Arc<dyn DatabaseDriver>,
    pub active_queries: ActiveQueries,
    pub max_rows: usize,
    pub query_timeout: u64,
    pub auth: crate::auth::Auth,
}

#[derive(Clone)]
struct ActiveQuery {
    owner_id: Uuid,
    handle: AbortHandle,
}

#[derive(Clone, Default)]
pub struct ActiveQueries(Arc<Mutex<HashMap<Uuid, ActiveQuery>>>);

impl Service {
    pub async fn execute_query(
        &self,
        query_id: Uuid,
        connection_id: Uuid,
        sql: String,
        max_rows: usize,
        user: &crate::auth::CurrentUser,
    ) -> Result<QueryResult, AppError> {
        let policy = crate::connection_policy::get(
            &self.repository.0,
            connection_id,
            self.max_rows,
            self.query_timeout,
        )
        .await?;
        let (max_rows, policy_timeout) =
            crate::connection_policy::enforce(&policy, user, &sql, max_rows)?;
        let target = self.target(connection_id).await?;
        let driver = Arc::clone(&self.driver);
        let task = tokio::spawn(async move {
            tokio::time::timeout(
                policy_timeout,
                driver.execute_query(&target, &sql, max_rows),
            )
            .await
            .map_err(|_| DriverError::Timeout)?
        });
        {
            let mut active = self.active_queries.0.lock().await;
            if active.contains_key(&query_id) {
                task.abort();
                return Err(AppError::Validation("query_id is already active"));
            }
            active.insert(
                query_id,
                ActiveQuery {
                    owner_id: user.id,
                    handle: task.abort_handle(),
                },
            );
        }
        let result = task.await;
        self.active_queries.0.lock().await.remove(&query_id);
        match result {
            Ok(result) => result.map_err(Into::into),
            Err(error) if error.is_cancelled() => Err(DriverError::Cancelled.into()),
            Err(_) => Err(AppError::Internal),
        }
    }

    pub async fn cancel_query(
        &self,
        query_id: Uuid,
        user: &crate::auth::CurrentUser,
    ) -> Result<bool, AppError> {
        let mut active = self.active_queries.0.lock().await;
        let Some(query) = active.get(&query_id) else {
            return Ok(false);
        };
        if query.owner_id != user.id && user.role != crate::auth::Role::Admin {
            return Err(AppError::Forbidden);
        }
        let query = active.remove(&query_id).expect("active query was checked");
        query.handle.abort();
        Ok(true)
    }
    pub async fn save(
        &self,
        id: Option<Uuid>,
        input: ConnectionInput,
    ) -> Result<ConnectionView, AppError> {
        input.validate(id.is_none())?;
        let existing = match id {
            Some(id) => Some(self.repository.get(id).await?),
            None => None,
        };
        let id = id.unwrap_or_else(Uuid::new_v4);
        let encrypted = match &input.password {
            Some(password) => self.credentials.encrypt(id, password)?,
            None => existing
                .as_ref()
                .ok_or(AppError::Validation("Password required"))?
                .encrypted_password
                .clone(),
        };
        Ok(self
            .repository
            .save(id, &input, &encrypted, existing.is_none())
            .await?
            .into())
    }
    pub async fn target(&self, id: Uuid) -> Result<ConnectionConfig, AppError> {
        let r = self.repository.get(id).await?;
        Ok(ConnectionConfig {
            database_type: r.database_type,
            host: r.host,
            port: r.port as u16,
            database_name: r.database_name,
            username: r.username,
            password: self.credentials.decrypt(id, &r.encrypted_password)?,
            read_only: r.read_only,
            ssl_mode: match r.ssl_mode.as_str() {
                "disable" => SslMode::Disable,
                _ => SslMode::VerifyFull,
            },
        })
    }
}
