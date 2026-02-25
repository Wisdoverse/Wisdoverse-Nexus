//! Nexus Gateway - Control Plane Entry Point
//!
//! This is the main entry point for the Nexus Control Plane gateway.

use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use nexis_gateway::{init_metrics, observability, router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing + export config
    observability::init_tracing()?;

    tracing::info!("Starting Nexus Gateway v{}", env!("CARGO_PKG_VERSION"));
    init_metrics();

    // Build router
    let app = Router::new()
        .merge(router::build_routes())
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any))
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr: SocketAddr = std::env::var("NEXIS_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".into())
        .parse()?;

    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
