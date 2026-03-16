//! Nexis Context Management - Context windowing and management
//!
//! This crate provides:
//! - Context window management
//! - Token counting (optional, feature-gated)
//! - Conversation context tracking
//! - Context summarization (when window overflows)
//!
//! ## Features
//!
//! - `token-counting` - Enable accurate token counting using tokenizers
//! - `ai-summarizer` - Enable AI-powered summarization using nexis-runtime
//! - `metrics` - Enable Prometheus metrics for monitoring

pub mod context;
pub mod error;
pub mod manager;
pub mod summarizer;
pub mod window;

#[cfg(feature = "ai-summarizer")]
pub mod ai_summarizer;

#[cfg(feature = "ai-summarizer")]
pub use ai_summarizer::AISummarizer;

#[cfg(feature = "metrics")]
pub mod metrics;

#[cfg(feature = "metrics")]
pub use metrics::{
    record_summarization_failure, record_summarization_overflow, record_summarization_success,
    record_truncation, record_window_utilization, set_active_contexts, CONTEXTS_ACTIVE,
    MESSAGES_SUMMARIZED, MESSAGES_TRUNCATED, OVERFLOW_EVENTS, SUMMARIZATION_ATTEMPTS,
    SUMMARIZATION_LATENCY, TOKEN_SAVINGS,
};

pub use context::{ConversationContext, Message, MessageRole};
pub use error::{ContextError, ContextResult};
pub use manager::ContextManager;
pub use summarizer::{ContextSummarizer, MockSummarizer, NoOpSummarizer, SummarizerConfig};
pub use window::{ContextWindow, OverflowStrategy};

/// Prelude for common imports
pub mod prelude {
    pub use crate::context::ConversationContext;
    pub use crate::error::{ContextError, ContextResult};
    pub use crate::manager::ContextManager;
    pub use crate::window::ContextWindow;
}
