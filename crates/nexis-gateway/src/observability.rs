//! Observability bootstrap for logging and tracing.

use anyhow::{anyhow, Result};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[cfg(feature = "otel")]
use tracing_subscriber::Layer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceExportConfig {
    pub exporter: String,
    pub endpoint: Option<String>,
}

impl TraceExportConfig {
    pub fn from_env() -> Result<Self> {
        let exporter =
            std::env::var("NEXIS_OTEL_EXPORTER").unwrap_or_else(|_| "stdout".to_string());
        let endpoint = std::env::var("NEXIS_OTEL_EXPORT_ENDPOINT").ok();
        match exporter.as_str() {
            "stdout" | "none" | "otlp" => Ok(Self { exporter, endpoint }),
            other => Err(anyhow!(
                "unsupported NEXIS_OTEL_EXPORTER '{}', expected one of: stdout, none, otlp",
                other
            )),
        }
    }
}

pub fn init_tracing() -> Result<()> {
    let trace_export = TraceExportConfig::from_env()?;
    let env_filter = EnvFilter::new(
        std::env::var("RUST_LOG").unwrap_or_else(|_| "nexis_gateway=info,tower_http=info".into()),
    );

    let json_logs = std::env::var("NEXIS_LOG_FORMAT")
        .map(|v| v.eq_ignore_ascii_case("json"))
        .unwrap_or(true);
    let with_span_list = std::env::var("NEXIS_LOG_SPANS")
        .map(|v| !v.eq_ignore_ascii_case("false"))
        .unwrap_or(true);

    // If OTLP exporter is configured, initialize OpenTelemetry
    #[cfg(feature = "otel")]
    if trace_export.exporter == "otlp" {
        if let Some(endpoint) = &trace_export.endpoint {
            init_otel_tracing(endpoint, env_filter, json_logs, with_span_list)?;
            return Ok(());
        } else {
            tracing::warn!(
                target: "observability",
                "NEXIS_OTEL_EXPORTER=otlp is set without NEXIS_OTEL_EXPORT_ENDPOINT, falling back to stdout"
            );
        }
    }

    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(with_span_list)
        .with_span_list(with_span_list)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true);

    if json_logs {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    tracing::info!(
        target: "observability",
        otel_exporter = %trace_export.exporter,
        otel_endpoint = ?trace_export.endpoint,
        "tracing initialized"
    );

    Ok(())
}

/// Initialize OpenTelemetry tracing with OTLP exporter.
#[cfg(feature = "otel")]
fn init_otel_tracing(
    endpoint: &str,
    env_filter: EnvFilter,
    json_logs: bool,
    with_span_list: bool,
) -> Result<()> {
    use opentelemetry::global;
    use opentelemetry_otlp::WithExportConfig;

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint),
        )
        .install_simple()
        .map_err(|e| anyhow!("failed to install OTLP tracer: {}", e))?;

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let fmt_layer = if json_logs {
        tracing_subscriber::fmt::layer()
            .json()
            .with_current_span(with_span_list)
            .with_span_list(with_span_list)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .boxed()
    } else {
        tracing_subscriber::fmt::layer().boxed()
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(telemetry)
        .with(fmt_layer)
        .init();

    tracing::info!(
        target: "observability",
        otel_exporter = "otlp",
        otel_endpoint = %endpoint,
        "OpenTelemetry tracing initialized"
    );

    Ok(())
}

/// Shutdown the OpenTelemetry tracer provider.
///
/// Call this during graceful shutdown to ensure all spans are exported.
#[cfg(feature = "otel")]
pub fn shutdown() {
    opentelemetry::global::shutdown_tracer_provider();
}

/// No-op shutdown when otel feature is disabled
#[cfg(not(feature = "otel"))]
pub fn shutdown() {}

#[cfg(test)]
mod tests {
    use super::TraceExportConfig;

    #[test]
    fn trace_export_config_defaults_to_stdout() {
        std::env::remove_var("NEXIS_OTEL_EXPORTER");
        std::env::remove_var("NEXIS_OTEL_EXPORT_ENDPOINT");
        let cfg = TraceExportConfig::from_env().unwrap();
        assert_eq!(cfg.exporter, "stdout");
        assert!(cfg.endpoint.is_none());
    }
}
