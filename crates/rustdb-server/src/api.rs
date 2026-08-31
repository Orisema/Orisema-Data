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

pub fn router(service: Arc<Service>, max_rows: usize) -> Router {
    Router::new()
        .route(
            "/health",
            get(|| async { Json(serde_json::json!({"status":"ok"})) }),
        )
        .route("/connections", get(list).post(create))
        .route("/connections/{id}", get(read).put(update).delete(delete))
        .route("/connections/{id}/test", post(test))
        .route("/connections/{id}/schemas", get(schemas))
        .route("/connections/{id}/schemas/{schema}/tables", get(tables))
        .route(
            "/connections/{id}/schemas/{schema}/tables/{table}/columns",
            get(columns),
        )
        .route("/connections/{id}/query", post(query))
        .route("/queries/{id}", axum::routing::delete(cancel_query))
        .layer(axum::Extension(max_rows))
        .layer(DefaultBodyLimit::max(128 * 1024))
        .with_state(service)
}

async fn schemas(State(s): AppState, Path(id): Path<Uuid>) -> Result<Json<Vec<String>>, AppError> {
    Ok(Json(s.driver.list_schemas(&s.target(id).await?).await?))
}
async fn tables(
    State(s): AppState,
    Path((id, schema)): Path<(Uuid, String)>,
) -> Result<Json<Vec<rustdb_core::driver::TableInfo>>, AppError> {
    identifier(&schema)?;
    Ok(Json(
        s.driver.list_tables(&s.target(id).await?, &schema).await?,
    ))
}
async fn columns(
    State(s): AppState,
    Path((id, schema, table)): Path<(Uuid, String, String)>,
) -> Result<Json<Vec<rustdb_core::driver::ColumnInfo>>, AppError> {
    identifier(&schema)?;
    identifier(&table)?;
    Ok(Json(
        s.driver
            .list_columns(&s.target(id).await?, &schema, &table)
            .await?,
    ))
}
fn identifier(value: &str) -> Result<(), AppError> {
    if value.is_empty() || value.len() > 128 || value.contains('\0') {
        return Err(AppError::Validation("Invalid schema or table name"));
    }
    Ok(())
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct QueryInput {
    sql: String,
    max_rows: Option<usize>,
    query_id: Option<Uuid>,
}
async fn query(
    State(s): AppState,
    Path(id): Path<Uuid>,
    axum::Extension(limit): axum::Extension<usize>,
    body: Result<Json<QueryInput>, JsonRejection>,
) -> Result<Json<rustdb_core::driver::QueryResult>, AppError> {
    let body = json(body)?;
    let max_rows = body.max_rows.unwrap_or(limit);
    if max_rows == 0 || max_rows > limit {
        return Err(AppError::Validation(
            "max_rows must be positive and no greater than the server limit",
        ));
    }
    Ok(Json(
        s.execute_query(
            body.query_id.unwrap_or_else(Uuid::new_v4),
            id,
            body.sql,
            max_rows,
        )
        .await?,
    ))
}
async fn cancel_query(State(s): AppState, Path(id): Path<Uuid>) -> Result<StatusCode, AppError> {
    if s.cancel_query(id).await {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
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
