//! Wisdoverse Nexus Memory - long-term memory and context persistence for AI agents.

pub mod context;
pub mod embedding;
pub mod error;
pub mod memory;

pub use context::{ContextManager, ContextWindow};
pub use embedding::{EmbeddingService, EmbeddingVector};
pub use error::{MemoryError, MemoryResult};
pub use memory::{MemoryEntry, MemoryStore, MemoryType};

/// Prelude for common imports.
pub mod prelude {
    pub use crate::context::{ContextManager, ContextWindow};
    pub use crate::embedding::{EmbeddingService, EmbeddingVector};
    pub use crate::error::{MemoryError, MemoryResult};
    pub use crate::memory::{MemoryEntry, MemoryStore, MemoryType};
}
