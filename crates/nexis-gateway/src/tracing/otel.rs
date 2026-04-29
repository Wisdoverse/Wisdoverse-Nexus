//! OpenTelemetry integration for distributed tracing
//!
//! Enable with `OTEL_ENABLED=true` environment variable.

use std::env;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

/// Initialize OpenTelemetry tracing
///
/// # Arguments
/// * `endpoint` - OTLP collector endpoint (e.g., "http://localhost:4317")
///
/// # Errors
/// Returns error if initialization fails
pub fn init_otel(endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry::global;
    use opentelemetry_otlp::WithExportConfig;

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint),
        )
        .install_simple()?;

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = Registry::default().with(telemetry);

    tracing::subscriber::set_global_default(subscriber)?;

    tracing::info!(
        endpoint = %endpoint,
        "OpenTelemetry tracing initialized"
    );

    Ok(())
}

/// Initialize OpenTelemetry from environment variables
///
/// Uses:
/// - `OTEL_EXPORTER_OTLP_ENDPOINT`: Collector endpoint (default: http://localhost:4317)
/// - `OTEL_SERVICE_NAME`: Service name (default: nexis-gateway)
pub fn init_from_env() -> Result<(), Box<dyn std::error::Error>> {
    if env::var("OTEL_ENABLED").is_err() {
        tracing::debug!("OpenTelemetry not enabled (set OTEL_ENABLED to enable)");
        return Ok(());
    }

    let endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    init_otel(&endpoint)
}

/// Shutdown OpenTelemetry tracer
///
/// Call this before exiting to flush pending spans.
pub fn shutdown() {
    use opentelemetry::global;
    global::shutdown_tracer_provider();
    tracing::debug!("OpenTelemetry tracer shutdown complete");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_is_safe() {
        // Should not panic even if never initialized
        shutdown();
    }
}
