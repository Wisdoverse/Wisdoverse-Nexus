//! Structured logging configuration for Wisdoverse Nexus Gateway
//!
//! This module provides structured logging using `tracing-subscriber` with
//! support for both JSON (production) and human-readable (development) formats.
//!
//! # Configuration
//!
//! Environment variables:
//! - `NEXIS_LOG_FORMAT`: `json` or `text` (default: `json`)
//! - `NEXIS_LOG_LEVEL`: Log level filter (default: `info`)
//! - `RUST_LOG`: Alternative log level filter (fallback)
//! - `NEXIS_LOG_SPANS`: Include span context (default: `true`)
//!
//! # Request ID Tracking
//!
//! The logging layer automatically captures and includes request IDs from
//! tracing spans. Use the `tracing::info_span!` macro with a `request_id` field
//! to correlate log entries across a request lifecycle.

use anyhow::{anyhow, Result};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// Log output format configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogFormat {
    /// JSON structured logging (recommended for production)
    #[default]
    Json,
    /// Human-readable plain text (recommended for development)
    Text,
}

impl std::str::FromStr for LogFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "text" | "plain" | "pretty" => Ok(Self::Text),
            other => Err(format!(
                "invalid log format '{}', expected 'json' or 'text'",
                other
            )),
        }
    }
}

/// Logging configuration derived from environment variables.
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Output format for log entries.
    pub format: LogFormat,
    /// Log level filter (e.g., "info", "debug").
    pub level: String,
    /// Whether to include span context in log output.
    pub with_spans: bool,
    /// Whether to include target module in log output.
    pub with_target: bool,
    /// Whether to include thread IDs in log output.
    pub with_thread_ids: bool,
    /// Whether to include thread names in log output.
    pub with_thread_names: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            format: LogFormat::Json,
            level: "info".to_string(),
            with_spans: true,
            with_target: true,
            with_thread_ids: true,
            with_thread_names: true,
        }
    }
}

