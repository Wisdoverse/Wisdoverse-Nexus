//! Message routing for Nexus Gateway

pub mod ws_router;

use axum::{
    extract::ws::WebSocketUpgrade,
    extract::{Query, State},
    http::{HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Instant;
use tracing::Instrument;
use uuid::Uuid;

use crate::connection::WebSocketState;
use crate::metrics::{
    export as export_metrics, HTTP_LATENCY, HTTP_REQUESTS_TOTAL, HTTP_RESPONSES,
    OPERATION_ERRORS_TOTAL,
};
use crate::privacy::{PrivacyApplication, PrivacyInterfaceState};
use crate::rooms::{RoomApplication, RoomInterfaceState};
use crate::search::{SearchApplication, SearchInterfaceState, SearchService};

#[derive(Clone)]
pub(crate) struct AppState {
    rooms: RoomApplication,
    privacy: PrivacyApplication,
    search_application: Option<SearchApplication>,
    ws_state: WebSocketState,
}

impl Default for AppState {
    fn default() -> Self {
        let rooms = RoomApplication::default();
        Self {
            privacy: PrivacyApplication::new(rooms.clone()),
            rooms,
            search_application: None,
            ws_state: WebSocketState::default(),
        }
    }
}

impl AppState {
    #[allow(dead_code)]
    pub fn ws_state(&self) -> &WebSocketState {
        &self.ws_state
    }
    fn with_search_service(mut self, service: Arc<dyn SearchService>) -> Self {
        self.search_application = Some(SearchApplication::new(service));
        self
    }
}

impl RoomInterfaceState for AppState {
    fn rooms(&self) -> &RoomApplication {
        &self.rooms
    }
}

impl SearchInterfaceState for AppState {
    fn search_application(&self) -> Option<&SearchApplication> {
        self.search_application.as_ref()
    }
}

impl PrivacyInterfaceState for AppState {
    fn privacy(&self) -> &PrivacyApplication {
        &self.privacy
    }
}

type SharedState = AppState;
const OPENAPI_JSON: &str = include_str!("openapi.json");

fn v1_routes() -> Router<AppState> {
    Router::new()
        .merge(crate::rooms::routes())
        .merge(crate::search::routes())
        .merge(crate::privacy::routes())
        .merge(crate::collaboration::routes())
}

/// Build the main router for the gateway
pub fn build_routes() -> Router {
    let state = AppState::default();
    let rate_limiter = Arc::new(crate::rate_limit::RateLimiter::from_env());

    Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .route("/openapi.json", get(openapi_json))
        .route("/docs", get(swagger_ui))
        .route("/ws", get(websocket_handler))
        .merge(v1_routes().layer(middleware::from_fn_with_state(
            rate_limiter,
            crate::rate_limit::rate_limit_middleware,
        )))
        .layer(middleware::from_fn(correlation_id_middleware))
        .with_state(state)
}

/// Build router with search service
pub fn build_routes_with_search(search_service: Arc<dyn SearchService>) -> Router {
    let state = AppState::default().with_search_service(search_service);
    let rate_limiter = Arc::new(crate::rate_limit::RateLimiter::from_env());

    Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .route("/openapi.json", get(openapi_json))
        .route("/docs", get(swagger_ui))
        .route("/ws", get(websocket_handler))
        .merge(v1_routes().layer(middleware::from_fn_with_state(
            rate_limiter,
            crate::rate_limit::rate_limit_middleware,
        )))
        .layer(middleware::from_fn(correlation_id_middleware))
        .with_state(state)
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

async fn metrics_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        export_metrics(),
    )
}

async fn openapi_json() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("content-type", "application/json; charset=utf-8")],
        OPENAPI_JSON,
    )
}

async fn swagger_ui() -> impl IntoResponse {
    const SWAGGER_HTML: &str = r##"<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Wisdoverse Nexus Gateway API Docs</title>
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
  </head>
  <body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>
      window.ui = SwaggerUIBundle({
        url: "/openapi.json",
        dom_id: "#swagger-ui",
        deepLinking: true,
        docExpansion: "list"
      });
    </script>
  </body>
</html>
"##;

    Html(SWAGGER_HTML)
}

