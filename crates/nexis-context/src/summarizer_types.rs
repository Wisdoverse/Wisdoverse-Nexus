//! Core types for AI summarization

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// AI Provider trait for abstracting different LLM backends
#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    /// Generate completion from the AI provider
    async fn complete(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;

    /// Get provider name
    fn provider_name(&self) -> &str;
}

/// Message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub sender_name: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

/// Role of a message sender
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Generated summary of conversation messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    /// Main summary text
    pub summary: String,
    /// Key discussion points
    pub key_points: Vec<String>,
    /// Action items extracted from conversation
    pub action_items: Vec<String>,
    /// Participants in the conversation
    pub participants: Vec<String>,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Source message IDs that were summarized
    pub source_message_ids: Vec<String>,
    /// Token count of original messages
    pub original_tokens: usize,
    /// Token count of summary
    pub summary_tokens: usize,
}

impl Summary {
    pub fn new(summary: String) -> Self {
        Self {
            summary,
            key_points: Vec::new(),
            action_items: Vec::new(),
            participants: Vec::new(),
            generated_at: Utc::now(),
            source_message_ids: Vec::new(),
            original_tokens: 0,
            summary_tokens: 0,
        }
    }
}

/// Meeting information for generating notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meeting {
    pub id: String,
    pub room_id: String,
    pub title: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub participants: Vec<String>,
    pub messages: Vec<Message>,
    pub agenda: Option<Vec<String>>,
}

/// Generated meeting notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingNotes {
    /// Meeting title
    pub title: String,
    /// Summary of the meeting
    pub summary: String,
    /// Key discussion points
    pub key_points: Vec<String>,
    /// Decisions made
    pub decisions: Vec<String>,
    /// Action items with assignees
    pub action_items: Vec<ActionItem>,
    /// Participants
    pub participants: Vec<String>,
    /// Meeting duration in minutes
    pub duration_minutes: u32,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Next meeting suggestion (if any)
    pub next_meeting: Option<String>,
}

/// Action item with assignee and optional due date
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub description: String,
    pub assignee: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub priority: Option<Priority>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    High,
    Medium,
    Low,
}
