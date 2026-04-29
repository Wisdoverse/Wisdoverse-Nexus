//! Request logging middleware.
//!
//! Provides structured request/response logging with:
//! - Automatic `X-Request-ID` generation (UUID v4)
//! - Request method, path, status code, duration
//! - Client IP and User-Agent
//! - Optional user_id from auth context
//! - Tracing spans for nested log correlation
//! - Prometheus metrics integration

use std::time::Instant;

use axum::{
    body::Body,
    extract::Request,
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::{info, warn};
use uuid::Uuid;

use crate::metrics::{HTTP_LATENCY, HTTP_RESPONSES};

// ---------------------------------------------------------------------------
// RequestLog
// ---------------------------------------------------------------------------

/// Structured representation of a completed HTTP request for logging.
#[derive(Debug, Clone, Serialize)]
pub struct RequestLog {
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration_ms: f64,
    pub client_ip: String,
    pub user_agent: String,
    pub request_id: String,
    pub user_id: Option<String>,
}

impl std::fmt::Display for RequestLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} {} {} {}ms ip={} uid={:?}",
            self.request_id,
            self.method,
            self.path,
            self.status,
            self.duration_ms,
            self.client_ip,
            self.user_id,
        )
    }
}

// ---------------------------------------------------------------------------
// Middleware
// ---------------------------------------------------------------------------

/// Axum middleware that logs every HTTP request with structured output and
/// records Prometheus metrics.
pub async fn request_logging(mut req: Request, next: Next) -> Response {
    let start = Instant::now();

    // --- Request-ID --------------------------------------------------------
    let request_id = req
        .headers()
        .get(header::X_REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().hyphenated().to_string());

    if req.headers().get(header::X_REQUEST_ID).is_none() {
        if let Ok(val) = HeaderValue::from_str(&request_id) {
            req.headers_mut().insert(header::X_REQUEST_ID, val);
        }
    }

    // --- Extract metadata ---------------------------------------------------
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let client_ip = extract_client_ip(&req);
    let user_agent = req
        .headers()
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-")
        .to_string();

    // --- Tracing span -------------------------------------------------------
    let span = tracing::info_span!(
        "http_request",
        request_id = %request_id,
        method = %method,
        path = %path,
        client_ip = %client_ip,
    );

    let log = async {
        let response = next.run(req).await;
        let status = response.status().as_u16();

        // Extract user_id from response extensions (set by auth middleware).
        let user_id = response.extensions().get::<UserId>().map(|u| u.0.clone());

        // --- Prometheus metrics ---------------------------------------------
        HTTP_RESPONSES
            .with_label_values(&[&method, &path, &status.to_string()])
            .inc();
        HTTP_LATENCY
            .with_label_values(&[&method, &path])
            .observe(start.elapsed().as_secs_f64());

        // --- Structured log -------------------------------------------------
        let log_entry = RequestLog {
            timestamp: Utc::now(),
            method: method.clone(),
            path: path.clone(),
            status,
            duration_ms: start.elapsed().as_secs_f64() * 1000.0,
            client_ip: client_ip.clone(),
            user_agent: user_agent.clone(),
            request_id: request_id.clone(),
            user_id,
        };

        let json = serde_json::to_string(&log_entry).unwrap_or_default();
        match status {
            400..=499 => warn!(log = %json, "client error"),
            500..=599 => warn!(log = %json, "server error"),
            _ => info!(log = %json, "request completed"),
        }

        info!(%log_entry);

        response
    }
    .instrument(span)
    .await;

    log
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the best-guess client IP from the request.
fn extract_client_ip(req: &Request) -> String {
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(val) = forwarded.to_str() {
            if let Some(first) = val.split(',').next() {
                return first.trim().to_string();
            }
        }
    }
    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(val) = real_ip.to_str() {
            return val.to_string();
        }
    }
    req.extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|addr| addr.0.ip().to_string())
        .unwrap_or_else(|| "-".to_string())
}

/// Extension type that auth middleware can inject so logging picks up user_id.
#[derive(Debug, Clone)]
pub struct UserId(pub String);

/// Attach a user_id to response extensions (call from auth middleware).
pub fn attach_user_id(response: &mut Response, user_id: String) {
    response.extensions_mut().insert(UserId(user_id));
}

// ---------------------------------------------------------------------------
// Layer
// ---------------------------------------------------------------------------

/// Tower layer wrapping [`request_logging`] for use in `ServiceBuilder`.
#[derive(Clone)]
pub struct RequestLoggingLayer;

impl<S> tower::Layer<S> for RequestLoggingLayer {
    type Service = tower::ServiceBuilder<S>;

    fn layer(&self, service: S) -> Self::Service {
        tower::ServiceBuilder::new()
            .layer(axum::middleware::from_fn(request_logging))
            .service(service)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use tower::ServiceExt;

    fn test_app() -> Router {
        Router::new()
            .route("/health", get(|| async { StatusCode::OK }))
            .route(
                "/error",
                get(|| async { StatusCode::INTERNAL_SERVER_ERROR }),
            )
            .layer(axum::middleware::from_fn(request_logging))
    }

    #[tokio::test]
    async fn request_id_is_generated() {
        let app = test_app();
        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert!(res.headers().contains_key(header::X_REQUEST_ID));
    }

    #[tokio::test]
    async fn request_id_is_preserved() {
        let app = test_app();
        let req = Request::builder()
            .uri("/health")
            .header(header::X_REQUEST_ID, "test-id-123")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(
            res.headers().get(header::X_REQUEST_ID).unwrap(),
            "test-id-123"
        );
    }
}