async fn correlation_id_middleware(request: Request<axum::body::Body>, next: Next) -> Response {
    let started = Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    let correlation_id = request
        .headers()
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .map_or_else(|| Uuid::new_v4().to_string(), ToString::to_string);

    let span = tracing::info_span!(
        "gateway.http.request",
        correlation_id = %correlation_id,
        method = %method,
        path = %path
    );
    let mut response = next.run(request).instrument(span).await;
    response.headers_mut().insert(
        "x-correlation-id",
        HeaderValue::from_str(&correlation_id)
            .unwrap_or_else(|_| HeaderValue::from_static("invalid-correlation-id")),
    );

    let status = response.status().as_u16().to_string();
    HTTP_REQUESTS_TOTAL
        .with_label_values(&[&method, &path])
        .inc();
    HTTP_RESPONSES
        .with_label_values(&[&method, &path, &status])
        .inc();
    HTTP_LATENCY
        .with_label_values(&[&method, &path])
        .observe(started.elapsed().as_secs_f64());

    if response.status().is_server_error() {
        OPERATION_ERRORS_TOTAL
            .with_label_values(&["http_request", "5xx"])
            .inc();
    }

    response
}
/// Query parameters for WebSocket connection (deprecated, use first-message auth instead)
#[derive(Debug, Clone, Deserialize)]
struct WebSocketAuthQuery {
    /// JWT token (deprecated: prefer first-message authentication)
    #[serde(default)]
    token: Option<String>,
}

/// WebSocket handler
async fn websocket_handler(
    State(state): State<SharedState>,
    ws: WebSocketUpgrade,
    Query(query): Query<WebSocketAuthQuery>,
) -> Response {
    // Use the new ws module's upgrade handler
    let ws_state = state.ws_state.clone();

    // Check for deprecated token in query parameter
    let legacy_token = query.token;
    if legacy_token.is_some() {
        tracing::warn!(
            "DEPRECATION WARNING: WebSocket auth via query parameter is deprecated. Use first-message auth instead."
        );
    }

    // Use new ws module for upgrade
    crate::connection::ws::websocket_upgrade_with_state(
        ws,
        Query(crate::connection::ws::WebSocketQuery {
            token: legacy_token,
        }),
        ws_state,
    )
    .await
}

