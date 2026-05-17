//! Infrastructure adapters for semantic search.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use nexis_ai::{EmbeddingProvider, EmbeddingRequest};
use nexis_vector::prelude::*;
use tracing::debug;
use uuid::Uuid;

use super::service::{SearchError, SearchRequest, SearchResponse, SearchResultItem, SearchService};

/// Semantic search service implementation backed by a vector store and an embedding provider.
pub struct SemanticSearchService {
    vector_store: Arc<dyn VectorStore>,
    embedding_provider: Arc<dyn EmbeddingProvider>,
    default_limit: usize,
    query_embedding_cache: Mutex<QueryEmbeddingCache>,
}

impl SemanticSearchService {
    /// Create a new semantic search service.
    pub fn new(
        vector_store: Arc<dyn VectorStore>,
        embedding_provider: Arc<dyn EmbeddingProvider>,
    ) -> Self {
        Self {
            vector_store,
            embedding_provider,
            default_limit: 10,
            query_embedding_cache: Mutex::new(QueryEmbeddingCache::new(256)),
        }
    }

    /// Set default result limit.
    pub fn with_default_limit(mut self, limit: usize) -> Self {
        self.default_limit = limit;
        self
    }

    /// Set maximum number of cached query embeddings.
    pub fn with_cache_capacity(mut self, capacity: usize) -> Self {
        let capped = capacity.max(1);
        self.query_embedding_cache = Mutex::new(QueryEmbeddingCache::new(capped));
        self
    }

    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, SearchError> {
        let cache_key = normalize_query(text);
        if let Some(cached) = self
            .query_embedding_cache
            .lock()
            .expect("embedding cache mutex poisoned")
            .get(&cache_key)
        {
            return Ok(cached);
        }

        let req = EmbeddingRequest::new(text);
        let response = self
            .embedding_provider
            .embed(req)
            .await
            .map_err(|e| SearchError::EmbeddingError(e.to_string()))?;
        let embedding = response.embedding;
        self.query_embedding_cache
            .lock()
            .expect("embedding cache mutex poisoned")
            .insert(cache_key, embedding.clone());
        Ok(embedding)
    }
}

impl From<nexis_vector::SearchResult> for SearchResultItem {
    fn from(result: nexis_vector::SearchResult) -> Self {
        Self {
            id: result.document.id,
            score: result.score,
            content: Some(result.document.content),
            room_id: result.document.metadata.room_id,
            metadata: result.document.metadata.to_json(),
        }
    }
}

#[derive(Debug)]
struct QueryEmbeddingCache {
    map: HashMap<String, Vec<f32>>,
    order: VecDeque<String>,
    capacity: usize,
}

impl QueryEmbeddingCache {
    fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            capacity: capacity.max(1),
        }
    }

    fn get(&self, key: &str) -> Option<Vec<f32>> {
        self.map.get(key).cloned()
    }

    fn insert(&mut self, key: String, value: Vec<f32>) {
        use std::collections::hash_map::Entry;

        if let Entry::Occupied(mut e) = self.map.entry(key.clone()) {
            e.insert(value);
            return;
        }

        if self.map.len() >= self.capacity {
            while let Some(oldest) = self.order.pop_front() {
                if self.map.remove(&oldest).is_some() {
                    break;
                }
            }
        }

        self.order.push_back(key.clone());
        self.map.insert(key, value);
    }
}

fn normalize_query(text: &str) -> String {
    text.trim().to_ascii_lowercase()
}

