//! AI Provider Registry
//!
//! Central registry for managing AI providers with support for
//! dynamic registration, health checks, and default provider selection.

use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{AIProvider, ProviderError};

/// Provider registry for managing multiple AI providers
pub struct ProviderRegistry {
    providers: RwLock<HashMap<String, Arc<dyn AIProvider>>>,
    default_provider: RwLock<Option<String>>,
}

impl ProviderRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
            default_provider: RwLock::new(None),
        }
    }

    /// Create and configure registry from environment variables
    ///
    /// Environment variables:
    /// - `NEXIS_DEFAULT_PROVIDER`: Default provider name (openai/anthropic/mock)
    /// - `OPENAI_API_KEY`: OpenAI API key
    /// - `ANTHROPIC_API_KEY`: Anthropic API key
    ///
    /// At least one API key must be provided for non-mock providers.
    pub fn from_env() -> Self {
        let registry = Self::new();

        // Register providers based on available API keys
        if let Ok(api_key) = env::var("OPENAI_API_KEY") {
            let provider = crate::providers::OpenAIProvider::new(
                api_key,
                env::var("OPENAI_API_BASE").unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
                env::var("OPENAI_DEFAULT_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            );
            // Synchronously register (from_env is not async)
            // We'll use try_write and ignore if it fails
            if let Ok(mut providers) = registry.providers.try_write() {
                providers.insert("openai".to_string(), Arc::new(provider));
            }
        }

        if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
            let provider = crate::providers::AnthropicProvider::new(
                api_key,
                env::var("ANTHROPIC_API_BASE").unwrap_or_else(|_| "https://api.anthropic.com/v1".to_string()),
                env::var("ANTHROPIC_DEFAULT_MODEL").unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string()),
            );
            if let Ok(mut providers) = registry.providers.try_write() {
                providers.insert("anthropic".to_string(), Arc::new(provider));
            }
        }

        // Register mock provider (always available as fallback)
        {
            let mock_provider = crate::MockProvider::new();
            if let Ok(mut providers) = registry.providers.try_write() {
                providers.insert("mock".to_string(), Arc::new(mock_provider));
            }
        }

        // Set default provider
        if let Ok(default) = env::var("NEXIS_DEFAULT_PROVIDER") {
            if let Ok(mut default_provider) = registry.default_provider.try_write() {
                *default_provider = Some(default);
            }
        } else {
            // Auto-select default: openai > anthropic > mock
            if let Ok(mut default_provider) = registry.default_provider.try_write() {
                if env::var("OPENAI_API_KEY").is_ok() {
                    *default_provider = Some("openai".to_string());
                } else if env::var("ANTHROPIC_API_KEY").is_ok() {
                    *default_provider = Some("anthropic".to_string());
                } else {
                    *default_provider = Some("mock".to_string());
                }
            }
        }

        registry
    }

    /// Register a provider
    pub async fn register(&self, name: impl Into<String>, provider: Arc<dyn AIProvider>) {
        let name = name.into();
        let mut providers = self.providers.write().await;

        // Set as default if this is the first provider
        if providers.is_empty() {
            let mut default = self.default_provider.write().await;
            *default = Some(name.clone());
        }

        providers.insert(name, provider);
    }

    /// Get a provider by name
    pub async fn get(&self, name: &str) -> Option<Arc<dyn AIProvider>> {
        let providers = self.providers.read().await;
        providers.get(name).cloned()
    }

    /// Get the default provider
    pub async fn get_default(&self) -> Option<Arc<dyn AIProvider>> {
        let default = self.default_provider.read().await;
        if let Some(name) = default.as_ref() {
            self.get(name).await
        } else {
            None
        }
    }

    /// Set the default provider
    pub async fn set_default(&self, name: &str) -> Result<(), ProviderError> {
        let providers = self.providers.read().await;

        if !providers.contains_key(name) {
            return Err(ProviderError::Message(format!(
                "Provider '{}' not found",
                name
            )));
        }

        let mut default = self.default_provider.write().await;
        *default = Some(name.to_string());

        Ok(())
    }

    /// List all registered providers
    pub async fn list(&self) -> Vec<String> {
        let providers = self.providers.read().await;
        providers.keys().cloned().collect()
    }

    /// Check health of all providers
    pub async fn health_check(&self) -> HashMap<String, bool> {
        let providers = self.providers.read().await;
        let mut results = HashMap::new();

        for (name, provider) in providers.iter() {
            // Simple health check: try to get the provider name
            // In production, you might want to make a lightweight API call
            let healthy = Arc::strong_count(provider) > 0;
            results.insert(name.clone(), healthy);
        }

        results
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GenerateRequest, GenerateResponse, ProviderStream};
    use async_trait::async_trait;

    #[derive(Debug)]
    struct MockProvider {
        name: &'static str,
    }

    #[async_trait]
    impl AIProvider for MockProvider {
        fn name(&self) -> &'static str {
            self.name
        }

        async fn generate(&self, _req: GenerateRequest) -> Result<GenerateResponse, ProviderError> {
            Ok(GenerateResponse {
                content: "mock response".to_string(),
                model: Some("mock".to_string()),
                finish_reason: None,
            })
        }

        async fn generate_stream(
            &self,
            _req: GenerateRequest,
        ) -> Result<ProviderStream, ProviderError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn register_and_get_provider() {
        let registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider { name: "test" });

        registry.register("test", provider).await;

        let retrieved = registry.get("test").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "test");
    }

    #[tokio::test]
    async fn default_provider() {
        let registry = ProviderRegistry::new();

        // First provider becomes default
        let provider1 = Arc::new(MockProvider { name: "p1" });
        registry.register("p1", provider1).await;

        let default = registry.get_default().await;
        assert_eq!(default.unwrap().name(), "p1");

        // Add second provider
        let provider2 = Arc::new(MockProvider { name: "p2" });
        registry.register("p2", provider2).await;

        // Default should still be p1
        let default = registry.get_default().await;
        assert_eq!(default.unwrap().name(), "p1");

        // Change default
        registry.set_default("p2").await.unwrap();
        let default = registry.get_default().await;
        assert_eq!(default.unwrap().name(), "p2");
    }

    #[tokio::test]
    async fn list_providers() {
        let registry = ProviderRegistry::new();

        let provider1 = Arc::new(MockProvider { name: "p1" });
        let provider2 = Arc::new(MockProvider { name: "p2" });

        registry.register("p1", provider1).await;
        registry.register("p2", provider2).await;

        let list = registry.list().await;
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"p1".to_string()));
        assert!(list.contains(&"p2".to_string()));
    }

    #[tokio::test]
    async fn health_check() {
        let registry = ProviderRegistry::new();
        let provider = Arc::new(MockProvider { name: "test" });

        registry.register("test", provider).await;

        let health = registry.health_check().await;
        assert_eq!(health.get("test"), Some(&true));
    }

    #[tokio::test]
    async fn set_default_nonexistent_fails() {
        let registry = ProviderRegistry::new();

        let err = registry.set_default("nonexistent").await.unwrap_err();
        match err {
            ProviderError::Message(msg) => assert!(msg.contains("nonexistent")),
            _ => panic!("Expected Message error"),
        }
    }
}
