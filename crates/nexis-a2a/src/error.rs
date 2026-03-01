//! Error types for the A2A protocol crate.

use thiserror::Error;

/// Unified error type for A2A operations.
#[derive(Debug, Error)]
pub enum A2AError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("message error: {0}")]
    Message(String),
    #[error("discovery error: {0}")]
    Discovery(String),
    #[error("collaboration error: {0}")]
    Collaboration(String),
    #[error("internal error: {0}")]
    Internal(String),
}

/// Standard result type for A2A crate operations.
pub type A2AResult<T> = Result<T, A2AError>;
