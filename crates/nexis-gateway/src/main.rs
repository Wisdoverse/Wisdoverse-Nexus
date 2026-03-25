//! Nexus Gateway - Control Plane Entry Point
//!
//! This is the main entry point for the Nexus Control Plane gateway.

use axum::http::header::{HeaderName, ACCEPT, AUTHORIZATION, CONTENT_TYPE, HOST};
use axum::http::{HeaderValue, Method, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Router;
use std::net::SocketAddr;
use std::sync::OnceLock;
use std::time::Duration;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::trace::TraceLayer;

use nexis_gateway::{init_metrics, observability, router};

#[derive(Debug)]
struct GatewaySecurityConfig {
    https_redirect_enabled: bool,
    hsts_enabled: bool,
    csp_policy: String,
}

fn env_flag(name: &str, default: bool) -> bool {
    std::env::var(name)
        .ok()
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(default)
}

fn security_config() -> &'static GatewaySecurityConfig {
    static CONFIG: OnceLock<GatewaySecurityConfig> = OnceLock::new();
    CONFIG.get_or_init(|| GatewaySecurityConfig {
        https_redirect_enabled: env_flag("NEXIS_HTTPS_REDIRECT_ENABLED", false),
        hsts_enabled: env_flag("NEXIS_HSTS_ENABLED", true),
        csp_policy: std::env::var("NEXIS_CSP_POLICY").unwrap_or_else(|_| {
            "default-src 'self'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'; object-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self' https: wss:".to_string()
        }),
    })
}

fn configured_origins() -> Vec<HeaderValue> {
    let raw = std::env::var("NEXIS_CORS_ALLOW_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:5173,http://127.0.0.1:5173".to_string());

    raw.split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .filter_map(|origin| match HeaderValue::from_str(origin) {
            Ok(value) => Some(value),
            Err(_) => {
                tracing::warn!("Skipping invalid CORS origin '{}'", origin);
                None
            }
        })
        .collect()
}

fn build_cors_layer() -> CorsLayer {
    let mut cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            AUTHORIZATION,
            CONTENT_TYPE,
            ACCEPT,
            HeaderName::from_static("x-correlation-id"),
        ])
        .expose_headers([axum::http::header::HeaderName::from_static(
            "x-correlation-id",
        )])
        .max_age(Duration::from_secs(3600));

    let origins = configured_origins();
    if origins.iter().any(|value| value.as_bytes() == b"*") {
        cors = cors.allow_origin(Any);
    } else if !origins.is_empty() {
        cors = cors.allow_origin(AllowOrigin::list(origins));
    } else {
        tracing::warn!("No valid CORS origins found; defaulting to localhost dev origins");
        cors = cors.allow_origin(AllowOrigin::list([
            HeaderValue::from_static("http://localhost:5173"),
            HeaderValue::from_static("http://127.0.0.1:5173"),
        ]));
    }

    if env_flag("NEXIS_CORS_ALLOW_CREDENTIALS", true) {
        cors = cors.allow_credentials(true);
    }

    cors
}

fn is_https(request: &Request<axum::body::Body>) -> bool {
    if request
        .headers()
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .is_some_and(|proto| proto.eq_ignore_ascii_case("https"))
    {
        return true;
    }

    request
        .uri()
        .scheme_str()
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("https"))
}

async fn enforce_https_middleware(request: Request<axum::body::Body>, next: Next) -> Response {
    let config = security_config();
    if !config.https_redirect_enabled || is_https(&request) {
        return next.run(request).await;
    }

    let Some(host) = request
        .headers()
        .get(HOST)
        .and_then(|value| value.to_str().ok())
    else {
        return (StatusCode::BAD_REQUEST, "missing host header").into_response();
    };

    let location = format!("https://{}{}", host, request.uri());
    Redirect::permanent(&location).into_response()
}

async fn security_headers_middleware(request: Request<axum::body::Body>, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    let config = security_config();

    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert(
        "cross-origin-opener-policy",
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        "cross-origin-resource-policy",
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        "referrer-policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );

    if let Ok(value) = HeaderValue::from_str(&config.csp_policy) {
        headers.insert("content-security-policy", value);
    } else {
        tracing::warn!("Invalid NEXIS_CSP_POLICY value. CSP header skipped.");
    }

    if config.hsts_enabled {
        headers.insert(
            "strict-transport-security",
            HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
        );
    }

    response
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing + export config
    observability::init_logging()?;

    tracing::info!("Starting Nexus Gateway v{}", env!("CARGO_PKG_VERSION"));
    init_metrics();

    // Build router
    let app = Router::new()
        .merge(router::build_routes())
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(middleware::from_fn(enforce_https_middleware))
        .layer(build_cors_layer())
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr: SocketAddr = std::env::var("NEXIS_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".into())
        .parse()?;

    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Graceful shutdown setup
    let shutdown = async {
        use tokio::signal;

        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => tracing::info!("Received Ctrl+C"),
            _ = terminate => tracing::info!("Received SIGTERM"),
        }

        tracing::info!("Shutdown signal received, stopping gracefully...");
    };

    // Run server with graceful shutdown
    tokio::select! {
        _ = axum::serve(listener, app) => {},
        _ = shutdown => {},
    }

    tracing::info!("Server stopped");
    Ok(())
}
