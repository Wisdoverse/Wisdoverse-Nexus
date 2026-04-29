//! Error types for memory and context operations.

use thiserror::Error;

/// Result type for memory operations.
pub type MemoryResult<T> = Result<T, MemoryError>;

/// Memory crate error type.
#[derive(Debug, Error)]
pub enum MemoryError {
    /// Entity could not be found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Input validation failed.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Storage operation failed.
    #[error("Store error: {0}")]
    Store(String),

    /// Embedding generation or processing failed.
    #[error("Embedding error: {0}")]
    Embedding(String),

    /// JSON serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// SQL database error (feature-gated).
    #[cfg(feature = "sqlx")]
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
