//! Communication primitives for agent-to-agent messaging.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent::AgentId;

/// Message identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(Uuid);

impl MessageId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

/// Message category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MessageType {
    #[default]
    Request,
    Response,
    Event,
    Error,
}

/// Payload container for protocol-level messages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum Payload {
    Text(String),
    Json(serde_json::Value),
    Binary(Vec<u8>),
    Empty,
}

impl Default for Payload {
    fn default() -> Self {
        Self::Empty
    }
}

/// Error details included with error messages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Authentication context attached to a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AuthContext {
    pub subject: Option<String>,
    pub scopes: Vec<String>,
    pub token_id: Option<String>,
}

/// Core A2A message payload and metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub message_id: MessageId,
    pub message_type: MessageType,
    pub payload: Payload,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<MessageId>,
}

impl Message {
    pub fn new(message_type: MessageType, payload: Payload) -> Self {
        Self {
            message_id: MessageId::new(),
            message_type,
            payload,
            timestamp: Utc::now(),
            correlation_id: None,
        }
    }
}

/// Wire envelope including routing and optional auth/error context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    pub sender: AgentId,
    pub recipient: AgentId,
    pub message: Message,
    pub auth: Option<AuthContext>,
    pub error: Option<ErrorInfo>,
}
