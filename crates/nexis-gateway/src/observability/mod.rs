//! Observability infrastructure for Wisdoverse Nexus Gateway
//!
//! This module provides structured logging and distributed tracing capabilities
//! for production-grade observability.
//!
//! # Features
//!
//! - `otel` - Enable OpenTelemetry OTLP export for distributed tracing
//!
//! # Environment Variables
//!
//! ## Logging Configuration
//! - `NEXIS_LOG_FORMAT` - Log format: `json` (production) or `text` (development), defaults to `json`
//! - `NEXIS_LOG_LEVEL` - Log level filter, defaults to `info`
//! - `RUST_LOG` - Alternative log level filter (lower priority than `NEXIS_LOG_LEVEL`)
//! - `NEXIS_LOG_SPANS` - Include span context in logs, defaults to `true`
//!
//! ## Tracing Configuration (requires `otel` feature)
//! - `NEXIS_OTEL_EXPORTER` - Exporter type: `otlp`, `stdout`, or `none`, defaults to `stdout`
//! - `NEXIS_OTEL_EXPORT_ENDPOINT` - OTLP collector endpoint (e.g., `http://localhost:4317`)
//!
//! # Example
//!
//! ```rust,no_run
//! use nexis_gateway::observability::init_logging;
//!
//! // Initialize logging with environment-based configuration
//! init_logging().expect("Failed to initialize logging");
//!
//! // Now you can use tracing macros
//! tracing::info!(message = "Server started", port = 8080);
//! ```

pub mod logging;

#[cfg(feature = "otel")]
pub mod tracing;

// Re-export main initialization functions for convenience
pub use logging::{init_logging, LoggingConfig};

#[cfg(feature = "otel")]
pub use tracing::{init_tracing, shutdown_tracing, TracingConfig};

/// Initialize observability with default configuration.
///
/// This is the main entry point for setting up observability.
/// It initializes structured logging and, if the `otel` feature is enabled
/// and configured, distributed tracing.
///
/// # Errors
///
/// Returns an error if the logging subscriber fails to initialize.
///
/// # Example
///
/// ```rust,no_run
/// use nexis_gateway::observability::init;
///
/// init().expect("Failed to initialize observability");
/// ```
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    init_logging()?;

    #[cfg(feature = "otel")]
    {
        init_tracing()?;
    }

    Ok(())
}

/// Shutdown observability components gracefully.
///
/// Call this during application shutdown to ensure all pending
/// spans and traces are flushed.
pub fn shutdown() {
    #[cfg(feature = "otel")]
    shutdown_tracing();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logging_config_defaults() {
        let config = LoggingConfig::from_env();
        // Default format should be json
        assert_eq!(config.format, logging::LogFormat::Json);
    }
}
