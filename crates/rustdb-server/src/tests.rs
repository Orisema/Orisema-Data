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
            query_timeout: Duration::from_secs(1),
        }),
        active_queries: crate::service::ActiveQueries::default(),
    });
    let app = api::router(service.clone(), 1000);
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
    let stored: String =
        sqlx::query_scalar("SELECT encrypted_password FROM connections WHERE id=$1")
            .bind(id.parse::<uuid::Uuid>()?)
            .fetch_one(&pool)
            .await?;
    assert!(stored.starts_with("v1:"));
    assert_ne!(stored, password);
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
    let query_path = format!("{path}/query");

    let query_id = uuid::Uuid::new_v4();
    let slow_query = request(
        &app,
        "POST",
        &query_path,
        json!({"sql":"SELECT pg_sleep(10)", "query_id": query_id}),
    );
    let cancellation = async {
        tokio::time::sleep(Duration::from_millis(250)).await;
        request(&app, "DELETE", &format!("/queries/{query_id}"), Value::Null).await
    };
    let (slow_result, cancel_result) = tokio::join!(slow_query, cancellation);
    assert_eq!(cancel_result?.0, StatusCode::NO_CONTENT);
    assert_eq!(slow_result?.0, StatusCode::CONFLICT);
    let mut still_running = 1_i64;
    for _ in 0..20 {
        still_running = sqlx::query_scalar(
            "SELECT count(*) FROM pg_stat_activity WHERE application_name='rustdb' AND state='active' AND query LIKE '%pg_sleep(10)%'",
        )
        .fetch_one(&pool)
        .await?;
        if still_running == 0 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(still_running, 0, "cancelled backend query remained active");
    assert_eq!(
        request(&app, "GET", &format!("{path}/schemas"), Value::Null)
            .await?
            .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &app,
            "GET",
            &format!("{path}/schemas/public/tables"),
            Value::Null
        )
        .await?
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &app,
            "GET",
            &format!("{path}/schemas/public/tables/connections/columns"),
            Value::Null
        )
        .await?
        .0,
        StatusCode::OK
    );
    let (status,result) = request(&app,"POST",&query_path,json!({"sql":"SELECT 1::int4 AS id, NULL::text AS empty, true AS flag, 9007199254740993::bigint AS exact, 12.50::numeric AS amount"})).await?;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(
        result["rows"][0],
        json!([1, null, true, "9007199254740993", "12.50"])
    );
    let (_, result) = request(
        &app,
        "POST",
        &query_path,
        json!({"sql":"SELECT * FROM generate_series(1,100)","max_rows":3}),
    )
    .await?;
    assert_eq!(result["row_count"], 3);
    assert_eq!(result["truncated"], true);
    let (_, result) = request(
        &app,
        "POST",
        &query_path,
        json!({"sql":"SELECT 1 AS empty WHERE false"}),
    )
    .await?;
    assert_eq!(result["row_count"], 0);
    assert_eq!(result["columns"][0]["name"], "empty");
    for sql in [
        "DELETE FROM connections",
        "SELECT 1; SELECT 2",
        "SET transaction_read_only=off",
        "WITH deleted AS (DELETE FROM connections RETURNING *) SELECT * FROM deleted",
    ] {
        assert_eq!(
            request(&app, "POST", &query_path, json!({"sql":sql}))
                .await?
                .0,
            StatusCode::UNPROCESSABLE_ENTITY,
            "{sql}"
        );
    }
    assert_eq!(
        request(
            &app,
            "POST",
            &query_path,
            json!({"sql":"SELECT pg_sleep(3)"})
        )
        .await?
        .0,
        StatusCode::GATEWAY_TIMEOUT
    );
    assert_eq!(
        request(
            &app,
            "POST",
            &query_path,
            json!({"sql":"SELECT 1","max_rows":1001})
        )
        .await?
        .0,
        StatusCode::BAD_REQUEST
    );
    let mut target = service.target(id.parse()?).await?;
    let table = format!("rustdb_test_{}", uuid::Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE TABLE {table} (id integer, value text)"))
        .execute(&pool)
        .await?;
    assert!(
        service
            .driver
            .execute_query(
                &target,
                &format!("INSERT INTO {table} VALUES (1,'blocked')"),
                10
            )
            .await
            .is_err()
    );
    target.read_only = false;
    let inserted = service
        .driver
        .execute_query(
            &target,
            &format!("INSERT INTO {table} SELECT n,'ok' FROM generate_series(1,5) n RETURNING id"),
            2,
        )
        .await?;
    assert_eq!(inserted.affected_rows, 5);
    assert_eq!(inserted.row_count, 2);
    assert!(inserted.truncated);
    let count: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {table}"))
        .fetch_one(&pool)
        .await?;
    assert_eq!(count, 5);
    let failure = service
        .driver
        .execute_query(
            &target,
            &format!("UPDATE {table} SET value=repeat('x',2000000) RETURNING value"),
            10,
        )
        .await;
    assert!(matches!(
        failure,
        Err(orisema_data_core::driver::DriverError::ResultTooLarge)
    ));
    let value: String = sqlx::query_scalar(&format!("SELECT value FROM {table} LIMIT 1"))
        .fetch_one(&pool)
        .await?;
    assert_eq!(value, "ok", "Oversize write results must roll back");
    let typed=service.driver.execute_query(&target,"SELECT ARRAY[1,2] AS a, '{}'::jsonb AS j, '2026-01-01'::date AS d, 1 AS repeated, 2 AS repeated",10).await?;
    assert_eq!(
        typed.rows[0],
        vec![
            json!("{1,2}"),
            json!("{}"),
            json!("2026-01-01"),
            json!(1),
            json!(2)
        ]
    );
    assert!(
        service
            .driver
            .list_columns(&target, "public' OR '1'='1", "anything")
            .await?
            .is_empty()
    );
    sqlx::query(&format!("DROP TABLE {table}"))
        .execute(&pool)
        .await?;
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
