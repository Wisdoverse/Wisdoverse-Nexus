//! Communication primitives for agent-to-agent messaging.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent::AgentId;

/// Message identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(Uuid);

impl MessageId {
    /// Creates a new random message identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Returns the underlying UUID value.
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
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
    /// Request expecting a response.
    #[default]
    Request,
    /// Response to a previous request.
    Response,
    /// One-way event notification.
    Event,
    /// Error response payload.
    Error,
}

/// Payload container for protocol-level messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", content = "data")]
pub enum Payload {
    /// UTF-8 text payload.
    Text(String),
    /// Arbitrary JSON payload.
    Json(serde_json::Value),
    /// Raw binary payload.
    Binary(Vec<u8>),
    /// Empty payload.
    #[default]
    Empty,
}

/// Error details included with error messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Machine-readable error code.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Optional structured error details.
    pub details: Option<serde_json::Value>,
}

/// Authentication context attached to a message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AuthContext {
    /// Subject principal for the authenticated context.
    pub subject: Option<String>,
    /// Authorization scopes granted to this message.
    pub scopes: Vec<String>,
    /// Optional token identifier.
    pub token_id: Option<String>,
}

/// Core A2A message payload and metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Stable identifier for this message.
    pub message_id: MessageId,
    /// Message semantic type.
    pub message_type: MessageType,
    /// Message body.
    pub payload: Payload,
    /// Message creation timestamp in UTC.
    pub timestamp: DateTime<Utc>,
    /// Optional correlation id linking related messages.
    pub correlation_id: Option<MessageId>,
}

impl Message {
    /// Creates a new message with generated id and current UTC timestamp.
    #[must_use]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    /// Sending agent identifier.
    pub sender: AgentId,
    /// Receiving agent identifier.
    pub recipient: AgentId,
    /// Protocol message content.
    pub message: Message,
    /// Optional authentication context.
    pub auth: Option<AuthContext>,
    /// Optional transport-level or protocol-level error details.
    pub error: Option<ErrorInfo>,
}
