use crate::{
    api, crypto::Credentials, postgres::PostgresDriver, repository::Repository, service::Service,
};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tower::ServiceExt;

async fn request(
    app: &Router,
    method: &str,
    path: &str,
    body: Value,
) -> Result<(StatusCode, Value), Box<dyn std::error::Error>> {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))?,
        )
        .await?;
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 20 * 1024 * 1024).await?;
    Ok((
        status,
        if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes)?
        },
    ))
}

#[tokio::test]
#[ignore = "Requires RUSTDB_TEST_DATABASE_URL pointing to a disposable PostgreSQL database"]
async fn postgres_api_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("RUSTDB_TEST_DATABASE_URL")?;
    let pool = sqlx::PgPool::connect(&url).await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;
    let options = url.parse::<sqlx::postgres::PgConnectOptions>()?;
    let password = std::env::var("RUSTDB_TEST_PASSWORD").unwrap_or_default();
    let service = Arc::new(Service {
        repository: Repository(pool.clone()),
        credentials: Credentials::new(&STANDARD.encode([9; 32]))?,
        driver: Arc::new(PostgresDriver {
            connect_timeout: Duration::from_secs(5),
        }),
    });
    let app = api::router(service);
    assert_eq!(
        request(&app, "GET", "/health", Value::Null).await?.1,
        json!({"status":"ok"})
    );
    let mut input = json!({"name":"Integration test", "database_type":"postgresql", "host":options.get_host(), "port":options.get_port(), "database_name":options.get_database(), "username":options.get_username(), "password":password, "ssl_mode":"disable", "read_only":true});
    let (status, created) = request(&app, "POST", "/connections", input.clone()).await?;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["has_password"], true);
    assert!(created.get("password").is_none());
    assert!(created.get("encrypted_password").is_none());
    let id = created["id"].as_str().ok_or("Missing id")?;
    let path = format!("/connections/{id}");
    assert_eq!(
        request(&app, "POST", &format!("{path}/test"), Value::Null)
            .await?
            .0,
        StatusCode::OK
    );
    input
        .as_object_mut()
        .ok_or("Not object")?
        .remove("password");
    input["name"] = json!("Updated");
    assert_eq!(
        request(&app, "PUT", &path, input).await?.1["name"],
        "Updated"
    );
    assert_eq!(
        request(&app, "POST", &format!("{path}/test"), Value::Null)
            .await?
            .0,
        StatusCode::OK
    );
    assert_eq!(
        request(&app, "DELETE", &path, Value::Null).await?.0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(&app, "GET", &path, Value::Null).await?.0,
        StatusCode::NOT_FOUND
    );
    pool.close().await;
    Ok(())
}
