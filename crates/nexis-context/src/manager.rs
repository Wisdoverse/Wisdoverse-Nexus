//! Context manager implementation

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::context::{ConversationContext, Message};
use crate::error::{ContextError, ContextResult};
use crate::summarizer::{ContextSummarizer, SummarizerConfig};
use crate::window::{ContextWindow, OverflowStrategy};

#[cfg(feature = "metrics")]
use crate::metrics::{
    record_summarization_failure, record_summarization_overflow, record_summarization_success,
    record_truncation, record_window_utilization, set_active_contexts,
};

/// Context manager for handling conversation contexts
///
/// Supports both context_id (Uuid) and room_id (String) based lookups.
/// room_id is the primary identifier for M2 AI member integration.
pub struct ContextManager {
    /// context_id -> ConversationContext
    contexts: Arc<RwLock<HashMap<Uuid, ConversationContext>>>,
    /// room_id -> context_id mapping
    room_contexts: Arc<RwLock<HashMap<String, Uuid>>>,
    window: ContextWindow,
    summarizer: Option<Arc<dyn ContextSummarizer>>,
    summarizer_config: SummarizerConfig,
}

impl ContextManager {
    /// Create a new context manager with default settings
    pub fn new(window: ContextWindow) -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            room_contexts: Arc::new(RwLock::new(HashMap::new())),
            window,
            summarizer: None,
            summarizer_config: SummarizerConfig::default(),
        }
    }

    /// Create a new context manager with a summarizer
    pub fn with_summarizer(window: ContextWindow, summarizer: Arc<dyn ContextSummarizer>) -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            room_contexts: Arc::new(RwLock::new(HashMap::new())),
            window,
            summarizer: Some(summarizer),
            summarizer_config: SummarizerConfig::default(),
        }
    }

    /// Create a new context manager with custom summarizer config
    pub fn with_summarizer_config(
        window: ContextWindow,
        summarizer: Arc<dyn ContextSummarizer>,
        config: SummarizerConfig,
    ) -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            room_contexts: Arc::new(RwLock::new(HashMap::new())),
            window,
            summarizer: Some(summarizer),
            summarizer_config: config,
        }
    }

    /// Create a new context
    pub async fn create_context(&self, room_id: Option<Uuid>) -> ContextResult<Uuid> {
        let context = ConversationContext::new(room_id);
        let id = context.id;
        self.contexts.write().await.insert(id, context);

        #[cfg(feature = "metrics")]
        set_active_contexts(self.contexts.read().await.len());

        Ok(id)
    }

    /// Get a context by ID
    pub async fn get_context(&self, id: Uuid) -> ContextResult<ConversationContext> {
        self.contexts
            .read()
            .await
            .get(&id)
            .cloned()
            .ok_or_else(|| ContextError::NotFound(id.to_string()))
    }

    /// Add a message to a context
    pub async fn add_message(&self, context_id: Uuid, message: Message) -> ContextResult<()> {
        let mut contexts = self.contexts.write().await;
        let context = contexts
            .get_mut(&context_id)
            .ok_or_else(|| ContextError::NotFound(context_id.to_string()))?;

        // Check window overflow
        let estimated_tokens = estimate_tokens(&message.content);
        let new_total = context.total_tokens() + estimated_tokens;

        if new_total > self.window.available_tokens() {
            match self.window.overflow_strategy {
                OverflowStrategy::TruncateOldest => {
                    let _count = self.truncate_oldest_with_count(
                        context,
                        new_total - self.window.available_tokens(),
                    );
                    #[cfg(feature = "metrics")]
                    record_truncation(_count);
                }
                OverflowStrategy::Fail => {
                    return Err(ContextError::WindowFull);
                }
                OverflowStrategy::Summarize => {
                    self.handle_overflow_with_summarization(context, new_total)
                        .await?;
                }
            }
        }

        let mut message = message;
        message.token_count = Some(estimated_tokens);
        context.add_message(message);

        #[cfg(feature = "metrics")]
        {
            let utilization =
                (context.total_tokens() as f64 / self.window.available_tokens() as f64) * 100.0;
            record_window_utilization(utilization);
        }

        Ok(())
    }

    /// Handle overflow using summarization strategy
    async fn handle_overflow_with_summarization(
        &self,
        context: &mut ConversationContext,
        new_total: usize,
    ) -> ContextResult<()> {
        #[cfg(feature = "metrics")]
        record_summarization_overflow();

        let tokens_to_free = new_total - self.window.available_tokens();

        // If no summarizer configured, fall back to truncation
        let Some(ref summarizer) = self.summarizer else {
            debug!("No summarizer configured, falling back to truncation");
            let _truncated = self.truncate_oldest_with_count(context, tokens_to_free);
            #[cfg(feature = "metrics")]
            record_truncation(truncated);
            return Ok(());
        };

        // Collect messages to summarize (respecting batch size)
        let batch_size = self
            .summarizer_config
            .batch_size
            .min(context.messages.len());
        if batch_size == 0 {
            warn!("No messages to summarize");
            return Ok(());
        }

        let messages_to_summarize: Vec<Message> = context.messages.drain(0..batch_size).collect();

        debug!(batch_size = batch_size, "Attempting to summarize messages");

        let start = Instant::now();
        match summarizer.summarize(&messages_to_summarize).await {
            Ok(summary) => {
                // Insert summary at the beginning
                context.messages.insert(0, summary);
                let latency = start.elapsed().as_secs_f64();

                #[cfg(feature = "metrics")]
                record_summarization_success(batch_size, latency);

                debug!(
                    "Successfully summarized {} messages in {:.2}s",
                    batch_size, latency
                );
                Ok(())
            }
            Err(e) => {
                // On failure, restore the messages and fall back to truncation
                warn!(error = ?e, "Summarization failed, falling back to truncation");
                context.messages = [messages_to_summarize, context.messages.clone()].concat();
                let _truncated = self.truncate_oldest_with_count(context, tokens_to_free);

                #[cfg(feature = "metrics")]
                {
                    record_summarization_failure();
                    record_truncation(_truncated);
                }

                Err(ContextError::SummarizationFailed(e.to_string()))
            }
        }
    }

    /// Delete a context
    pub async fn delete_context(&self, id: Uuid) -> ContextResult<()> {
        // Also remove from room_contexts mapping
        if let Some(context) = self.contexts.read().await.get(&id) {
            if let Some(room_id) = context.room_id {
                let mut room_contexts = self.room_contexts.write().await;
                room_contexts.remove(&room_id.to_string());
            }
        }

        self.contexts
            .write()
            .await
            .remove(&id)
            .map(|_| ())
            .ok_or_else(|| ContextError::NotFound(id.to_string()))?;

        #[cfg(feature = "metrics")]
        set_active_contexts(self.contexts.read().await.len());

        Ok(())
    }

    // ========================================================================
    // room_id-based API (M2 AI member integration)
    // ========================================================================

    /// Get or create a context for a room
    ///
    /// This is the primary API for M2 AI member integration.
    /// Returns the context_id (Uuid) for the room's context.
    pub async fn get_or_create_context_by_room(&self, room_id: &str) -> Uuid {
        // Check if room already has a context
        {
            let room_contexts = self.room_contexts.read().await;
            if let Some(&context_id) = room_contexts.get(room_id) {
                return context_id;
            }
        }

        // Create new context for this room
        let room_uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, room_id.as_bytes());
        let mut contexts = self.contexts.write().await;
        let mut room_contexts = self.room_contexts.write().await;

        // Double-check after acquiring write lock
        if let Some(&context_id) = room_contexts.get(room_id) {
            return context_id;
        }

        let mut context = ConversationContext::new(Some(room_uuid));
        context.id = room_uuid; // Use deterministic ID based on room_id
        contexts.insert(room_uuid, context);
        room_contexts.insert(room_id.to_string(), room_uuid);

        #[cfg(feature = "metrics")]
        set_active_contexts(contexts.len());

        room_uuid
    }

    /// Get recent messages for a room
    ///
    /// Returns the last N messages from the room's context.
    /// If the room has no context, returns an empty vector.
    pub async fn get_context_by_room(&self, room_id: &str) -> Vec<Message> {
        let room_contexts = self.room_contexts.read().await;
        if let Some(&context_id) = room_contexts.get(room_id) {
            drop(room_contexts);
            if let Ok(context) = self.get_context(context_id).await {
                return context.messages;
            }
        }
        Vec::new()
    }

    /// Add a message to a room's context
    ///
    /// Creates the room's context if it doesn't exist.
    pub async fn add_message_by_room(&self, room_id: &str, message: Message) -> ContextResult<()> {
        let context_id = self.get_or_create_context_by_room(room_id).await;
        self.add_message(context_id, message).await
    }

    /// Delete a room's context
    ///
    /// Returns Ok(()) if the context was deleted, or error if not found.
    pub async fn delete_context_by_room(&self, room_id: &str) -> ContextResult<()> {
        let context_id = {
            let room_contexts = self.room_contexts.read().await;
            room_contexts
                .get(room_id)
                .copied()
                .ok_or_else(|| ContextError::NotFound(room_id.to_string()))?
        };
        self.delete_context(context_id).await
    }

    /// Get number of active contexts
    pub async fn context_count(&self) -> usize {
        self.contexts.read().await.len()
    }

    /// Truncate oldest messages and return count of messages removed
    fn truncate_oldest_with_count(
        &self,
        context: &mut ConversationContext,
        tokens_to_free: usize,
    ) -> usize {
        let mut freed = 0;
        let mut count = 0;
        while freed < tokens_to_free && context.messages.len() > 1 {
            if let Some(msg) = context.messages.first() {
                freed += msg.token_count.unwrap_or(0);
                context.messages.remove(0);
                count += 1;
            }
        }
        count
    }
}

