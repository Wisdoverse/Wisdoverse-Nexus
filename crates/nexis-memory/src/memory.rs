//! Memory entities and storage abstraction.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::embedding::EmbeddingVector;
use crate::error::MemoryResult;

/// Classifies what kind of memory entry is stored.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    Conversation,
    Fact,
    Preference,
    Task,
    Custom(String),
}

/// A persisted memory record.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Unique memory identifier.
    pub id: Uuid,
    /// Agent identifier that owns this memory.
    pub agent_id: Uuid,
    /// Memory classification.
    pub memory_type: MemoryType,
    /// Main memory content.
    pub content: String,
    /// Arbitrary metadata payload.
    pub metadata: Value,
    /// Optional embedding for vector search.
    pub embedding: Option<EmbeddingVector>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

impl MemoryEntry {
    /// Build a new entry with generated identifiers and timestamps.
    #[must_use]
    pub fn new(agent_id: Uuid, memory_type: MemoryType, content: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            agent_id,
            memory_type,
            content: content.into(),
            metadata: Value::Object(serde_json::Map::new()),
            embedding: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Storage abstraction for long-term memory.
#[async_trait]
pub trait MemoryStore: Send + Sync {
    /// Insert or update a memory entry.
    async fn upsert(&self, entry: MemoryEntry) -> MemoryResult<MemoryEntry>;

    /// Fetch a memory by id.
    async fn get(&self, id: Uuid) -> MemoryResult<Option<MemoryEntry>>;

    /// Delete a memory by id.
    async fn delete(&self, id: Uuid) -> MemoryResult<()>;

    /// Search memories for an agent by query text.
    async fn search(
        &self,
        agent_id: Uuid,
        query: &str,
        limit: usize,
    ) -> MemoryResult<Vec<MemoryEntry>>;

    /// List most recent memories for an agent.
    async fn recent(&self, agent_id: Uuid, limit: usize) -> MemoryResult<Vec<MemoryEntry>>;
}
