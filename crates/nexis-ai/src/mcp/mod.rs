//! MCP (Model Context Protocol) integration
//!
//! Re-exports AI providers for MCP compatibility.

mod gemini;
mod registry;

pub use gemini::GeminiProvider;
pub use registry::{create_provider, create_provider_from_env, ProviderKind};

// Re-export providers from parent module for convenience
pub use crate::providers::{AnthropicProvider, OpenAIProvider};
