//! Semantic search service for gateway
//!
//! This module provides:
//! - Semantic search across messages
//! - Room-scoped search
//! - Search result ranking and filtering

mod application;
mod infrastructure;
mod interfaces;
mod service;

pub use application::{
    SearchApplication, SearchApplicationError, SearchMessagesQuery, SearchMessagesResult,
};
pub use infrastructure::SemanticSearchService;
pub use interfaces::{routes, SearchInterfaceState};
pub use service::{SearchError, SearchRequest, SearchResponse, SearchService};
