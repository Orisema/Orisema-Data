use crate::{
    auth::{BootstrapInput, CurrentUser, LoginInput, UserInput, require},
    error::AppError,
    models::{ConnectionInput, ConnectionView},
    service::Service,
};
use axum::{
    Extension, Json, Router,
    extract::{DefaultBodyLimit, Path, Query, State, rejection::JsonRejection},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use std::sync::Arc;
use uuid::Uuid;
type AppState = State<Arc<Service>>;

pub fn router(service: Arc<Service>, max_rows: usize) -> Router {
    let protected = Router::new()
        .route("/auth/me", get(me))
        .route("/auth/logout", post(logout))
        .route("/users", get(users).post(create_user))
        .route(
            "/users/{id}",
            axum::routing::put(update_user).delete(delete_user),
        )
        .route("/audit", get(audit))
        .route("/connections", get(list).post(create))
        .route("/connections/{id}", get(read).put(update).delete(delete))
        .route("/connections/{id}/test", post(test))
        .route(
            "/connections/{id}/policy",
            get(get_policy).put(update_policy),
        )
        .route("/connections/{id}/schemas", get(schemas))
        .route("/connections/{id}/schemas/{schema}/tables", get(tables))
        .route(
            "/connections/{id}/schemas/{schema}/tables/{table}/columns",
            get(columns),
        )
        .route("/connections/{id}/query", post(query))
        .route("/queries/{id}", axum::routing::delete(cancel_query))
        .route_layer(axum::middleware::from_fn_with_state(
            Arc::clone(&service),
            crate::auth::require_session,
        ));
    Router::new()
        .route(
            "/health",
            get(|| async { Json(serde_json::json!({"status":"ok"})) }),
        )
        .route("/auth/bootstrap-status", get(bootstrap_status))
        .route("/auth/bootstrap", post(bootstrap))
        .route("/auth/login", post(login))
        .merge(protected)
        .layer(axum::Extension(max_rows))
        .layer(DefaultBodyLimit::max(128 * 1024))
        .with_state(service)
}

async fn bootstrap_status(State(s): AppState) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(
        serde_json::json!({"required":s.auth.bootstrap_required().await?}),
    ))
}
async fn bootstrap(
    State(s): AppState,
    body: Result<Json<BootstrapInput>, JsonRejection>,
) -> Result<(StatusCode, Json<CurrentUser>), AppError> {
    let input = json(body)?;
    Ok((
        StatusCode::CREATED,
        Json(s.auth.bootstrap(&input.bootstrap_token, input.user).await?),
    ))
}
async fn login(
    State(s): AppState,
    body: Result<Json<LoginInput>, JsonRejection>,
) -> Result<impl IntoResponse, AppError> {
    let (user, cookie) = s.auth.login(json(body)?).await?;
    Ok(([(header::SET_COOKIE, cookie)], Json(user)))
}
async fn logout(
    State(s): AppState,
    Extension(user): Extension<CurrentUser>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let cookie = s.auth.logout(&headers, &user).await?;
    Ok(([(header::SET_COOKIE, cookie)], StatusCode::NO_CONTENT))
}
async fn me(Extension(user): Extension<CurrentUser>) -> Json<CurrentUser> {
    Json(user)
}
async fn users(
    State(s): AppState,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Vec<CurrentUser>>, AppError> {
    Ok(Json(s.auth.list_users(&user).await?))
}
async fn create_user(
    State(s): AppState,
    Extension(user): Extension<CurrentUser>,
    body: Result<Json<UserInput>, JsonRejection>,
) -> Result<(StatusCode, Json<CurrentUser>), AppError> {
    Ok((
        StatusCode::CREATED,
        Json(s.auth.create_user(&user, json(body)?).await?),
    ))
}
async fn update_user(
    State(s): AppState,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    body: Result<Json<UserInput>, JsonRejection>,
) -> Result<Json<CurrentUser>, AppError> {
    Ok(Json(s.auth.update_user(&user, id, json(body)?).await?))
}
async fn delete_user(
    State(s): AppState,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    s.auth.delete_user(&user, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
#[derive(serde::Deserialize)]
struct AuditQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}
async fn audit(
    State(s): AppState,
    Extension(user): Extension<CurrentUser>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Vec<crate::auth::AuditView>>, AppError> {
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);
    if !(1..=1000).contains(&limit) || offset < 0 {
        return Err(AppError::Validation("Invalid audit pagination"));
    }
    Ok(Json(s.auth.list_audit(&user, limit, offset).await?))
}

async fn schemas(
    State(s): AppState,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<String>>, AppError> {
    require_connection_access(&s, &user, id).await?;
    Ok(Json(s.driver.list_schemas(&s.target(id).await?).await?))
}
async fn tables(
    State(s): AppState,
    Extension(user): Extension<CurrentUser>,
    Path((id, schema)): Path<(Uuid, String)>,
) -> Result<Json<Vec<orisema_data_core::driver::TableInfo>>, AppError> {
    require_connection_access(&s, &user, id).await?;
    identifier(&schema)?;
    Ok(Json(
        s.driver.list_tables(&s.target(id).await?, &schema).await?,
    ))
}
async fn columns(
    State(s): AppState,
    Extension(user): Extension<CurrentUser>,
    Path((id, schema, table)): Path<(Uuid, String, String)>,
) -> Result<Json<Vec<orisema_data_core::driver::ColumnInfo>>, AppError> {
    require_connection_access(&s, &user, id).await?;
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
        Err(AppError::Validation("Invalid schema or table name"))
    } else {
        Ok(())
    }
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
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    axum::Extension(limit): axum::Extension<usize>,
    body: Result<Json<QueryInput>, JsonRejection>,
) -> Result<Json<orisema_data_core::driver::QueryResult>, AppError> {
    require(user.role.can_execute())?;
    let body = json(body)?;
    let max_rows = body.max_rows.unwrap_or(limit);
    if max_rows == 0 || max_rows > limit {
        return Err(AppError::Validation(
            "max_rows must be positive and no greater than the server limit",
        ));
    }
    let query_id = body.query_id.unwrap_or_else(Uuid::new_v4);
    let result = s
        .execute_query(query_id, id, body.sql, max_rows, &user)
        .await;
    s.auth
        .audit(
            Some(user.id),
            "query.execute",
            "connection",
            Some(id),
            result.is_ok(),
            serde_json::json!({"query_id":query_id,"max_rows":max_rows}),
        )
        .await?;
    Ok(Json(result?))
}
async fn cancel_query(
    State(s): AppState,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    require(user.role.can_execute())?;
    let cancelled = s.cancel_query(id, &user).await?;
    s.auth
        .audit(
            Some(user.id),
            "query.cancel",
            "query",
            Some(id),
            cancelled,
            serde_json::json!({}),
        )
        .await?;
    if cancelled {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}
async fn list(
    State(s): AppState,
    Extension(user): Extension<CurrentUser>,
) -> Result<Json<Vec<ConnectionView>>, AppError> {
    let mut visible = Vec::new();
    for record in s.repository.list().await? {
        let policy =
            crate::connection_policy::get(&s.repository.0, record.id, s.max_rows, s.query_timeout)
                .await?;
        if crate::connection_policy::permits(&policy, &user) {
            visible.push(record.into());
        }
    }
    Ok(Json(visible))
}
async fn read(
    State(s): AppState,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ConnectionView>, AppError> {
    require_connection_access(&s, &user, id).await?;
    Ok(Json(s.repository.get(id).await?.into()))
}
async fn create(
    State(s): AppState,
    Extension(user): Extension<CurrentUser>,
    body: Result<Json<ConnectionInput>, JsonRejection>,
) -> Result<(StatusCode, Json<ConnectionView>), AppError> {
    require(user.role.can_manage_connections())?;
    let view = s.save(None, json(body)?).await?;
    s.auth
        .audit(
            Some(user.id),
            "connection.create",
            "connection",
            Some(view.id),
            true,
            serde_json::json!({}),
        )
        .await?;
    Ok((StatusCode::CREATED, Json(view)))
}
async fn update(
    State(s): AppState,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    body: Result<Json<ConnectionInput>, JsonRejection>,
) -> Result<Json<ConnectionView>, AppError> {
    require(user.role.can_manage_connections())?;
    let view = s.save(Some(id), json(body)?).await?;
    s.auth
        .audit(
            Some(user.id),
            "connection.update",
            "connection",
            Some(id),
            true,
            serde_json::json!({}),
        )
        .await?;
    Ok(Json(view))
}
async fn delete(
    State(s): AppState,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    require(user.role.can_manage_connections())?;
    s.repository.delete(id).await?;
    s.auth
        .audit(
            Some(user.id),
            "connection.delete",
            "connection",
            Some(id),
            true,
            serde_json::json!({}),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn get_policy(
    State(s): AppState,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::connection_policy::ConnectionPolicy>, AppError> {
    require(user.role.can_manage_connections())?;
    Ok(Json(
        crate::connection_policy::get(&s.repository.0, id, s.max_rows, s.query_timeout).await?,
    ))
}
async fn update_policy(
    State(s): AppState,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    body: Result<Json<crate::connection_policy::PolicyInput>, JsonRejection>,
) -> Result<Json<crate::connection_policy::ConnectionPolicy>, AppError> {
    let policy = crate::connection_policy::save(&s.repository.0, &user, id, json(body)?).await?;
    s.auth
        .audit(
            Some(user.id),
            "connection.policy_update",
            "connection",
            Some(id),
            true,
            serde_json::json!({}),
        )
        .await?;
    Ok(Json(policy))
}
async fn test(
    State(s): AppState,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    require_connection_access(&s, &user, id).await?;
    s.driver.test_connection(&s.target(id).await?).await?;
    Ok(Json(serde_json::json!({"status":"ok"})))
}
async fn require_connection_access(
    s: &Service,
    user: &CurrentUser,
    id: Uuid,
) -> Result<(), AppError> {
    require(user.role.can_execute())?;
    let policy =
        crate::connection_policy::get(&s.repository.0, id, s.max_rows, s.query_timeout).await?;
    require(crate::connection_policy::permits(&policy, user))
}
pub fn json<T>(body: Result<Json<T>, JsonRejection>) -> Result<T, AppError> {
    body.map(|Json(v)| v)
        .map_err(|_| AppError::Validation("Invalid JSON body or field types"))
}
