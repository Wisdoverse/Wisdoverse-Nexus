//! Error types for document collaboration operations.

use thiserror::Error;
use uuid::Uuid;

/// Result type for document operations.
pub type DocResult<T> = Result<T, DocError>;

/// Doc crate error type.
#[derive(Debug, Error)]
pub enum DocError {
    /// Entity could not be found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Input validation failed.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// CRDT operation failed.
    #[error("CRDT error: {0}")]
    Crdt(String),

    /// Snapshot operation failed.
    #[error("Snapshot error: {0}")]
    Snapshot(String),

    /// Failed to decode or apply a CRDT update payload.
    #[error("Invalid CRDT update: {0}")]
    InvalidCrdtUpdate(String),

    /// Concurrent document updates could not be merged safely.
    #[error("Concurrent edit conflict on document {doc_id}: {reason}")]
    ConcurrentEditConflict { doc_id: Uuid, reason: String },

    /// Snapshot identifier was not found.
    #[error("Snapshot not found: {snapshot_id}")]
    SnapshotNotFound { snapshot_id: Uuid },

    /// Catch-all for irrecoverable internal failures.
    #[error("Internal document error: {0}")]
    Internal(String),

    /// JSON serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
