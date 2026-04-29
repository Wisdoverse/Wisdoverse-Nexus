//! Comment and thread models for document collaboration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Anchor location for a comment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentAnchor {
    pub start_offset: usize,
    pub end_offset: usize,
    pub block_id: Option<String>,
}

/// Single comment entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub doc_id: Uuid,
    pub author_id: Uuid,
    pub content: String,
    pub anchor: CommentAnchor,
    pub created_at: DateTime<Utc>,
}

/// Thread of comments with lifecycle state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentThread {
    pub id: Uuid,
    pub doc_id: Uuid,
    pub comments: Vec<Comment>,
    pub resolved: bool,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
}
