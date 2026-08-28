use crate::{
    crypto::Credentials,
    error::AppError,
    models::{ConnectionInput, ConnectionView},
    repository::Repository,
};
use rustdb_core::driver::{ConnectionConfig, DatabaseDriver, SslMode};
use std::sync::Arc;
use uuid::Uuid;

pub struct Service {
    pub repository: Repository,
    pub credentials: Credentials,
    pub driver: Arc<dyn DatabaseDriver>,
}
impl Service {
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
