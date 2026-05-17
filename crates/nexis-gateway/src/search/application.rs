//! Application use cases for gateway search.

use std::sync::Arc;

use uuid::Uuid;

use super::service::{SearchError, SearchRequest, SearchService};

#[derive(Clone)]
pub struct SearchApplication {
    service: Arc<dyn SearchService>,
}

impl SearchApplication {
    pub fn new(service: Arc<dyn SearchService>) -> Self {
        Self { service }
    }

    pub async fn search_messages(
        &self,
        query: SearchMessagesQuery,
    ) -> Result<SearchMessagesResult, SearchApplicationError> {
        if query.query.trim().is_empty() {
            return Err(SearchApplicationError::InvalidQuery);
        }

        let mut request = SearchRequest::new(&query.query).with_limit(query.limit);

        if let Some(min_score) = query.min_score {
            request = request.with_min_score(min_score);
        }

        if let Some(room_id) = query.room_id {
            request = request.in_room(room_id);
        }

        let response = self.service.search(request).await?;
        let results = response
            .results
            .into_iter()
            .filter_map(|result| {
                result.content.map(|content| SearchMessagesResultItem {
                    id: result.id,
                    score: result.score,
                    content,
                    room_id: result.room_id,
                })
            })
            .collect::<Vec<_>>();

        Ok(SearchMessagesResult {
            query: response.query,
            total: results.len(),
            results,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SearchMessagesQuery {
    pub query: String,
    pub limit: usize,
    pub min_score: Option<f32>,
    pub room_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct SearchMessagesResult {
    pub query: String,
    pub results: Vec<SearchMessagesResultItem>,
    pub total: usize,
}

#[derive(Debug, Clone)]
pub struct SearchMessagesResultItem {
    pub id: Uuid,
    pub score: f32,
    pub content: String,
    pub room_id: Option<Uuid>,
}

#[derive(Debug, thiserror::Error)]
pub enum SearchApplicationError {
    #[error("search query cannot be empty")]
    InvalidQuery,
    #[error(transparent)]
    Search(#[from] SearchError),
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::super::service::{SearchResponse, SearchResultItem};
    use super::*;

    struct StubSearchService {
        response: SearchMessagesResult,
    }

    #[async_trait]
    impl SearchService for StubSearchService {
        async fn search(&self, request: SearchRequest) -> Result<SearchResponse, SearchError> {
            Ok(SearchResponse {
                query: request.query,
                results: self
                    .response
                    .results
                    .iter()
                    .map(|item| SearchResultItem {
                        id: item.id,
                        score: item.score,
                        content: Some(item.content.clone()),
                        room_id: item.room_id,
                        metadata: serde_json::Value::Null,
                    })
                    .collect(),
                total: self.response.total,
                truncated: false,
            })
        }

        async fn search_in_room(
            &self,
            query: &str,
            room_id: Uuid,
            limit: usize,
        ) -> Result<SearchResponse, SearchError> {
            self.search(SearchRequest::new(query).with_limit(limit).in_room(room_id))
                .await
        }
    }

    #[tokio::test]
    async fn search_messages_rejects_blank_query() {
        let application = SearchApplication::new(Arc::new(StubSearchService {
            response: SearchMessagesResult {
                query: String::new(),
                results: Vec::new(),
                total: 0,
            },
        }));

        let result = application
            .search_messages(SearchMessagesQuery {
                query: " ".to_string(),
                limit: 10,
                min_score: None,
                room_id: None,
            })
            .await;

        assert!(matches!(result, Err(SearchApplicationError::InvalidQuery)));
    }

    #[tokio::test]
    async fn search_messages_maps_service_results() {
        let id = Uuid::new_v4();
        let room_id = Uuid::new_v4();
        let application = SearchApplication::new(Arc::new(StubSearchService {
            response: SearchMessagesResult {
                query: "rust".to_string(),
                total: 1,
                results: vec![SearchMessagesResultItem {
                    id,
                    score: 0.91,
                    content: "rust message".to_string(),
                    room_id: Some(room_id),
                }],
            },
        }));

        let result = application
            .search_messages(SearchMessagesQuery {
                query: "rust".to_string(),
                limit: 10,
                min_score: None,
                room_id: None,
            })
            .await
            .expect("search should succeed");

        assert_eq!(result.total, 1);
        assert_eq!(result.results[0].id, id);
        assert_eq!(result.results[0].room_id, Some(room_id));
    }
}
