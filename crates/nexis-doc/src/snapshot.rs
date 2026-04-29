//! Snapshot models for document persistence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Snapshot metadata fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotMeta {
    pub id: Uuid,
    pub doc_id: Uuid,
    pub version: u64,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

/// Serialized document snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocSnapshot {
    pub meta: SnapshotMeta,
    pub storage_url: Option<String>,
    pub payload: serde_json::Value,
}
