//! Request ID Middleware for Nexis Gateway
//!
//! Generates unique request IDs and propagates them through the request lifecycle.
//! Uses UUID v7 for time-sortable IDs.

use axum::{
    body::Body,
    extract::FromRequestParts,
    http::{header, request::Parts, Request, Response},
    middleware::Next,
};
use std::sync::Arc;
use uuid::Uuid;

/// Header name for request ID
pub const X_REQUEST_ID: &str = "X-Request-ID";

/// Request ID wrapper for extracting from request parts
#[derive(Debug, Clone)]
pub struct RequestId(pub Arc<str>);

impl std::ops::Deref for RequestId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<S> FromRequestParts<S> for RequestId
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Try to get from extensions first (set by middleware)
        if let Some(id) = parts.extensions.get::<RequestId>() {
            return Ok(id.clone());
        }

        // Fallback: generate a new one
        Ok(RequestId(generate_request_id()))
    }
}

/// Generate a time-sortable UUID v7 request ID
fn generate_request_id() -> Arc<str> {
    // Use UUID v7 for time-sortable IDs
    let id = Uuid::now_v7();
    Arc::from(id.hyphenated().to_string())
}

/// Request ID middleware
///
/// This middleware:
/// 1. Checks for existing X-Request-ID header
/// 2. Generates a new UUID v7 if not present
/// 3. Adds the ID to request extensions for downstream access
/// 4. Adds the ID to response headers
/// 5. Creates a tracing span with the request ID
pub async fn request_id_middleware(request: Request<Body>, next: Next) -> Response<Body> {
    // Get or generate request ID
    let request_id = request
        .headers()
        .get(X_REQUEST_ID)
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(|s| Arc::from(s.to_string()))
        .unwrap_or_else(generate_request_id);

    // Create tracing span with request ID
    let span = tracing::info_span!(
        "request",
        request_id = %request_id,
    );

    // Add to request extensions
    let mut request = request;
    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    // Run the request in the span
    let response = async move { next.run(request).await }
        .instrument(span)
        .await;

    // Add to response headers
    let mut response = response;
    response.headers_mut().insert(
        header::HeaderName::try_from(X_REQUEST_ID).unwrap(),
        header::HeaderValue::from_str(&request_id).unwrap(),
    );

    response
}

/// Tracing instrumentation helper
use tracing::Instrument;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        extract::State,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn generates_request_id_if_missing() {
        let app = Router::new()
            .route(
                "/test",
                get(|req_id: RequestId| async move { req_id.to_string() }),
            )
            .layer(axum::middleware::from_fn(request_id_middleware));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key(X_REQUEST_ID));

        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let id_str = String::from_utf8_lossy(&body);

        // Should be a valid UUID format
        assert!(Uuid::parse_str(&id_str).is_ok());
    }

    #[tokio::test]
    async fn propagates_existing_request_id() {
        let existing_id = "01234567-89ab-7def-8123-456789abcdef";

        let app = Router::new()
            .route(
                "/test",
                get(|req_id: RequestId| async move { req_id.to_string() }),
            )
            .layer(axum::middleware::from_fn(request_id_middleware));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header(X_REQUEST_ID, existing_id)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let response_id = response
            .headers()
            .get(X_REQUEST_ID)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(response_id, existing_id);
    }

    #[test]
    fn generates_valid_uuid_v7() {
        let id = generate_request_id();
        let parsed = Uuid::parse_str(&id).expect("Should be valid UUID");

        // UUID v7 has version 7
        assert_eq!(parsed.get_version_num(), 7);
    }
}
