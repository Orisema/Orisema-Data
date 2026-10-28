use axum::{
    Json,
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

pub struct BrowserPolicy {
    pub hosts: Vec<String>,
    pub origins: Vec<String>,
}
impl BrowserPolicy {
    pub fn new(origin: &str, port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let uri: axum::http::Uri = origin.parse()?;
        if !matches!(uri.scheme_str(), Some("http" | "https"))
            || uri.authority().is_none()
            || uri.path() != "/"
            || uri.query().is_some()
            || origin.ends_with('/')
            || uri.authority().is_some_and(|a| a.as_str().contains('@'))
        {
            return Err("RUSTDB_CORS_ORIGIN must be an exact HTTP(S) origin".into());
        }
        let mut hosts = vec![
            format!("127.0.0.1:{port}"),
            format!("localhost:{port}"),
            format!("[::1]:{port}"),
        ];
        let mut origins: Vec<String> = hosts.iter().map(|host| format!("http://{host}")).collect();
        if let Some(authority) = uri.authority() {
            hosts.push(authority.to_string());
        }
        origins.push(origin.to_owned());
        Ok(Self { hosts, origins })
    }
}

pub async fn protect(
    State(policy): State<Arc<BrowserPolicy>>,
    request: Request,
    next: Next,
) -> Response {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();
    let origin = request.headers().get(header::ORIGIN);
    if !policy
        .hosts
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(host))
        || origin.is_some_and(|o| {
            !policy
                .origins
                .iter()
                .any(|allowed| o.as_bytes() == allowed.as_bytes())
        })
    {
        return failure(
            StatusCode::FORBIDDEN,
            "origin_rejected",
            "Host or browser origin is not allowed",
        );
    }
    let method = request.method().clone();
    let start = std::time::Instant::now();
    let mut response = next.run(request).await;
    if (response.status().is_client_error() || response.status().is_server_error())
        && response
            .headers()
            .get(header::CONTENT_TYPE)
            .is_none_or(|h| !h.as_bytes().starts_with(b"application/json"))
    {
        response = failure(
            response.status(),
            "request_rejected",
            "Request path, method or parameters are invalid",
        );
    }
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
        .headers_mut()
        .insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    response.headers_mut().insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    tracing::info!(method=%method,status=response.status().as_u16(),duration_ms=start.elapsed().as_millis(),"HTTP request");
    response
}
fn failure(status: StatusCode, code: &str, message: &str) -> Response {
    (
        status,
        Json(serde_json::json!({"error":{"code":code,"message":message}})),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;
    #[tokio::test]
    async fn rejects_rebinding_and_foreign_origins() -> Result<(), Box<dyn std::error::Error>> {
        let app = axum::Router::new()
            .route("/", axum::routing::get(|| async { "ok" }))
            .layer(axum::middleware::from_fn_with_state(
                Arc::new(BrowserPolicy::new("http://localhost:5173", 8080)?),
                protect,
            ));
        for (host, origin, status) in [
            (
                "evil.test:8080",
                "http://evil.test:8080",
                StatusCode::FORBIDDEN,
            ),
            ("localhost:8080", "http://evil.test", StatusCode::FORBIDDEN),
            ("localhost:8080", "http://localhost:5173", StatusCode::OK),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/")
                        .header("host", host)
                        .header("origin", origin)
                        .body(axum::body::Body::empty())?,
                )
                .await?;
            assert_eq!(response.status(), status);
        }
        assert!(BrowserPolicy::new("*", 8080).is_err());
        Ok(())
    }
}
