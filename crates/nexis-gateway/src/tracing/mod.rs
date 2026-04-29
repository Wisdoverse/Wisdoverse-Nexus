//! Tracing and observability utilities

#[cfg(feature = "otel")]
pub mod otel;

#[cfg(feature = "otel")]
pub use otel::{init_from_env, init_otel, shutdown};

/// No-op for when otel feature is disabled
#[cfg(not(feature = "otel"))]
pub fn shutdown() {}
