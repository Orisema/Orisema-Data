mod api;
mod config;
mod crypto;
mod error;
mod http;
mod models;
mod policy;
mod postgres;
mod repository;
mod service;
#[cfg(test)]
mod tests;
use sqlx::ConnectOptions;
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    if run().await.is_err() {
        tracing::error!("Startup failed; check configuration and internal database availability");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::from_env()?;
    let credentials = crypto::Credentials::new(&config.encryption_key)?;
    let options = config
        .database_url
        .parse::<sqlx::postgres::PgConnectOptions>()?
        .disable_statement_logging();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(config.connect_timeout))
        .connect_with(options)
        .await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;
    let service = Arc::new(service::Service {
        repository: repository::Repository(pool),
        credentials,
        driver: Arc::new(postgres::PostgresDriver {
            connect_timeout: Duration::from_secs(config.connect_timeout),
            query_timeout: Duration::from_secs(config.query_timeout),
        }),
    });
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(config.cors_origin.parse::<axum::http::HeaderValue>()?)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
        ])
        .allow_headers([axum::http::header::CONTENT_TYPE]);
    let browser_policy = Arc::new(http::BrowserPolicy::new(
        &config.cors_origin,
        config.bind.port(),
    )?);
    let app = axum::Router::new()
        .nest("/api/v1", api::router(service, config.max_rows))
        .fallback_service(tower_http::services::ServeDir::new(
            std::env::var("RUSTDB_STATIC_DIR").unwrap_or_else(|_| "frontend/dist".into()),
        ))
        .layer(cors)
        .layer(tower::limit::ConcurrencyLimitLayer::new(16))
        .layer(axum::middleware::from_fn_with_state(
            browser_policy,
            http::protect,
        ));
    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    tracing::info!(address = %config.bind, "RustDB listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;
    Ok(())
}
