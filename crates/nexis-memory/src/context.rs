//! Context window types and management over memory storage.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::MemoryResult;
use crate::memory::{MemoryEntry, MemoryStore};

/// In-memory context window used to build prompts.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextWindow {
    /// Entries currently loaded into context.
    pub entries: Vec<MemoryEntry>,
    /// Maximum number of entries retained.
    pub max_entries: usize,
}

impl ContextWindow {
    /// Create an empty context window.
    #[must_use]
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    /// Push an entry and evict oldest entries if over capacity.
    pub fn push(&mut self, entry: MemoryEntry) {
        self.entries.push(entry);
        if self.entries.len() > self.max_entries {
            let drop_count = self.entries.len() - self.max_entries;
            self.entries.drain(0..drop_count);
        }
    }
}

/// Convenience manager that coordinates memory store access and context windows.
pub struct ContextManager<S: MemoryStore> {
    store: Arc<S>,
    default_window_size: usize,
}

impl<S: MemoryStore> ContextManager<S> {
    /// Create a new manager with a default context size.
    #[must_use]
    pub fn new(store: Arc<S>, default_window_size: usize) -> Self {
        Self {
            store,
            default_window_size,
        }
    }

    /// Build a context window from recent memories.
    pub async fn load_recent(&self, agent_id: Uuid) -> MemoryResult<ContextWindow> {
        self.load_recent_with_limit(agent_id, self.default_window_size)
            .await
    }

    /// Build a context window from recent memories using an explicit size.
    pub async fn load_recent_with_limit(
        &self,
        agent_id: Uuid,
        limit: usize,
    ) -> MemoryResult<ContextWindow> {
        let mut window = ContextWindow::new(limit);
        for memory in self.store.recent(agent_id, limit).await? {
            window.push(memory);
        }
        Ok(window)
    }
}
