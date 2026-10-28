use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use orisema_data_core::driver::DriverError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(&'static str),
    #[error("Connection not found")]
    NotFound,
    #[error("Internal operation failed")]
    Internal,
    #[error(transparent)]
    Driver(#[from] DriverError),
}
impl From<sqlx::Error> for AppError {
    fn from(_: sqlx::Error) -> Self {
        Self::Internal
    }
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            Self::Validation(_) => (StatusCode::BAD_REQUEST, "invalid_input"),
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found"),
            Self::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            Self::Driver(DriverError::Timeout) => (StatusCode::GATEWAY_TIMEOUT, "timeout"),
            Self::Driver(DriverError::Cancelled) => (StatusCode::CONFLICT, "query_cancelled"),
            Self::Driver(DriverError::Connection) => (StatusCode::BAD_GATEWAY, "connection_failed"),
            Self::Driver(_) => (StatusCode::UNPROCESSABLE_ENTITY, "query_rejected"),
        };
        if status.is_server_error() {
            tracing::warn!(error_code = code, "Request failed");
        }
        (
            status,
            Json(serde_json::json!({"error": {"code": code, "message": self.to_string()}})),
        )
            .into_response()
    }
}
