use crate::{
    error::AppError,
    models::{ConnectionInput, ConnectionView},
    service::Service,
};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, State, rejection::JsonRejection},
    http::StatusCode,
    routing::{get, post},
};
use std::sync::Arc;
use uuid::Uuid;
type AppState = State<Arc<Service>>;

pub fn router(service: Arc<Service>) -> Router {
    Router::new()
        .route(
            "/health",
            get(|| async { Json(serde_json::json!({"status":"ok"})) }),
        )
        .route("/connections", get(list).post(create))
        .route("/connections/{id}", get(read).put(update).delete(delete))
        .route("/connections/{id}/test", post(test))
        .layer(DefaultBodyLimit::max(128 * 1024))
        .with_state(service)
}
async fn list(State(s): AppState) -> Result<Json<Vec<ConnectionView>>, AppError> {
    Ok(Json(
        s.repository
            .list()
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}
async fn read(State(s): AppState, Path(id): Path<Uuid>) -> Result<Json<ConnectionView>, AppError> {
    Ok(Json(s.repository.get(id).await?.into()))
}
async fn create(
    State(s): AppState,
    body: Result<Json<ConnectionInput>, JsonRejection>,
) -> Result<(StatusCode, Json<ConnectionView>), AppError> {
    Ok((StatusCode::CREATED, Json(s.save(None, json(body)?).await?)))
}
async fn update(
    State(s): AppState,
    Path(id): Path<Uuid>,
    body: Result<Json<ConnectionInput>, JsonRejection>,
) -> Result<Json<ConnectionView>, AppError> {
    Ok(Json(s.save(Some(id), json(body)?).await?))
}
async fn delete(State(s): AppState, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    s.repository.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn test(
    State(s): AppState,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    s.driver.test_connection(&s.target(id).await?).await?;
    Ok(Json(serde_json::json!({"status":"ok"})))
}
pub fn json<T>(body: Result<Json<T>, JsonRejection>) -> Result<T, AppError> {
    body.map(|Json(v)| v)
        .map_err(|_| AppError::Validation("Invalid JSON body or field types"))
}
