# nexis-memory

Long-term memory and context persistence for Wisdoverse Nexus AI agents.

## What It Provides

- `MemoryEntry` and `MemoryType` domain types
- `MemoryStore` trait for pluggable persistence backends
- `ContextWindow` and `ContextManager` for prompt context assembly
- `EmbeddingVector` and `EmbeddingService` trait for embedding providers
- `MemoryError` and `MemoryResult` for unified error handling

## Notes

- Backends are intentionally left abstract through traits.
- SQL support can be wired in through the optional `sqlx` feature.
