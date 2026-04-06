//! Error types for the A2A protocol crate.

use thiserror::Error;

/// Unified error type for A2A operations.
#[derive(Debug, Error)]
pub enum A2AError {
    /// Validation failure for user or protocol input.
    #[error("validation error: {0}")]
    Validation(String),
    /// Message serialization, parsing, or delivery failure.
    #[error("message error: {0}")]
    Message(String),
    /// Discovery query or registry failure.
    #[error("discovery error: {0}")]
    Discovery(String),
    /// Collaboration workflow failure.
    #[error("collaboration error: {0}")]
    Collaboration(String),
    /// Unexpected internal failure.
    #[error("internal error: {0}")]
    Internal(String),
}

/// Standard result type for A2A crate operations.
pub type A2AResult<T> = Result<T, A2AError>;
