//! Search application port and contract types.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Search request parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    /// Search query text.
    pub query: String,
    /// Maximum number of results.
    pub limit: Option<usize>,
    /// Minimum similarity score (0.0 to 1.0).
    pub min_score: Option<f32>,
    /// Filter to specific room.
    pub room_id: Option<Uuid>,
    /// Include full content in results.
    pub include_content: Option<bool>,
}

impl SearchRequest {
    /// Create a new search request.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            limit: None,
            min_score: None,
            room_id: None,
            include_content: None,
        }
    }

    /// Set result limit.
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set minimum score threshold.
    pub fn with_min_score(mut self, score: f32) -> Self {
        self.min_score = Some(score);
        self
    }

    /// Filter to specific room.
    pub fn in_room(mut self, room_id: Uuid) -> Self {
        self.room_id = Some(room_id);
        self
    }
}

/// Search result item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    /// Document ID.
    pub id: Uuid,
    /// Similarity score.
    pub score: f32,
    /// Document content, if included.
    pub content: Option<String>,
    /// Room ID.
    pub room_id: Option<Uuid>,
    /// Custom metadata.
    pub metadata: serde_json::Value,
}

/// Search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Original query.
    pub query: String,
    /// Search results.
    pub results: Vec<SearchResultItem>,
    /// Total results found.
    pub total: usize,
    /// Whether results were truncated.
    pub truncated: bool,
}

impl SearchResponse {
    /// Create a new search response.
    pub fn new(query: String, results: Vec<SearchResultItem>) -> Self {
        let total = results.len();
        Self {
            query,
            results,
            total,
            truncated: false,
        }
    }

    /// Mark response as truncated.
    pub fn with_truncated(mut self) -> Self {
        self.truncated = true;
        self
    }
}

/// Search service port used by application use cases.
#[async_trait]
pub trait SearchService: Send + Sync {
    /// Perform semantic search.
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse, SearchError>;

    /// Search within a specific room.
    async fn search_in_room(
        &self,
        query: &str,
        room_id: Uuid,
        limit: usize,
    ) -> Result<SearchResponse, SearchError>;
}

/// Search error type.
#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("Embedding generation failed: {0}")]
    EmbeddingError(String),

    #[error("Vector search failed: {0}")]
    VectorError(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_request_builder() {
        let room_id = Uuid::new_v4();
        let request = SearchRequest::new("test query")
            .with_limit(5)
            .with_min_score(0.5)
            .in_room(room_id);

        assert_eq!(request.query, "test query");
        assert_eq!(request.limit, Some(5));
        assert_eq!(request.min_score, Some(0.5));
        assert_eq!(request.room_id, Some(room_id));
    }
}
