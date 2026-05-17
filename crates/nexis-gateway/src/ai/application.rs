//! AI chat application service.

use super::domain::{self, AiMention, AiResponse};
use nexis_ai::{GenerateRequest, ProviderRegistry};
use nexis_context::{ContextManager, Message as ContextMessage, MessageRole};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// AI application configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiApplicationConfig {
    /// AI mention triggers, without the leading `@`.
    pub triggers: Vec<String>,
    /// Default agent name for responses.
    pub default_agent_name: String,
    /// Max context messages to include.
    pub max_context_messages: usize,
    /// System prompt prefix for AI.
    pub system_prompt: String,
}

impl Default for AiApplicationConfig {
    fn default() -> Self {
        Self {
            triggers: domain::default_triggers(),
            default_agent_name: "ai".to_string(),
            max_context_messages: 20,
            system_prompt:
                "You are a helpful AI assistant in a group chat. Be concise and helpful."
                    .to_string(),
        }
    }
}

/// Application service for processing AI chat mentions.
#[derive(Clone)]
pub struct AiApplication {
    context_manager: Arc<ContextManager>,
    provider_registry: Arc<ProviderRegistry>,
    config: AiApplicationConfig,
}

impl AiApplication {
    /// Create a new AI application service.
    pub fn new(
        context_manager: Arc<ContextManager>,
        provider_registry: Arc<ProviderRegistry>,
        config: AiApplicationConfig,
    ) -> Self {
        Self {
            context_manager,
            provider_registry,
            config,
        }
    }

    /// Create with default config.
    pub fn with_defaults(
        context_manager: Arc<ContextManager>,
        provider_registry: Arc<ProviderRegistry>,
    ) -> Self {
        Self::new(
            context_manager,
            provider_registry,
            AiApplicationConfig::default(),
        )
    }

    /// Detect if message contains an AI mention using default triggers.
    pub fn detect_ai_mention(message: &str) -> Option<String> {
        domain::detect_ai_mention(message, &domain::default_triggers())
            .map(|mention| mention.trigger)
    }

    /// Extract the actual question/prompt from the message using default triggers.
    pub fn extract_prompt(message: &str) -> String {
        domain::extract_prompt(message, &domain::default_triggers())
    }

    /// Detect if message contains an AI mention using this service's configured triggers.
    pub fn detect_message_mention(&self, message: &str) -> Option<AiMention> {
        domain::detect_ai_mention(message, &self.config.triggers)
    }

    /// Handle an AI message request.
    pub async fn handle_message(
        &self,
        room_id: &str,
        sender_id: &str,
        original_message: &str,
    ) -> Option<AiResponse> {
        let context_messages = self.assemble_context(room_id).await;
        let provider = self.provider_registry.get_default().await?;

        debug!(
            room_id = %room_id,
            sender_id = %sender_id,
            context_messages = context_messages.len(),
            provider = provider.name(),
            "Processing AI message"
        );

        let prompt = self.build_prompt(&context_messages, original_message, sender_id);
        let request = GenerateRequest {
            prompt,
            model: None,
            max_tokens: Some(500),
            temperature: Some(0.7),
            metadata: None,
        };

        match provider.generate(request).await {
            Ok(response) => {
                info!(
                    room_id = %room_id,
                    provider = provider.name(),
                    response_len = response.content.len(),
                    "AI response generated"
                );

                Some(AiResponse {
                    content: response.content,
                    agent_name: self.config.default_agent_name.clone(),
                    model: response.model,
                })
            }
            Err(error) => {
                error!(
                    room_id = %room_id,
                    error = %error,
                    "AI provider error"
                );
                None
            }
        }
    }

    /// Add a message to the room's context.
    pub async fn add_to_context(&self, room_id: &str, role: MessageRole, content: String) {
        let message = match role {
            MessageRole::User => ContextMessage::user(content),
            MessageRole::Assistant => ContextMessage::assistant(content),
            MessageRole::System => ContextMessage::system(content),
        };

        if let Err(error) = self
            .context_manager
            .add_message_by_room(room_id, message)
            .await
        {
            warn!(
                room_id = %room_id,
                error = %error,
                "Failed to add message to context"
            );
        }
    }

    async fn assemble_context(&self, room_id: &str) -> Vec<ContextMessage> {
        let messages = self.context_manager.get_context_by_room(room_id).await;
        let recent: Vec<ContextMessage> = messages
            .into_iter()
            .rev()
            .take(self.config.max_context_messages)
            .rev()
            .collect();

        debug!(
            room_id = %room_id,
            message_count = recent.len(),
            "Context assembled"
        );

        recent
    }

    fn build_prompt(
        &self,
        context_messages: &[ContextMessage],
        current_message: &str,
        sender_id: &str,
    ) -> String {
        let mut prompt = self.config.system_prompt.clone();
        prompt.push_str("\n\n");

        if !context_messages.is_empty() {
            prompt.push_str("Recent conversation:\n");
            for msg in context_messages {
                let role = match msg.role {
                    MessageRole::User => "User",
                    MessageRole::Assistant => "AI",
                    MessageRole::System => "System",
                };
                prompt.push_str(&format!("[{}]: {}\n", role, msg.content));
            }
            prompt.push('\n');
        }

        let clean_prompt = domain::extract_prompt(current_message, &self.config.triggers);
        prompt.push_str(&format!("[User {sender_id}]: {clean_prompt}\n"));
        prompt.push_str("\nRespond concisely and helpfully:");

        prompt
    }
}

impl std::fmt::Debug for AiApplication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AiApplication")
            .field("config", &self.config)
            .finish()
    }
}