impl LoggingConfig {
    /// Create configuration from environment variables.
    ///
    /// # Environment Variables
    ///
    /// - `NEXIS_LOG_FORMAT`: Log format (`json` or `text`), defaults to `json`
    /// - `NEXIS_LOG_LEVEL`: Log level, defaults to `info`
    /// - `NEXIS_LOG_SPANS`: Include span context (`true`/`false`), defaults to `true`
    /// - `RUST_LOG`: Fallback log level if `NEXIS_LOG_LEVEL` not set
    pub fn from_env() -> Self {
        let format = std::env::var("NEXIS_LOG_FORMAT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or_default();

        let level = std::env::var("NEXIS_LOG_LEVEL")
            .or_else(|_| std::env::var("RUST_LOG"))
            .unwrap_or_else(|_| "nexis_gateway=info,tower_http=info".to_string());

        let with_spans = std::env::var("NEXIS_LOG_SPANS")
            .map(|v| !v.eq_ignore_ascii_case("false"))
            .unwrap_or(true);

        Self {
            format,
            level,
            with_spans,
            with_target: true,
            with_thread_ids: true,
            with_thread_names: true,
        }
    }

    /// Create a development-friendly configuration.
    pub fn development() -> Self {
        Self {
            format: LogFormat::Text,
            level: "debug".to_string(),
            with_spans: true,
            with_target: true,
            with_thread_ids: false,
            with_thread_names: false,
        }
    }

    /// Create a production-friendly configuration.
    pub fn production() -> Self {
        Self {
            format: LogFormat::Json,
            level: "info".to_string(),
            with_spans: true,
            with_target: true,
            with_thread_ids: true,
            with_thread_names: true,
        }
    }
}

/// Initialize structured logging with the given configuration.
///
/// This sets up a `tracing-subscriber` with the configured format and filters.
///
/// # Errors
///
/// Returns an error if the subscriber has already been initialized or
/// if the filter directive is invalid.
///
/// # Example
///
/// ```rust,no_run
/// use nexis_gateway::observability::logging::{init_logging_with_config, LoggingConfig, LogFormat};
///
/// let config = LoggingConfig {
///     format: LogFormat::Json,
///     level: "debug".to_string(),
///     ..Default::default()
/// };
///
/// init_logging_with_config(config).expect("Failed to initialize logging");
/// ```
pub fn init_logging_with_config(config: LoggingConfig) -> Result<()> {
    let env_filter = EnvFilter::new(&config.level);

    match config.format {
        LogFormat::Json => {
            let layer = tracing_subscriber::fmt::layer()
                .json()
                .with_current_span(config.with_spans)
                .with_span_list(config.with_spans)
                .with_target(config.with_target)
                .with_thread_ids(config.with_thread_ids)
                .with_thread_names(config.with_thread_names);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(layer)
                .try_init()
                .map_err(|e| anyhow!("failed to initialize logging subscriber: {}", e))?;
        }
        LogFormat::Text => {
            let layer = tracing_subscriber::fmt::layer()
                .with_target(config.with_target)
                .with_thread_ids(config.with_thread_ids)
                .with_thread_names(config.with_thread_names);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(layer)
                .try_init()
                .map_err(|e| anyhow!("failed to initialize logging subscriber: {}", e))?;
        }
    }

    tracing::info!(
        target: "observability.logging",
        format = ?config.format,
        level = %config.level,
        with_spans = config.with_spans,
        "Structured logging initialized"
    );

    Ok(())
}

/// Initialize structured logging with environment-based configuration.
///
/// This is the main entry point for logging initialization.
/// Configuration is read from environment variables.
///
/// # Environment Variables
///
/// See [`LoggingConfig::from_env`] for supported environment variables.
///
/// # Errors
///
/// Returns an error if the subscriber has already been initialized.
///
/// # Example
///
/// ```rust,no_run
/// // Set via environment:
/// // NEXIS_LOG_FORMAT=json
/// // NEXIS_LOG_LEVEL=info
///
/// use nexis_gateway::observability::logging::init_logging;
///
/// init_logging().expect("Failed to initialize logging");
/// ```
pub fn init_logging() -> Result<()> {
    let config = LoggingConfig::from_env();
    init_logging_with_config(config)
}

/// Create a JSON format logging layer.
///
/// This is useful when you need to add the JSON layer to an existing
/// subscriber (e.g., when combining with OpenTelemetry).
pub fn create_json_layer(with_spans: bool) -> impl Layer<tracing_subscriber::Registry> {
    tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(with_spans)
        .with_span_list(with_spans)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
}

/// Create a text format logging layer.
///
/// This is useful when you need to add the text layer to an existing
/// subscriber (e.g., when combining with OpenTelemetry).
pub fn create_text_layer() -> impl Layer<tracing_subscriber::Registry> {
    tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn log_format_parsing() {
        assert_eq!(LogFormat::from_str("json").unwrap(), LogFormat::Json);
        assert_eq!(LogFormat::from_str("JSON").unwrap(), LogFormat::Json);
        assert_eq!(LogFormat::from_str("text").unwrap(), LogFormat::Text);
        assert_eq!(LogFormat::from_str("TEXT").unwrap(), LogFormat::Text);
        assert_eq!(LogFormat::from_str("plain").unwrap(), LogFormat::Text);
        assert_eq!(LogFormat::from_str("pretty").unwrap(), LogFormat::Text);
        assert!(LogFormat::from_str("invalid").is_err());
    }

    #[test]
    fn logging_config_defaults() {
        let config = LoggingConfig::default();
        assert_eq!(config.format, LogFormat::Json);
        assert!(config.with_spans);
        assert!(config.with_target);
    }

    #[test]
    fn development_config() {
        let config = LoggingConfig::development();
        assert_eq!(config.format, LogFormat::Text);
        assert_eq!(config.level, "debug");
    }

    #[test]
    fn production_config() {
        let config = LoggingConfig::production();
        assert_eq!(config.format, LogFormat::Json);
        assert_eq!(config.level, "info");
    }
}