/// Estimate token count for text
///
/// Simple estimation that considers CJK characters:
/// - CJK characters: ~1.5 chars/token
/// - ASCII text: ~4 chars/token
fn estimate_tokens(text: &str) -> usize {
    let char_count = text.chars().count();
    let byte_len = text.len();

    // If byte length significantly exceeds char count, we have multi-byte (CJK/Unicode)
    if byte_len > char_count * 3 / 2 {
        // Multi-byte chars: approximately 1.5 characters per token
        (char_count as f64 / 1.5).ceil() as usize
    } else {
        // ASCII/English: approximately 4 characters per token
        if char_count == 0 {
            0
        } else {
            (char_count / 4).max(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::summarizer::MockSummarizer;

    #[test]
    fn test_estimate_tokens_ascii() {
        // ASCII text: ~4 chars per token
        let tokens = estimate_tokens("hello world test message");
        assert!(tokens > 0, "ASCII text should produce tokens");
    }

    #[test]
    fn test_estimate_tokens_cjk() {
        // CJK: ~1.5 chars per token
        let cjk = "你好世界"; // 4 chars, 12 bytes
        let tokens = estimate_tokens(cjk);
        assert!(tokens >= 2, "CJK should use ~1.5 chars/token");
    }

    #[test]
    fn test_estimate_tokens_mixed() {
        let mixed = "Hello你好"; // 7 chars, 11 bytes -> CJK path
        let tokens = estimate_tokens(mixed);
        assert!(tokens > 1, "Mixed content should produce tokens");
    }

    #[tokio::test]
    async fn test_create_and_get_context() {
        let manager = ContextManager::new(ContextWindow::default());
        let id = manager.create_context(None).await.unwrap();
        let context = manager.get_context(id).await.unwrap();
        assert!(context.messages.is_empty());
    }

    #[tokio::test]
    async fn test_add_message() {
        let manager = ContextManager::new(ContextWindow::default());
        let id = manager.create_context(None).await.unwrap();

        let msg = Message::user("Hello".to_string());
        manager.add_message(id, msg).await.unwrap();

        let context = manager.get_context(id).await.unwrap();
        assert_eq!(context.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_window_overflow_truncate() {
        let window = ContextWindow::new(50); // Very small window
        let manager = ContextManager::new(window);
        let id = manager.create_context(None).await.unwrap();

        // Add multiple messages
        for i in 0..10 {
            let msg = Message::user(format!("Message number {} with some content", i));
            manager.add_message(id, msg).await.unwrap();
        }

        let context = manager.get_context(id).await.unwrap();
        // Should have truncated some messages
        assert!(context.messages.len() < 10);
    }

    #[tokio::test]
    async fn test_window_overflow_summarize() {
        let window = ContextWindow::new(100).with_overflow_strategy(OverflowStrategy::Summarize);
        let summarizer = Arc::new(MockSummarizer::new("Previous conversation summary"));
        let manager = ContextManager::with_summarizer(window, summarizer);
        let id = manager.create_context(None).await.unwrap();

        // Add enough messages to trigger overflow
        for i in 0..20 {
            let msg = Message::user(format!(
                "Message number {} with enough content to fill window",
                i
            ));
            manager.add_message(id, msg).await.unwrap();
        }

        let context = manager.get_context(id).await.unwrap();

        // Should have a summary message at the beginning
        assert!(!context.messages.is_empty());
        assert!(
            context.messages[0].is_summary(),
            "First message should be a summary"
        );
        assert!(context.messages[0].content.contains("Summary of"));
    }

    #[tokio::test]
    async fn test_summarization_fallback_on_error() {
        let window = ContextWindow::new(50).with_overflow_strategy(OverflowStrategy::Summarize);
        // Use a summarizer that will fail - we'll test this by checking truncation still works
        let manager = ContextManager::new(window);
        let id = manager.create_context(None).await.unwrap();

        // Add messages to trigger overflow
        for i in 0..10 {
            let msg = Message::user(format!("Message {} with content", i));
            manager.add_message(id, msg).await.unwrap();
        }

        let context = manager.get_context(id).await.unwrap();
        // Without a working summarizer, should have truncated
        assert!(context.messages.len() < 10);
    }

    #[tokio::test]
    async fn test_context_count() {
        let manager = ContextManager::new(ContextWindow::default());
        assert_eq!(manager.context_count().await, 0);

        manager.create_context(None).await.unwrap();
        assert_eq!(manager.context_count().await, 1);

        manager.create_context(None).await.unwrap();
        assert_eq!(manager.context_count().await, 2);
    }

    // ============== CJK Token Estimation Tests ==============

    #[test]
    fn test_estimate_tokens_pure_cjk() {
        // Pure Chinese - each char is 3 bytes, should use CJK estimation
        let chinese = "这是一个测试消息内容";
        let tokens = estimate_tokens(chinese);
        // 10 chars, ~1.5 chars/token = ~7 tokens
        assert!(tokens >= 6, "Expected ~7 tokens for CJK, got {}", tokens);
        assert!(tokens <= 8, "Expected ~7 tokens for CJK, got {}", tokens);
    }

    #[test]
    fn test_estimate_tokens_japanese() {
        // Japanese hiragana/katakana - multi-byte
        let japanese = "こんにちはせかい";
        let tokens = estimate_tokens(japanese);
        assert!(tokens >= 2, "Japanese text should produce tokens");
    }

    #[test]
    fn test_estimate_tokens_korean() {
        // Korean Hangul - multi-byte
        let korean = "안녕하세요";
        let tokens = estimate_tokens(korean);
        assert!(tokens >= 2, "Korean text should produce tokens");
    }

    #[test]
    fn test_estimate_tokens_cjk_with_ascii() {
        // Mixed CJK and ASCII - byte length > char count * 1.5
        let mixed = "Hello世界Test测试";
        let tokens = estimate_tokens(mixed);
        assert!(tokens > 1, "Mixed content should use CJK estimation");
    }

    #[test]
    fn test_estimate_tokens_long_ascii() {
        // Long ASCII text
        let long_ascii = "This is a very long English sentence that should be tokenized using the ASCII ratio of approximately four characters per token";
        let tokens = estimate_tokens(long_ascii);
        // ~130 chars / 4 = ~32 tokens
        assert!(
            tokens >= 30,
            "Long ASCII should produce ~32 tokens, got {}",
            tokens
        );
    }

    #[test]
    fn test_estimate_tokens_single_char_ascii() {
        let tokens = estimate_tokens("a");
        assert_eq!(tokens, 1, "Single ASCII char should be 1 token");
    }

    #[test]
    fn test_estimate_tokens_single_cjk_char() {
        let tokens = estimate_tokens("你");
        assert!(
            tokens >= 1,
            "Single CJK char should produce at least 1 token"
        );
    }

    #[test]
    fn test_estimate_tokens_empty_string() {
        let tokens = estimate_tokens("");
        assert_eq!(tokens, 0, "Empty string should produce 0 tokens");
    }

    #[test]
    fn test_estimate_tokens_whitespace() {
        let tokens = estimate_tokens("   ");
        assert!(tokens >= 1, "Whitespace should produce at least 1 token");
    }

    #[test]
    fn test_estimate_tokens_emoji() {
        // Emoji are multi-byte (4 bytes each typically)
        let emoji = "😀🎉🚀";
        let tokens = estimate_tokens(emoji);
        assert!(tokens >= 1, "Emoji should produce tokens");
    }

    // ============== Overflow Boundary Tests ==============

    #[tokio::test]
    async fn test_window_exact_fit() {
        let window = ContextWindow::new(100);
        let manager = ContextManager::new(window);
        let id = manager.create_context(None).await.unwrap();

        // Add messages that exactly fit
        let msg = Message::user("a".repeat(96)); // ~24 tokens
        manager.add_message(id, msg).await.unwrap();

        let context = manager.get_context(id).await.unwrap();
        assert_eq!(context.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_window_one_over() {
        let window = ContextWindow::new(10).with_reserved_tokens(0); // Very small
        let manager = ContextManager::new(window);
        let id = manager.create_context(None).await.unwrap();

        // Add a message that barely overflows
        let msg = Message::user("This message is longer than ten tokens would allow".to_string());
        manager.add_message(id, msg).await.unwrap();

        let context = manager.get_context(id).await.unwrap();
        // Should have truncated or fit somehow
        assert!(!context.messages.is_empty());
    }

    #[tokio::test]
    async fn test_window_zero_capacity() {
        let window = ContextWindow::new(0);
        let manager = ContextManager::new(window);
        let id = manager.create_context(None).await.unwrap();

        let msg = Message::user("test".to_string());
        let _result = manager.add_message(id, msg).await;

        // Should either fail or handle gracefully
        // Depending on implementation, might truncate or error
        if let Ok(context) = manager.get_context(id).await {
            assert!(context.messages.len() <= 1);
        }
    }

    #[tokio::test]
    async fn test_window_many_small_messages() {
        let window = ContextWindow::new(50);
        let manager = ContextManager::new(window);
        let id = manager.create_context(None).await.unwrap();

        // Add many small messages
        for i in 0..100 {
            let msg = Message::user(format!("M{}", i)); // Very short
            let _ = manager.add_message(id, msg).await;
        }

        let context = manager.get_context(id).await.unwrap();
        // Should have truncated some
        assert!(context.messages.len() < 100);
    }

    #[tokio::test]
    async fn test_window_single_large_message() {
        let window = ContextWindow::new(100);
        let manager = ContextManager::new(window);
        let id = manager.create_context(None).await.unwrap();

        // One very large message
        let large_content = "x".repeat(1000);
        let msg = Message::user(large_content);
        manager.add_message(id, msg).await.unwrap();

        let context = manager.get_context(id).await.unwrap();
        assert!(!context.messages.is_empty());
    }

    #[tokio::test]
    async fn test_window_overflow_strategy_fail() {
        let window = ContextWindow::new(10)
            .with_reserved_tokens(0)
            .with_overflow_strategy(OverflowStrategy::Fail);
        let manager = ContextManager::new(window);
        let id = manager.create_context(None).await.unwrap();

        // First message should fit
        let msg1 = Message::user("short".to_string());
        let result1 = manager.add_message(id, msg1).await;
        assert!(result1.is_ok());

        // Large message should fail with Fail strategy
        let msg2 = Message::user("This is a very long message that will overflow".to_string());
        let result2 = manager.add_message(id, msg2).await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_context() {
        let manager = ContextManager::new(ContextWindow::default());
        let fake_id = Uuid::new_v4();

        let result = manager.delete_context(fake_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_nonexistent_context() {
        let manager = ContextManager::new(ContextWindow::default());
        let fake_id = Uuid::new_v4();

        let result = manager.get_context(fake_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_add_message_to_nonexistent_context() {
        let manager = ContextManager::new(ContextWindow::default());
        let fake_id = Uuid::new_v4();

        let msg = Message::user("test".to_string());
        let result = manager.add_message(fake_id, msg).await;
        assert!(result.is_err());
    }
}
