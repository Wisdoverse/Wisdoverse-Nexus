//! HTTP interfaces for search use cases.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthenticatedUser;

use super::application::{
    SearchApplication, SearchApplicationError, SearchMessagesQuery, SearchMessagesResult,
};
use super::service::SearchError;

/// State required by the search HTTP interface.
pub trait SearchInterfaceState: Clone + Send + Sync + 'static {
    fn search_application(&self) -> Option<&SearchApplication>;
}

#[derive(Debug, Clone, Deserialize)]
struct SearchQueryParams {
    q: String,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    min_score: Option<f32>,
    #[serde(default)]
    room_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
struct SearchApiRequest {
    query: String,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    min_score: Option<f32>,
    #[serde(default)]
    room_id: Option<Uuid>,
}

fn default_limit() -> usize {
    10
}

#[derive(Debug, Clone, Serialize)]
struct SearchApiResponse {
    query: String,
    results: Vec<SearchResultItem>,
    total: usize,
}

#[derive(Debug, Clone, Serialize)]
struct SearchResultItem {
    id: Uuid,
    score: f32,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    room_id: Option<Uuid>,
}

mod error_codes {
    pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";
    pub const INVALID_QUERY: &str = "INVALID_QUERY";
    pub const SEARCH_UNAVAILABLE: &str = "SEARCH_UNAVAILABLE";
}

#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<&'static str>,
}

impl ErrorResponse {
    fn internal_error() -> Self {
        Self {
            error: "An internal error occurred. Please try again later.".to_string(),
            code: Some(error_codes::INTERNAL_ERROR),
        }
    }
}

impl From<SearchError> for ErrorResponse {
    fn from(err: SearchError) -> Self {
        tracing::error!("Search error: {}", err);
        match err {
            SearchError::InvalidQuery(_) => Self {
                error: "Invalid search query".to_string(),
                code: Some(error_codes::INVALID_QUERY),
            },
            SearchError::EmbeddingError(_) | SearchError::VectorError(_) => Self::internal_error(),
        }
    }
}

/// Build search routes.
pub fn routes<S>() -> Router<S>
where
    S: SearchInterfaceState,
{
    Router::new().route(
        "/v1/search",
        get(search_messages_get::<S>).post(search_messages::<S>),
    )
}

fn search_unavailable_response() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorResponse {
            error: "Search service not configured".to_string(),
            code: Some(error_codes::SEARCH_UNAVAILABLE),
        }),
    )
        .into_response()
}

fn search_application_error_response(error: SearchApplicationError) -> Response {
    match error {
        SearchApplicationError::InvalidQuery => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid search query".to_string(),
                code: Some(error_codes::INVALID_QUERY),
            }),
        )
            .into_response(),
        SearchApplicationError::Search(error) => {
            let status = if matches!(error, SearchError::InvalidQuery(_)) {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, Json(ErrorResponse::from(error))).into_response()
        }
    }
}

fn search_api_response(response: SearchMessagesResult) -> SearchApiResponse {
    let results = response
        .results
        .into_iter()
        .map(|result| SearchResultItem {
            id: result.id,
            score: result.score,
            content: result.content,
            room_id: result.room_id,
        })
        .collect();

    SearchApiResponse {
        query: response.query,
        results,
        total: response.total,
    }
}

#[tracing::instrument(
    name = "gateway.search_messages.post",
    skip(state, _user, payload),
    fields(limit = payload.limit)
)]
async fn search_messages<S>(
    State(state): State<S>,
    _user: AuthenticatedUser,
    Json(payload): Json<SearchApiRequest>,
) -> Response
where
    S: SearchInterfaceState,
{
    let Some(search_application) = state.search_application() else {
        return search_unavailable_response();
    };

    if payload.query.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Query cannot be empty".to_string(),
                code: Some(error_codes::INVALID_QUERY),
            }),
        )
            .into_response();
    }

    match search_application
        .search_messages(SearchMessagesQuery {
            query: payload.query,
            limit: payload.limit,
            min_score: payload.min_score,
            room_id: payload.room_id,
        })
        .await
    {
        Ok(response) => (StatusCode::OK, Json(search_api_response(response))).into_response(),
        Err(error) => search_application_error_response(error),
    }
}

#[tracing::instrument(
    name = "gateway.search_messages.get",
    skip(state, _user, params),
    fields(limit = params.limit)
)]
async fn search_messages_get<S>(
    State(state): State<S>,
    _user: AuthenticatedUser,
    Query(params): Query<SearchQueryParams>,
) -> Response
where
    S: SearchInterfaceState,
{
    let Some(search_application) = state.search_application() else {
        return search_unavailable_response();
    };

    if params.q.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Query parameter 'q' is required".to_string(),
                code: Some(error_codes::INVALID_QUERY),
            }),
        )
            .into_response();
    }

    match search_application
        .search_messages(SearchMessagesQuery {
            query: params.q,
            limit: params.limit,
            min_score: params.min_score,
            room_id: params.room_id,
        })
        .await
    {
        Ok(response) => (StatusCode::OK, Json(search_api_response(response))).into_response(),
        Err(error) => search_application_error_response(error),
    }
}