#[async_trait]
impl SearchService for SemanticSearchService {
    async fn search(&self, request: SearchRequest) -> Result<SearchResponse, SearchError> {
        debug!("Searching for: {}", request.query);

        if request.query.trim().is_empty() {
            return Err(SearchError::InvalidQuery(
                "Query cannot be empty".to_string(),
            ));
        }

        let embedding = self.generate_embedding(&request.query).await?;
        let query_vector = Vector::new(embedding);

        let limit = request.limit.unwrap_or(self.default_limit);
        let mut search_query = SearchQuery::new(query_vector).with_limit(limit);

        if let Some(min_score) = request.min_score {
            search_query = search_query.with_min_score(min_score);
        }

        if let Some(room_id) = request.room_id {
            search_query = search_query.with_filter(SearchFilter::new().with_room(room_id));
        }

        if !request.include_content.unwrap_or(true) {
            search_query = search_query.without_content();
        }

        let results = self
            .vector_store
            .search(search_query)
            .await
            .map_err(|e| SearchError::VectorError(e.to_string()))?;

        let items = results
            .into_iter()
            .map(SearchResultItem::from)
            .collect::<Vec<_>>();

        let truncated = items.len() >= limit;
        let mut response = SearchResponse::new(request.query, items);
        if truncated {
            response = response.with_truncated();
        }

        Ok(response)
    }

    async fn search_in_room(
        &self,
        query: &str,
        room_id: Uuid,
        limit: usize,
    ) -> Result<SearchResponse, SearchError> {
        let request = SearchRequest::new(query).with_limit(limit).in_room(room_id);
        self.search(request).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use nexis_ai::{
        BatchEmbeddingRequest, BatchEmbeddingResponse, EmbeddingResponse, MockEmbeddingProvider,
        ProviderError,
    };
    use nexis_vector::InMemoryVectorStore;

    use super::*;

    fn create_test_service() -> SemanticSearchService {
        let store = Arc::new(InMemoryVectorStore::new(128));
        let embedding = Arc::new(MockEmbeddingProvider::new(128));
        SemanticSearchService::new(store, embedding)
    }

    #[tokio::test]
    async fn search_returns_empty_for_empty_index() {
        let service = create_test_service();
        let request = SearchRequest::new("test").with_limit(10);

        let response = service.search(request).await.unwrap();
        assert_eq!(response.total, 0);
        assert!(response.results.is_empty());
    }

    #[tokio::test]
    async fn search_rejects_empty_query() {
        let service = create_test_service();
        let request = SearchRequest::new("");

        let result = service.search(request).await;
        assert!(matches!(result, Err(SearchError::InvalidQuery(_))));
    }

    #[tokio::test]
    async fn search_in_room_uses_room_filter() {
        let service = create_test_service();
        let room_id = Uuid::new_v4();

        let response = service.search_in_room("test", room_id, 10).await.unwrap();
        assert_eq!(response.total, 0);
    }

    #[derive(Debug)]
    struct CountingEmbeddingProvider {
        calls: AtomicUsize,
        dimension: usize,
    }

    impl CountingEmbeddingProvider {
        fn new(dimension: usize) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                dimension,
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl EmbeddingProvider for CountingEmbeddingProvider {
        fn name(&self) -> &'static str {
            "counting-mock"
        }

        fn dimension(&self) -> usize {
            self.dimension
        }

        async fn embed(&self, _req: EmbeddingRequest) -> Result<EmbeddingResponse, ProviderError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(EmbeddingResponse::new(
                vec![0.1; self.dimension],
                "counting-model",
            ))
        }

        async fn embed_batch(
            &self,
            req: BatchEmbeddingRequest,
        ) -> Result<BatchEmbeddingResponse, ProviderError> {
            let embeddings = req
                .texts
                .iter()
                .map(|_| vec![0.1; self.dimension])
                .collect::<Vec<_>>();
            Ok(BatchEmbeddingResponse {
                embeddings,
                model: "counting-model".to_string(),
                dimension: self.dimension,
                usage: None,
            })
        }
    }

    #[tokio::test]
    async fn search_uses_cached_embedding_for_repeated_query() {
        let store = Arc::new(InMemoryVectorStore::new(128));
        let embedding = Arc::new(CountingEmbeddingProvider::new(128));
        let service = SemanticSearchService::new(store, embedding.clone()).with_cache_capacity(8);

        let request = SearchRequest::new("What is Wisdoverse Nexus?").with_limit(5);
        service.search(request.clone()).await.unwrap();
        service.search(request).await.unwrap();

        assert_eq!(embedding.calls(), 1);
    }
}
