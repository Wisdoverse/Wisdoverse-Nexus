//! WebSocket authentication module
//!
//! Implements first-message authentication pattern for WebSocket connections.
//! This module ensures tokens are never exposed in query parameters or logs.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::auth::{AuthError, JwtConfig};

/// Authentication timeout in seconds
pub const AUTH_TIMEOUT_SECS: u64 = 10;

/// Authentication timeout duration
pub const AUTH_TIMEOUT: Duration = Duration::from_secs(AUTH_TIMEOUT_SECS);

/// Client-to-server WebSocket messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Authentication message (must be the first message after connection)
    Auth {
        /// JWT token (format: "Bearer xxx")
        token: String,
    },
    /// Heartbeat/ping message
    Heartbeat {
        /// Optional timestamp
        #[serde(default)]
        timestamp: Option<i64>,
    },
    /// Join a room
    JoinRoom {
        /// Room ID to join
        room_id: String,
    },
    /// Leave a room
    LeaveRoom {
        /// Room ID to leave
        room_id: String,
    },
    /// Send a message to a room
    SendMessage {
        /// Target room ID
        room_id: String,
        /// Message content
        content: String,
        /// Optional reply-to message ID
        #[serde(default)]
        reply_to: Option<String>,
    },
}

/// Server-to-client WebSocket messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Authentication successful
    AuthSuccess {
        /// Authenticated member ID
        member_id: String,
        /// Member type (human/ai)
        member_type: String,
    },
    /// Authentication failed
    AuthError {
        /// Error message
        message: String,
        /// Error code
        #[serde(default)]
        code: Option<String>,
    },
    /// Authentication required (sent when non-auth message received before auth)
    AuthRequired {
        /// Message explaining auth is required
        message: String,
    },
    /// Heartbeat response
    HeartbeatAck {
        /// Echoed timestamp
        #[serde(default)]
        timestamp: Option<i64>,
    },
    /// Room joined successfully
    RoomJoined {
        /// Room ID
        room_id: String,
    },
    /// Room left successfully
    RoomLeft {
        /// Room ID
        room_id: String,
    },
    /// New message in room
    NewMessage {
        /// Room ID
        room_id: String,
        /// Message ID
        message_id: String,
        /// Sender member ID
        sender_id: String,
        /// Message content
        content: String,
        /// Reply-to message ID
        #[serde(default)]
        reply_to: Option<String>,
        /// Timestamp
        timestamp: i64,
    },
    /// Error message
    Error {
        /// Error message
        message: String,
        /// Error code
        #[serde(default)]
        code: Option<String>,
    },
}

/// WebSocket connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Waiting for authentication
    Unauthenticated,
    /// Authenticated and ready
    Authenticated,
}

/// Authenticated session information
#[derive(Debug, Clone)]
pub struct AuthenticatedSession {
    /// Member ID
    pub member_id: String,
    /// Member type (human/ai)
    pub member_type: String,
}

/// Result of WebSocket message processing
#[derive(Debug)]
pub enum MessageResult {
    /// Message processed successfully, response to send
    Response(ServerMessage),
    /// Authentication completed
    Authenticated(AuthenticatedSession),
    /// Connection should be closed
    CloseConnection,
    /// No response needed
    NoResponse,
}

/// WebSocket authenticator
pub struct WebSocketAuthenticator {
    /// JWT configuration
    jwt_config: JwtConfig,
}

impl WebSocketAuthenticator {
    /// Create a new WebSocket authenticator
    pub fn new(jwt_config: JwtConfig) -> Self {
        Self { jwt_config }
    }

    /// Create with environment configuration
    pub fn from_env() -> Self {
        let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
            let env = std::env::var("NEXIS_ENV").unwrap_or_default();
            if env == "production" {
                panic!("JWT_SECRET must be set in production environment");
            }
            tracing::warn!("Using default JWT secret. DO NOT use in production!");
            "dev_only_secret_change_in_production".to_string()
        });

        let issuer = std::env::var("JWT_ISSUER").unwrap_or_else(|_| "nexis".to_string());
        let audience = std::env::var("JWT_AUDIENCE").unwrap_or_else(|_| "nexis".to_string());

        Self::new(JwtConfig::new(&secret, issuer, audience))
    }

    /// Verify a token and return claims
    pub fn verify_token(&self, token: &str) -> Result<crate::auth::Claims, AuthError> {
        // Handle "Bearer " prefix if present
        let token = token.strip_prefix("Bearer ").unwrap_or(token);
        self.jwt_config.verify_token(token)
    }

    /// Process an incoming message
    pub fn process_message(
        &self,
        state: ConnectionState,
        message: &ClientMessage,
    ) -> MessageResult {
        match state {
            ConnectionState::Unauthenticated => {
                match message {
                    ClientMessage::Auth { token } => {
                        match self.verify_token(token) {
                            Ok(claims) => {
                                let session = AuthenticatedSession {
                                    member_id: claims.sub.clone(),
                                    member_type: claims.member_type.clone(),
                                };
                                let response = ServerMessage::AuthSuccess {
                                    member_id: session.member_id.clone(),
                                    member_type: session.member_type.clone(),
                                };
                                // Return both the response and authentication result
                                // The caller should handle both
                                MessageResult::Response(response)
                            }
                            Err(AuthError::TokenExpired) => {
                                MessageResult::Response(ServerMessage::AuthError {
                                    message: "Token has expired".to_string(),
                                    code: Some("TOKEN_EXPIRED".to_string()),
                                })
                            }
                            Err(e) => MessageResult::Response(ServerMessage::AuthError {
                                message: format!("Authentication failed: {}", e),
                                code: Some("AUTH_FAILED".to_string()),
                            }),
                        }
                    }
                    _ => {
                        // Any message other than Auth before authentication
                        MessageResult::Response(ServerMessage::AuthRequired {
                            message:
                                "Authentication required. Please send an 'auth' message first."
                                    .to_string(),
                        })
                    }
                }
            }
            ConnectionState::Authenticated => {
                // After authentication, handle normal messages
                match message {
                    ClientMessage::Auth { .. } => {
                        // Re-authentication not allowed
                        MessageResult::Response(ServerMessage::Error {
                            message: "Already authenticated".to_string(),
                            code: Some("ALREADY_AUTHENTICATED".to_string()),
                        })
                    }
                    ClientMessage::Heartbeat { timestamp } => {
                        MessageResult::Response(ServerMessage::HeartbeatAck {
                            timestamp: *timestamp,
                        })
                    }
                    // Other messages will be handled by the caller
                    _ => MessageResult::NoResponse,
                }
            }
        }
    }
}

