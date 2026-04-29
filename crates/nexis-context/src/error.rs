//! Error types for context module

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("Summarization failed: {0}")]
    SummarizationFailed(String),

    #[error("AI provider error: {0}")]
    AiProviderError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Context window is full")]
    WindowFull,

    #[error("Summarization not available")]
    SummarizationNotAvailable,
}

pub type ContextResult<T> = Result<T, ContextError>;
