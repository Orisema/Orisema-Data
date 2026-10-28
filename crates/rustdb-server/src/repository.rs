use crate::{
    error::AppError,
    models::{ConnectionInput, ConnectionRecord},
};
use sqlx::PgPool;
use uuid::Uuid;

pub struct Repository(pub PgPool);
impl Repository {
    pub async fn list(&self) -> Result<Vec<ConnectionRecord>, AppError> {
        Ok(
            sqlx::query_as("SELECT * FROM connections ORDER BY name, id")
                .fetch_all(&self.0)
                .await?,
        )
    }
    pub async fn get(&self, id: Uuid) -> Result<ConnectionRecord, AppError> {
        sqlx::query_as("SELECT * FROM connections WHERE id=$1")
            .bind(id)
            .fetch_optional(&self.0)
            .await?
            .ok_or(AppError::NotFound)
    }
    pub async fn save(
        &self,
        id: Uuid,
        input: &ConnectionInput,
        encrypted: &str,
        creating: bool,
    ) -> Result<ConnectionRecord, AppError> {
        let sql = if creating {
            "INSERT INTO connections (id,name,database_type,host,port,database_name,username,encrypted_password,read_only,ssl_mode) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) RETURNING *"
        } else {
            "UPDATE connections SET name=$2,database_type=$3,host=$4,port=$5,database_name=$6,username=$7,encrypted_password=$8,read_only=$9,ssl_mode=$10,updated_at=now() WHERE id=$1 RETURNING *"
        };
        let ssl = match input.ssl_mode {
            orisema_data_core::driver::SslMode::Disable => "disable",
            orisema_data_core::driver::SslMode::VerifyFull => "verify_full",
        };
        sqlx::query_as(sql)
            .bind(id)
            .bind(&input.name)
            .bind(&input.database_type)
            .bind(&input.host)
            .bind(i32::from(input.port))
            .bind(&input.database_name)
            .bind(&input.username)
            .bind(encrypted)
            .bind(input.read_only)
            .bind(ssl)
            .fetch_optional(&self.0)
            .await?
            .ok_or(AppError::NotFound)
    }
    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        if sqlx::query("DELETE FROM connections WHERE id=$1")
            .bind(id)
            .execute(&self.0)
            .await?
            .rows_affected()
            == 0
        {
            return Err(AppError::NotFound);
        }
        Ok(())
    }
}