/// Parse a text message into a ClientMessage
pub fn parse_client_message(text: &str) -> Result<ClientMessage, serde_json::Error> {
    serde_json::from_str(text)
}

/// Serialize a ServerMessage to text
pub fn serialize_server_message(message: &ServerMessage) -> Result<String, serde_json::Error> {
    serde_json::to_string(message)
}

/// Create a close message with reason
pub fn create_auth_timeout_message() -> String {
    serde_json::to_string(&ServerMessage::AuthError {
        message: format!(
            "Authentication timeout. Connection must authenticate within {} seconds.",
            AUTH_TIMEOUT_SECS
        ),
        code: Some("AUTH_TIMEOUT".to_string()),
    })
    .unwrap_or_else(|_| r#"{"type":"auth_error","message":"Authentication timeout"}"#.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::JwtConfig;

    fn test_config() -> JwtConfig {
        JwtConfig::new(
            "test_secret_key",
            "nexis-test".to_string(),
            "nexis".to_string(),
        )
    }

    fn test_authenticator() -> WebSocketAuthenticator {
        WebSocketAuthenticator::new(test_config())
    }

    #[test]
    fn parse_auth_message() {
        let json = r#"{"type":"auth","token":"Bearer test_token"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::Auth { token } => {
                assert_eq!(token, "Bearer test_token");
            }
            _ => panic!("Expected Auth message"),
        }
    }

    #[test]
    fn parse_heartbeat_message() {
        let json = r#"{"type":"heartbeat","timestamp":1234567890}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::Heartbeat { timestamp } => {
                assert_eq!(timestamp, Some(1234567890));
            }
            _ => panic!("Expected Heartbeat message"),
        }
    }

    #[test]
    fn serialize_auth_success() {
        let msg = ServerMessage::AuthSuccess {
            member_id: "nexis:human:alice@example.com".to_string(),
            member_type: "human".to_string(),
        };
        let json = serialize_server_message(&msg).unwrap();
        assert!(json.contains("auth_success"));
        assert!(json.contains("alice@example.com"));
    }

    #[test]
    fn serialize_auth_error() {
        let msg = ServerMessage::AuthError {
            message: "Invalid token".to_string(),
            code: Some("AUTH_FAILED".to_string()),
        };
        let json = serialize_server_message(&msg).unwrap();
        assert!(json.contains("auth_error"));
        assert!(json.contains("Invalid token"));
    }

    #[test]
    fn authenticator_validates_token() {
        let auth = test_authenticator();
        let token = test_config().generate_token("alice", "human").unwrap();

        let msg = ClientMessage::Auth {
            token: format!("Bearer {}", token),
        };

        match auth.process_message(ConnectionState::Unauthenticated, &msg) {
            MessageResult::Response(ServerMessage::AuthSuccess {
                member_id,
                member_type,
            }) => {
                assert_eq!(member_id, "alice");
                assert_eq!(member_type, "human");
            }
            other => panic!("Expected AuthSuccess, got {:?}", other),
        }
    }

    #[test]
    fn authenticator_rejects_invalid_token() {
        let auth = test_authenticator();
        let msg = ClientMessage::Auth {
            token: "invalid_token".to_string(),
        };

        match auth.process_message(ConnectionState::Unauthenticated, &msg) {
            MessageResult::Response(ServerMessage::AuthError { .. }) => {}
            other => panic!("Expected AuthError, got {:?}", other),
        }
    }

    #[test]
    fn authenticator_requires_auth_before_other_messages() {
        let auth = test_authenticator();
        let msg = ClientMessage::Heartbeat { timestamp: None };

        match auth.process_message(ConnectionState::Unauthenticated, &msg) {
            MessageResult::Response(ServerMessage::AuthRequired { .. }) => {}
            other => panic!("Expected AuthRequired, got {:?}", other),
        }
    }

    #[test]
    fn authenticated_state_heartbeat_returns_ack() {
        let auth = test_authenticator();
        let msg = ClientMessage::Heartbeat {
            timestamp: Some(123),
        };

        match auth.process_message(ConnectionState::Authenticated, &msg) {
            MessageResult::Response(ServerMessage::HeartbeatAck { timestamp }) => {
                assert_eq!(timestamp, Some(123));
            }
            other => panic!("Expected HeartbeatAck, got {:?}", other),
        }
    }

    #[test]
    fn reauthentication_returns_error() {
        let auth = test_authenticator();
        let token = test_config().generate_token("alice", "human").unwrap();
        let msg = ClientMessage::Auth {
            token: format!("Bearer {}", token),
        };

        match auth.process_message(ConnectionState::Authenticated, &msg) {
            MessageResult::Response(ServerMessage::Error { code, .. }) => {
                assert_eq!(code, Some("ALREADY_AUTHENTICATED".to_string()));
            }
            other => panic!("Expected Error, got {:?}", other),
        }
    }
}
