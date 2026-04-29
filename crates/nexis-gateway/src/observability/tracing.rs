//! Distributed tracing integration with OpenTelemetry
//!
//! This module provides OpenTelemetry (OTLP) integration for distributed tracing,
//! enabling end-to-end request tracing across services.
//!
//! # Feature Flag
//!
//! This module is only available when the `otel` feature is enabled.
//!
//! # Configuration
//!
//! Environment variables:
//! - `NEXIS_OTEL_EXPORTER`: Exporter type (`otlp`, `stdout`, or `none`)
//! - `NEXIS_OTEL_EXPORT_ENDPOINT`: OTLP collector endpoint (e.g., `http://localhost:4317`)
//! - `OTEL_SERVICE_NAME`: Service name for spans (default: `nexis-gateway`)
//!
//! # Span Creation and Propagation
//!
//! Use `tracing` macros to create spans:
//!
//! ```rust,ignore
//! #[tracing::instrument(
//!     name = "gateway.handle_request",
//!     skip(request),
//!     fields(
//!         request_id = %request.id,
//!         method = %request.method,
//!         path = %request.path
//!     )
//! )]
//! async fn handle_request(request: Request) -> Response {
//!     // Your handler code
//! }
//! ```

use anyhow::{anyhow, Result};
use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Layer};

/// Tracing exporter configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExporterType {
    /// Export to OTLP collector (e.g., Jaeger, Tempo, or other OTLP receivers)
    Otlp,
    /// Print traces to stdout (useful for debugging)
    Stdout,
    /// No export (tracing disabled)
    None,
}

impl std::str::FromStr for ExporterType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "otlp" => Ok(Self::Otlp),
            "stdout" => Ok(Self::Stdout),
            "none" => Ok(Self::None),
            other => Err(format!(
                "invalid exporter '{}', expected: otlp, stdout, or none",
                other
            )),
        }
    }
}

/// Configuration for distributed tracing.
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Type of exporter to use.
    pub exporter: ExporterType,
    /// OTLP collector endpoint (only used when exporter is `Otlp`).
    pub endpoint: Option<String>,
    /// Service name for spans.
    pub service_name: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            exporter: ExporterType::Stdout,
            endpoint: None,
            service_name: "nexis-gateway".to_string(),
        }
    }
}

impl TracingConfig {
    /// Create configuration from environment variables.
    ///
    /// # Environment Variables
    ///
    /// - `NEXIS_OTEL_EXPORTER`: Exporter type (`otlp`, `stdout`, `none`), default: `stdout`
    /// - `NEXIS_OTEL_EXPORT_ENDPOINT`: OTLP endpoint (required if exporter is `otlp`)
    /// - `OTEL_SERVICE_NAME`: Service name, default: `nexis-gateway`
    pub fn from_env() -> Result<Self> {
        let exporter = std::env::var("NEXIS_OTEL_EXPORTER")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(ExporterType::Stdout);

        let endpoint = std::env::var("NEXIS_OTEL_EXPORT_ENDPOINT").ok();

        // Validate that OTLP has an endpoint
        if exporter == ExporterType::Otlp && endpoint.is_none() {
            return Err(anyhow!(
                "NEXIS_OTEL_EXPORTER=otlp requires NEXIS_OTEL_EXPORT_ENDPOINT to be set"
            ));
        }

        let service_name =
            std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "nexis-gateway".to_string());

        Ok(Self {
            exporter,
            endpoint,
            service_name,
        })
    }
}

/// Initialize distributed tracing with the given configuration.
///
/// This sets up OpenTelemetry tracing with the configured exporter.
///
/// # Errors
///
/// Returns an error if:
/// - The OTLP exporter fails to initialize
/// - The global subscriber is already set
///
/// # Example
///
/// ```rust,no_run
/// use nexis_gateway::observability::tracing::{init_tracing_with_config, TracingConfig, ExporterType};
///
/// let config = TracingConfig {
///     exporter: ExporterType::Otlp,
///     endpoint: Some("http://localhost:4317".to_string()),
///     service_name: "nexis-gateway".to_string(),
/// };
///
/// init_tracing_with_config(config).expect("Failed to initialize tracing");
/// ```
pub fn init_tracing_with_config(config: TracingConfig) -> Result<()> {
    match config.exporter {
        ExporterType::None => {
            tracing::info!(
                target: "observability.tracing",
                exporter = "none",
                "Distributed tracing disabled"
            );
            Ok(())
        }
        ExporterType::Stdout => {
            // For stdout, we don't need OpenTelemetry layer - regular logging handles it
            tracing::info!(
                target: "observability.tracing",
                exporter = "stdout",
                "Distributed tracing using stdout (via logging layer)"
            );
            Ok(())
        }
        ExporterType::Otlp => {
            let endpoint = config
                .endpoint
                .as_ref()
                .ok_or_else(|| anyhow!("OTLP exporter requires an endpoint"))?;

            init_otlp_exporter(endpoint, &config.service_name)?;

            tracing::info!(
                target: "observability.tracing",
                exporter = "otlp",
                endpoint = %endpoint,
                service_name = %config.service_name,
                "OpenTelemetry tracing initialized"
            );

            Ok(())
        }
    }
}

