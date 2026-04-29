//! Error types for calendar operations.

use chrono::{DateTime, Utc};
use thiserror::Error;

/// Result type for calendar operations.
pub type CalendarResult<T> = Result<T, CalendarError>;

/// Calendar crate error type.
#[derive(Debug, Error)]
pub enum CalendarError {
    /// Entity could not be found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Input validation failed.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Conflict detection operation failed.
    #[error("Conflict error: {0}")]
    Conflict(String),

    /// Reminder scheduling operation failed.
    #[error("Reminder error: {0}")]
    Reminder(String),

    /// Event range is malformed (`start >= end`).
    #[error("Invalid time range: start={start}, end={end}")]
    InvalidTimeRange {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },

    /// Recurrence configuration was rejected.
    #[error("Invalid recurrence rule: {0}")]
    InvalidRecurrence(String),

    /// Catch-all for irrecoverable internal failures.
    #[error("Internal calendar error: {0}")]
    Internal(String),

    /// JSON serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
