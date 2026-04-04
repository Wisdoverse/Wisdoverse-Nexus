//! AI member message handler
//!
//! Handles detection and processing of AI mentions in chat messages.
//! When a message contains @ai or @<agent_name>, this handler:
//! 1. Assembles context from the room's conversation history
//! 2. Calls the AI provider to generate a response
//! 3. Returns the response for broadcasting

use nexis_context::{ContextManager, Message as ContextMessage, MessageRole};
use nexis_runtime::{GenerateRequest, ProviderRegistry};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// AI handler configuration
#[derive(Debug, Clone)]
pub struct AiHandlerConfig {
    /// AI mention triggers (e.g., ["ai", "assistant", "bot"])
    pub triggers: Vec<String>,
    /// Default agent name for responses
    pub default_agent_name: String,
    /// Max context messages to include
    pub max_context_messages: usize,
    /// System prompt prefix for AI
    pub system_prompt: String,
}

impl Default for AiHandlerConfig {
    fn default() -> Self {
        Self {
            triggers: vec!["ai".to_string(), "assistant".to_string()],
            default_agent_name: "ai".to_string(),
            max_context_messages: 20,
            system_prompt:
                "You are a helpful AI assistant in a group chat. Be concise and helpful."
                    .to_string(),
        }
    }
}

/// AI handler for processing @ai mentions
#[derive(Clone)]
pub struct AiHandler {
    context_manager: Arc<ContextManager>,
    provider_registry: Arc<ProviderRegistry>,
    config: AiHandlerConfig,
}

impl AiHandler {
    /// Create a new AI handler
    pub fn new(
        context_manager: Arc<ContextManager>,
        provider_registry: Arc<ProviderRegistry>,
        config: AiHandlerConfig,
    ) -> Self {
        Self {
            context_manager,
            provider_registry,
            config,
        }
    }

    /// Create with default config
    pub fn with_defaults(
        context_manager: Arc<ContextManager>,
        provider_registry: Arc<ProviderRegistry>,
    ) -> Self {
        Self::new(
            context_manager,
            provider_registry,
            AiHandlerConfig::default(),
        )
    }

    /// Detect if message contains an AI mention
    ///
    /// Returns Some(trigger) if found, None otherwise.
    /// Supports formats: @ai, @AI, @assistant, @<agent_name>
    pub fn detect_ai_mention(message: &str) -> Option<String> {
        let lower = message.to_lowercase();

        // Check for @ai or @assistant mentions
        for trigger in &["ai", "assistant"] {
            let pattern = format!("@{}", trigger);
            if lower.contains(&pattern) {
                return Some(trigger.to_string());
            }
        }

        None
    }

    /// Extract the actual question/prompt from the message
    ///
    /// Removes the @ai mention and returns the clean prompt
    pub fn extract_prompt(message: &str) -> String {
        let lower = message.to_lowercase();

        // Remove @ai or @assistant mentions
        let cleaned = lower
            .replace("@ai", "")
            .replace("@assistant", "")
            .trim()
            .to_string();

        cleaned
    }

    /// Handle an AI message request
    ///
    /// This is the main entry point for processing @ai mentions.
    /// Returns the AI response, or None if no provider available.
    pub async fn handle_message(
        &self,
        room_id: &str,
        sender_id: &str,
        original_message: &str,
    ) -> Option<AiResponse> {
        // Get context for this room
        let context_messages = self.assemble_context(room_id).await;

        // Get the default provider
        let provider = self.provider_registry.get_default().await?;

        debug!(
            room_id = %room_id,
            sender_id = %sender_id,
            context_messages = context_messages.len(),
            provider = provider.name(),
            "Processing AI message"
        );

        // Build the prompt with context
        let prompt = self.build_prompt(&context_messages, original_message, sender_id);

        // Call the AI provider
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
            Err(e) => {
                error!(
                    room_id = %room_id,
                    error = %e,
                    "AI provider error"
                );
                None
            }
        }
    }

    /// Assemble context from the room's conversation history
    async fn assemble_context(&self, room_id: &str) -> Vec<ContextMessage> {
        let messages = self.context_manager.get_context_by_room(room_id).await;

        // Take the most recent N messages
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

    /// Build the prompt with context
    fn build_prompt(
        &self,
        context_messages: &[ContextMessage],
        current_message: &str,
        sender_id: &str,
    ) -> String {
        let mut prompt = self.config.system_prompt.clone();
        prompt.push_str("\n\n");

        // Add conversation context
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

        // Add current message
        let clean_prompt = Self::extract_prompt(current_message);
        prompt.push_str(&format!("[User {}]: {}\n", sender_id, clean_prompt));
        prompt.push_str("\nRespond concisely and helpfully:");

        prompt
    }

    /// Add a message to the room's context
    pub async fn add_to_context(&self, room_id: &str, role: MessageRole, content: String) {
        let message = match role {
            MessageRole::User => ContextMessage::user(content),
            MessageRole::Assistant => ContextMessage::assistant(content),
            MessageRole::System => ContextMessage::system(content),
        };

        if let Err(e) = self
            .context_manager
            .add_message_by_room(room_id, message)
            .await
        {
            warn!(
                room_id = %room_id,
                error = %e,
                "Failed to add message to context"
            );
        }
    }
}

impl std::fmt::Debug for AiHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AiHandler")
            .field("config", &self.config)
            .finish()
    }
}

/// AI response
#[derive(Debug, Clone)]
pub struct AiResponse {
    /// The response content
    pub content: String,
    /// The agent name to display
    pub agent_name: String,
    /// The model used
    pub model: Option<String>,
}

impl AiResponse {
    /// Format the response as a chat message
    pub fn to_chat_message(&self) -> String {
        format!("[{}]: {}", self.agent_name, self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ai_mention() {
        assert!(AiHandler::detect_ai_mention("@ai hello").is_some());
        assert!(AiHandler::detect_ai_mention("@AI what's up").is_some());
        assert!(AiHandler::detect_ai_mention("Hey @assistant help").is_some());
        assert!(AiHandler::detect_ai_mention("@ASSISTANT please").is_some());
        assert!(AiHandler::detect_ai_mention("hello @ai").is_some());
    }

    #[test]
    fn test_no_ai_mention() {
        assert!(AiHandler::detect_ai_mention("hello world").is_none());
        assert!(AiHandler::detect_ai_mention("email@test.com").is_none());
        assert!(AiHandler::detect_ai_mention("someone@example.com").is_none());
    }

    #[test]
    fn test_extract_prompt() {
        assert_eq!(AiHandler::extract_prompt("@ai hello"), "hello");
        assert_eq!(AiHandler::extract_prompt("@AI what's up"), "what's up");
        assert_eq!(
            AiHandler::extract_prompt("Hey @assistant help me"),
            "hey  help me"
        );
        assert_eq!(AiHandler::extract_prompt("hello @ai"), "hello");
    }

    #[test]
    fn test_ai_response_format() {
        let response = AiResponse {
            content: "Hello! How can I help?".to_string(),
            agent_name: "ai".to_string(),
            model: Some("gpt-4o-mini".to_string()),
        };

        assert_eq!(response.to_chat_message(), "[ai]: Hello! How can I help?");
    }
}
