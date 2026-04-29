//! Error types for task operations.

use crate::task::TaskStatus;
use crate::workflow::TransitionError;
use thiserror::Error;
use uuid::Uuid;

/// Result type for task operations.
pub type TaskResult<T> = Result<T, TaskError>;

/// Task crate error type.
#[derive(Debug, Error)]
pub enum TaskError {
    /// Entity could not be found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Input validation failed.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Assignment operation failed.
    #[error("Assignment error: {0}")]
    Assignment(String),

    /// Report operation failed.
    #[error("Report error: {0}")]
    Report(String),

    /// Task cannot move between given states.
    #[error("Invalid task transition: {from:?} -> {to:?}")]
    InvalidTransition { from: TaskStatus, to: TaskStatus },

    /// Task operation requires an assignee but none is set.
    #[error("Task requires assignee: {task_id}")]
    MissingAssignee { task_id: Uuid },

    /// Task due date is invalid for workflow constraints.
    #[error("Invalid due date for task {task_id}: {reason}")]
    InvalidDueDate { task_id: Uuid, reason: String },

    /// Catch-all for irrecoverable internal failures.
    #[error("Internal task error: {0}")]
    Internal(String),

    /// JSON serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl From<TransitionError> for TaskError {
    fn from(value: TransitionError) -> Self {
        match value {
            TransitionError::InvalidTransition { from, to } => Self::InvalidTransition { from, to },
            TransitionError::InvalidBlockReason => {
                Self::InvalidInput("block reason cannot be empty".to_string())
            }
        }
    }
}