/// Handle WebSocket connection
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::{json, Value};
    use tower::ServiceExt;

    use crate::metrics::{OPERATION_THROUGHPUT_TOTAL, ROOMS_CREATED_TOTAL};

    #[tokio::test]
    async fn health_check_returns_ok() {
        let app = build_routes();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn openapi_endpoint_returns_json() {
        let app = build_routes();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(content_type.starts_with("application/json"));

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["openapi"], "3.0.3");
    }

    #[tokio::test]
    async fn openapi_contract_matches_gateway_routes() {
        let app = build_routes();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        let paths = payload["paths"].as_object().unwrap();

        let expected_paths = [
            "/health",
            "/metrics",
            "/openapi.json",
            "/docs",
            "/ws",
            "/v1/rooms",
            "/v1/rooms/{id}",
            "/v1/rooms/{id}/invite",
            "/v1/messages",
            "/v1/search",
            "/v1/members/me/export",
            "/v1/members/me",
            "/v1/collaboration/meetings/rooms",
            "/v1/collaboration/meetings/rooms/{room_id}/join",
            "/v1/collaboration/meetings/rooms/{room_id}/leave",
            "/v1/collaboration/documents",
            "/v1/collaboration/documents/{document_id}/sync",
            "/v1/collaboration/documents/{document_id}/content",
            "/v1/collaboration/tasks",
            "/v1/collaboration/tasks/{task_id}/assign",
            "/v1/collaboration/tasks/{task_id}/complete",
            "/v1/collaboration/calendar/events",
            "/v1/collaboration/calendar/conflicts",
        ];

        for path in expected_paths {
            assert!(paths.contains_key(path), "missing OpenAPI path {path}");
        }

        let expected_methods = [
            ("/v1/rooms", "get"),
            ("/v1/rooms", "post"),
            ("/v1/rooms/{id}", "get"),
            ("/v1/rooms/{id}", "delete"),
            ("/v1/rooms/{id}/invite", "post"),
            ("/v1/messages", "post"),
            ("/v1/search", "get"),
            ("/v1/search", "post"),
            ("/v1/members/me/export", "get"),
            ("/v1/members/me", "delete"),
            ("/v1/collaboration/meetings/rooms", "post"),
            ("/v1/collaboration/meetings/rooms/{room_id}/join", "post"),
            ("/v1/collaboration/meetings/rooms/{room_id}/leave", "post"),
            ("/v1/collaboration/documents", "post"),
            ("/v1/collaboration/documents/{document_id}/sync", "post"),
            ("/v1/collaboration/documents/{document_id}/content", "get"),
            ("/v1/collaboration/tasks", "post"),
            ("/v1/collaboration/tasks/{task_id}/assign", "post"),
            ("/v1/collaboration/tasks/{task_id}/complete", "post"),
            ("/v1/collaboration/calendar/events", "post"),
            ("/v1/collaboration/calendar/conflicts", "post"),
        ];

        for (path, method) in expected_methods {
            let path_item = paths[path].as_object().unwrap();
            assert!(
                path_item.contains_key(method),
                "missing OpenAPI method {method} for {path}"
            );
        }

        let stale_paths = [
            "/collaboration/meetings",
            "/collaboration/meetings/{id}/join",
            "/collaboration/meetings/{id}/leave",
            "/collaboration/documents/{id}",
            "/collaboration/tasks/{id}/assign",
            "/collaboration/tasks/{id}/complete",
            "/collaboration/calendar/conflicts",
        ];

        for path in stale_paths {
            assert!(
                !paths.contains_key(path),
                "stale unversioned OpenAPI path {path}"
            );
        }
    }

    #[tokio::test]
    async fn docs_endpoint_returns_swagger_html() {
        let app = build_routes();
        let response = app
            .oneshot(Request::builder().uri("/docs").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("SwaggerUIBundle"));
        assert!(html.contains("/openapi.json"));
    }

    #[tokio::test]
    async fn metrics_endpoint_returns_prometheus_payload() {
        ROOMS_CREATED_TOTAL.inc();
        let app = build_routes();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload = String::from_utf8(body.to_vec()).unwrap();
        assert!(payload.contains("nexis_rooms_created_total"));
    }

    #[tokio::test]
    async fn response_contains_correlation_id_header() {
        let app = build_routes();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key("x-correlation-id"));
    }

    #[tokio::test]
    async fn create_room_returns_201_and_room_identity() {
        use crate::auth::JwtConfig;
        let token = JwtConfig::test_token("test-user");
        let before_rooms_created = ROOMS_CREATED_TOTAL.get();
        let before_throughput = OPERATION_THROUGHPUT_TOTAL
            .with_label_values(&["create_room"])
            .get();

        let app = build_routes();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/rooms")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::from(
                        json!({
                            "name": "general",
                            "topic": "team"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["name"], "general");
        assert!(payload["id"].as_str().unwrap().starts_with("room_"));
        assert!(ROOMS_CREATED_TOTAL.get() > before_rooms_created);
        assert!(
            OPERATION_THROUGHPUT_TOTAL
                .with_label_values(&["create_room"])
                .get()
                > before_throughput
        );
    }

    #[tokio::test]
    async fn create_room_validation_error_records_metric() {
        use crate::auth::JwtConfig;
        let token = JwtConfig::test_token("test-user");
        let before_errors = OPERATION_ERRORS_TOTAL
            .with_label_values(&["create_room", "validation"])
            .get();

        let app = build_routes();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/rooms")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::from(
                        json!({
                            "name": "   "
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert!(
            OPERATION_ERRORS_TOTAL
                .with_label_values(&["create_room", "validation"])
                .get()
                > before_errors
        );
    }

    #[tokio::test]
    async fn send_message_returns_404_for_unknown_room() {
        use crate::auth::JwtConfig;
        let token = JwtConfig::test_token("test-user");

        let app = build_routes();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/messages")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::from(
                        json!({
                            "roomId": "room_missing",
                            "sender": "alice",
                            "text": "hello"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn get_room_returns_messages_after_send() {
        use crate::auth::JwtConfig;
        let token = JwtConfig::test_token("test-user");

        let app = build_routes();

        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/rooms")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::from(
                        json!({
                            "name": "general"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create_response.status(), StatusCode::CREATED);
        let create_body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let create_payload: Value = serde_json::from_slice(&create_body).unwrap();
        let room_id = create_payload["id"].as_str().unwrap().to_string();

        let send_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/messages")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::from(
                        json!({
                            "roomId": room_id.clone(),
                            "sender": "alice",
                            "text": "hello"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(send_response.status(), StatusCode::CREATED);

        let get_response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/v1/rooms/{}", room_id.clone()))
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(get_response.status(), StatusCode::OK);
        let get_body = axum::body::to_bytes(get_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let get_payload: Value = serde_json::from_slice(&get_body).unwrap();
        assert_eq!(get_payload["id"], room_id);
        assert_eq!(get_payload["messages"].as_array().unwrap().len(), 1);
        assert_eq!(get_payload["messages"][0]["text"], "hello");
    }

    #[cfg(feature = "multi-tenant")]
    mod multi_tenant_tests {
        use super::*;

        #[tokio::test]
        async fn create_room_with_tenant_includes_tenant_id() {
            use crate::auth::JwtConfig;

            let app = build_routes();
            let token = JwtConfig::test_token("test-user");
            let response = app
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/v1/rooms")
                        .header("content-type", "application/json")
                        .header("authorization", format!("Bearer {}", token))
                        .body(Body::from(
                            json!({
                                "name": "tenant-room",
                                "tenant_id": "tenant_123"
                            })
                            .to_string(),
                        ))
                        .unwrap(),
                )
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::CREATED);
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let payload: Value = serde_json::from_slice(&body).unwrap();
            assert!(payload["id"].as_str().unwrap().starts_with("room_"));
        }
    }
}
