//! Error types for skill registration and execution.

use thiserror::Error;

/// Result type for skill operations.
pub type SkillResult<T> = Result<T, SkillError>;

/// Skill crate error type.
#[derive(Debug, Error)]
pub enum SkillError {
    /// Skill could not be found.
    #[error("Skill not found: {0}")]
    NotFound(String),

    /// Skill already exists.
    #[error("Skill already exists: {0}")]
    AlreadyExists(String),

    /// Skill input failed validation.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Execution error raised by the executor.
    #[error("Execution failed: {0}")]
    Execution(String),

    /// JSON serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