/// Initialize OTLP exporter with the given endpoint.
///
/// This is an internal function used by `init_tracing_with_config`.
fn init_otlp_exporter(endpoint: &str, _service_name: &str) -> Result<()> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint),
        )
        .install_simple()
        .map_err(|e| anyhow!("failed to install OTLP tracer: {}", e))?;

    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Add the OpenTelemetry layer to the existing subscriber
    // Note: This assumes logging is already initialized.
    // If you need to initialize both together, use `init_combined` instead.
    let env_filter = EnvFilter::new(
        std::env::var("NEXIS_LOG_LEVEL")
            .or_else(|_| std::env::var("RUST_LOG"))
            .unwrap_or_else(|_| "nexis_gateway=info,tower_http=info".into()),
    );

    let json_logs = std::env::var("NEXIS_LOG_FORMAT")
        .map(|v| v.eq_ignore_ascii_case("json"))
        .unwrap_or(true);

    let with_spans = std::env::var("NEXIS_LOG_SPANS")
        .map(|v| !v.eq_ignore_ascii_case("false"))
        .unwrap_or(true);

    let fmt_layer = if json_logs {
        tracing_subscriber::fmt::layer()
            .json()
            .with_current_span(with_spans)
            .with_span_list(with_spans)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .boxed()
    } else {
        tracing_subscriber::fmt::layer().boxed()
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(telemetry_layer)
        .with(fmt_layer)
        .try_init()
        .map_err(|e| anyhow!("failed to initialize tracing subscriber: {}", e))?;

    Ok(())
}

/// Initialize distributed tracing with environment-based configuration.
///
/// This is the main entry point for tracing initialization.
/// Configuration is read from environment variables.
///
/// # Environment Variables
///
/// See [`TracingConfig::from_env`] for supported environment variables.
///
/// # Errors
///
/// Returns an error if configuration is invalid or initialization fails.
///
/// # Example
///
/// ```rust,no_run
/// // Set via environment:
/// // NEXIS_OTEL_EXPORTER=otlp
/// // NEXIS_OTEL_EXPORT_ENDPOINT=http://localhost:4317
///
/// use nexis_gateway::observability::tracing::init_tracing;
///
/// init_tracing().expect("Failed to initialize tracing");
/// ```
pub fn init_tracing() -> Result<()> {
    let config = TracingConfig::from_env()?;
    init_tracing_with_config(config)
}

/// Shutdown the OpenTelemetry tracer gracefully.
///
/// Call this during application shutdown to ensure all pending spans
/// are flushed to the exporter.
///
/// # Example
///
/// ```rust,no_run
/// use nexis_gateway::observability::tracing::shutdown_tracing;
///
/// // During shutdown:
/// shutdown_tracing();
/// ```
pub fn shutdown_tracing() {
    global::shutdown_tracer_provider();
    tracing::debug!(target: "observability.tracing", "OpenTelemetry tracer shutdown complete");
}

/// Create a tracing span with request ID for correlation.
///
/// This is a convenience function for creating spans that include
/// a request ID for log correlation.
///
/// # Example
///
/// ```rust,ignore
/// use nexis_gateway::observability::tracing::request_span;
/// use uuid::Uuid;
///
/// let request_id = Uuid::new_v4().to_string();
/// let span = request_span(&request_id, "handle_http_request");
/// let _enter = span.enter();
/// // Your code here
/// ```
pub fn request_span(request_id: &str, name: &'static str) -> tracing::Span {
    tracing::info_span!(
        name,
        request_id = %request_id,
        otel.name = %name
    )
}

/// Create a tracing span for HTTP requests with standard fields.
///
/// # Example
///
/// ```rust,ignore
/// use nexis_gateway::observability::tracing::http_request_span;
///
/// let span = http_request_span("req-123", "GET", "/api/users");
/// let _enter = span.enter();
/// // Your handler code here
/// ```
pub fn http_request_span(request_id: &str, method: &str, path: &str) -> tracing::Span {
    tracing::info_span!(
        "http.request",
        request_id = %request_id,
        http.method = %method,
        http.route = %path,
        otel.name = %"http.request"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exporter_type_parsing() {
        assert_eq!(ExporterType::from_str("otlp").unwrap(), ExporterType::Otlp);
        assert_eq!(ExporterType::from_str("OTLP").unwrap(), ExporterType::Otlp);
        assert_eq!(
            ExporterType::from_str("stdout").unwrap(),
            ExporterType::Stdout
        );
        assert_eq!(
            ExporterType::from_str("STDOUT").unwrap(),
            ExporterType::Stdout
        );
        assert_eq!(ExporterType::from_str("none").unwrap(), ExporterType::None);
        assert_eq!(ExporterType::from_str("NONE").unwrap(), ExporterType::None);
        assert!(ExporterType::from_str("invalid").is_err());
    }

    #[test]
    fn tracing_config_defaults() {
        let config = TracingConfig::default();
        assert_eq!(config.exporter, ExporterType::Stdout);
        assert_eq!(config.service_name, "nexis-gateway");
        assert!(config.endpoint.is_none());
    }

    #[test]
    fn tracing_config_from_env_without_endpoint() {
        std::env::remove_var("NEXIS_OTEL_EXPORTER");
        std::env::remove_var("NEXIS_OTEL_EXPORT_ENDPOINT");

        let config = TracingConfig::from_env().unwrap();
        assert_eq!(config.exporter, ExporterType::Stdout);
    }

    #[test]
    fn tracing_config_otlp_requires_endpoint() {
        std::env::set_var("NEXIS_OTEL_EXPORTER", "otlp");
        std::env::remove_var("NEXIS_OTEL_EXPORT_ENDPOINT");

        let result = TracingConfig::from_env();
        assert!(result.is_err());

        std::env::remove_var("NEXIS_OTEL_EXPORTER");
    }

    #[test]
    fn shutdown_is_safe() {
        // Should not panic even if never initialized
        shutdown_tracing();
    }
}
