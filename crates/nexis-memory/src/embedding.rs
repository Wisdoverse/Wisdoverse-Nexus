//! Embedding primitives and service trait.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::MemoryResult;

/// Dense embedding vector wrapper.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EmbeddingVector {
    /// Raw embedding values.
    pub values: Vec<f32>,
}

impl EmbeddingVector {
    /// Create a new embedding vector.
    #[must_use]
    pub fn new(values: Vec<f32>) -> Self {
        Self { values }
    }

    /// Return vector dimension.
    #[must_use]
    pub fn dimension(&self) -> usize {
        self.values.len()
    }
}

/// Service abstraction for text embedding generation.
#[async_trait]
pub trait EmbeddingService: Send + Sync {
    /// Generate an embedding for input text.
    async fn embed_text(&self, text: &str) -> MemoryResult<EmbeddingVector>;

    /// Batch-generate embeddings for multiple texts.
    async fn embed_batch(&self, texts: &[String]) -> MemoryResult<Vec<EmbeddingVector>> {
        let mut out = Vec::with_capacity(texts.len());
        for text in texts {
            out.push(self.embed_text(text).await?);
        }
        Ok(out)
    }
}
