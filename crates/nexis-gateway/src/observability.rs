//! Observability bootstrap for logging and tracing.

use anyhow::{anyhow, Result};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceExportConfig {
    pub exporter: String,
    pub endpoint: Option<String>,
}

impl TraceExportConfig {
    pub fn from_env() -> Result<Self> {
        let exporter = std::env::var("NEXIS_OTEL_EXPORTER").unwrap_or_else(|_| "stdout".to_string());
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

    if trace_export.exporter == "otlp" && trace_export.endpoint.is_none() {
        tracing::warn!(
            target: "observability",
            "NEXIS_OTEL_EXPORTER=otlp is set without NEXIS_OTEL_EXPORT_ENDPOINT"
        );
    }

    Ok(())
}

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
