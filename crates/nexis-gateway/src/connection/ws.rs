//! WebSocket connection handler with JWT authentication
//!
//! Handles WebSocket upgrade, JWT validation, and connection management.
//! JWT token can be provided via:
//! - Query parameter: `?token=xxx`
//! - First-message authentication: `{"type":"auth","token":"Bearer xxx"}`
//!
//! Note: Query parameter authentication is deprecated due to security concerns
//! (tokens may be logged). Use first-message authentication instead.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::Instant;

use crate::auth::JwtConfig;
use crate::connection::{
    create_auth_timeout_message, parse_client_message, serialize_server_message, ClientMessage,
    ServerMessage, WebSocketAuthenticator, AUTH_TIMEOUT,
};

/// Maximum messages per second allowed per connection
const MAX_MESSAGES_PER_SECOND: u32 = 10;
/// Time window for rate limiting (1 second)
const RATE_LIMIT_WINDOW_SECS: u64 = 1;

/// Query parameters for WebSocket connection
#[derive(Debug, Clone, Deserialize)]
pub struct WebSocketQuery {
    /// JWT token (optional - can also use first-message auth)
    #[serde(default)]
    pub token: Option<String>,
}

/// Active WebSocket connections keyed by user ID
pub type ConnectionMap = Arc<RwLock<HashMap<String, WebSocketSender>>>;

/// WebSocket sender for a connected user
#[derive(Clone)]
pub struct WebSocketSender {
    /// Channel to send messages to this connection
    pub tx: mpsc::Sender<Message>,
    /// Member type (human/ai)
    pub member_type: String,
}

/// Simple token bucket rate limiter for per-connection message rate limiting
#[derive(Debug)]
struct RateLimiter {
    /// Number of messages sent in the current window
    count: u32,
    /// Start of the current time window
    window_start: Instant,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            count: 0,
            window_start: Instant::now(),
        }
    }

    /// Check if a message is allowed under the rate limit
    /// Returns true if allowed, false if rate limited
    fn check_and_increment(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.window_start);

        // Reset counter if window has passed
        if elapsed.as_secs() >= RATE_LIMIT_WINDOW_SECS {
            self.count = 0;
            self.window_start = now;
        }

        if self.count < MAX_MESSAGES_PER_SECOND {
            self.count += 1;
            true
        } else {
            false
        }
    }
}

/// WebSocket connection state shared across handlers
#[derive(Clone)]
pub struct WebSocketState {
    /// Active connections by member_id
    connections: ConnectionMap,
    /// JWT authenticator
    authenticator: Arc<WebSocketAuthenticator>,
    rooms: crate::rooms::RoomApplication,
}

impl Default for WebSocketState {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketState {
    /// Create a new WebSocket state
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            authenticator: Arc::new(WebSocketAuthenticator::from_env()),
            rooms: crate::rooms::RoomApplication::default(),
        }
    }

    /// Create with custom JWT config (for testing)
    pub fn with_jwt_config(jwt_config: JwtConfig) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            authenticator: Arc::new(WebSocketAuthenticator::new(jwt_config)),
            rooms: crate::rooms::RoomApplication::default(),
        }
    }

    /// Use the same application service as the HTTP room routes.
    pub fn with_rooms(mut self, rooms: crate::rooms::RoomApplication) -> Self {
        self.rooms = rooms;
        self
    }

    /// Get the connection map
    pub fn connections(&self) -> &ConnectionMap {
        &self.connections
    }

    /// Get the authenticator
    pub fn authenticator(&self) -> &WebSocketAuthenticator {
        &self.authenticator
    }

    /// Add a connection
    pub async fn add_connection(
        &self,
        member_id: String,
        member_type: String,
        tx: mpsc::Sender<Message>,
    ) {
        let sender = WebSocketSender { tx, member_type };
        let mut connections = self.connections.write().await;

        // If user already has a connection, close the old one
        if let Some(old_sender) = connections.insert(member_id.clone(), sender) {
            tracing::info!(
                member_id = %member_id,
                "Closing previous connection for user"
            );
            let _ = old_sender.tx.try_send(Message::Close(None));
        }

        tracing::info!(
            member_id = %member_id,
            active_connections = connections.len(),
            "WebSocket connection added"
        );
    }

    /// Remove a connection
    pub async fn remove_connection(&self, member_id: &str) {
        let mut connections = self.connections.write().await;
        if connections.remove(member_id).is_some() {
            tracing::info!(
                member_id = %member_id,
                active_connections = connections.len(),
                "WebSocket connection removed"
            );
        }
    }

    /// Send a message to a specific user
    pub async fn send_to_user(&self, member_id: &str, message: Message) -> bool {
        let connections = self.connections.read().await;
        if let Some(sender) = connections.get(member_id) {
            sender.tx.try_send(message).is_ok()
        } else {
            false
        }
    }

    /// Broadcast a message to all connections
    pub async fn broadcast(&self, message: Message) {
        let connections = self.connections.read().await;
        let mut sent = 0;
        let mut failed = 0;

        for (member_id, sender) in connections.iter() {
            if sender.tx.try_send(message.clone()).is_ok() {
                sent += 1;
            } else {
                failed += 1;
                tracing::warn!(member_id = %member_id, "Failed to send broadcast message");
            }
        }

        if sent > 0 || failed > 0 {
            tracing::debug!(sent = sent, failed = failed, "Broadcast complete");
        }
    }

    /// Get connection count
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }
}

/// Handle WebSocket upgrade with explicit state (for use without State extractor)
pub async fn websocket_upgrade_with_state(
    ws: WebSocketUpgrade,
    Query(query): Query<WebSocketQuery>,
    state: WebSocketState,
) -> Response {
    if cfg!(feature = "multi-tenant") {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "M1 supports default single-tenant features only",
        )
            .into_response();
    }
    // Check if token provided in query
    if let Some(token) = query.token {
        tracing::warn!(
            "DEPRECATION WARNING: WebSocket auth via query parameter is deprecated. \
             Use first-message authentication instead."
        );

        // Validate token
        match state.authenticator.verify_token(&token) {
            Ok(claims) => {
                // Pre-authenticated via query token
                ws.on_upgrade(move |socket| handle_connection(socket, state, Some(claims)))
            }
            Err(e) => {
                // Invalid token - reject connection
                tracing::warn!("WebSocket connection rejected: invalid token - {}", e);
                (StatusCode::UNAUTHORIZED, "Invalid or expired token").into_response()
            }
        }
    } else {
        // No token in query - require first-message authentication
        ws.on_upgrade(move |socket| handle_connection(socket, state, None))
    }
}

/// Authenticate, subscribe to rooms, and dispatch through the shared application service.
async fn handle_connection(
    socket: WebSocket,
    state: WebSocketState,
    preauthenticated: Option<crate::auth::Claims>,
) {
    use futures::{SinkExt, StreamExt};
    use tokio::time::{timeout, Duration};

    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<Message>(256);
    let mut writer = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            let closing = matches!(message, Message::Close(_));
            if !matches!(
                timeout(Duration::from_secs(5), sender.send(message)).await,
                Ok(Ok(()))
            ) || closing
            {
                break;
            }
        }
    });

    let authenticate = async {
        if let Some(claims) = preauthenticated {
            return Some(claims);
        }
        while let Some(Ok(message)) = receiver.next().await {
            match message {
                Message::Text(text) => match parse_client_message(&text) {
                    Ok(ClientMessage::Auth { token }) => {
                        match state.authenticator.verify_token(&token) {
                            Ok(claims) => return Some(claims),
                            Err(error) => {
                                let code = if matches!(error, crate::auth::AuthError::TokenExpired)
                                {
                                    "TOKEN_EXPIRED"
                                } else {
                                    "AUTH_FAILED"
                                };
                                send_response(
                                    &tx,
                                    &ServerMessage::AuthError {
                                        message: "Invalid or expired credentials".to_string(),
                                        code: Some(code.to_string()),
                                    },
                                )
                                .await;
                                return None;
                            }
                        }
                    }
                    _ => {
                        send_response(
                            &tx,
                            &ServerMessage::AuthRequired {
                                message: "Send an auth message first".to_string(),
                            },
                        )
                        .await;
                        return None;
                    }
                },
                Message::Close(_) => return None,
                _ => {}
            }
        }
        None
    };

    let claims = match timeout(AUTH_TIMEOUT, authenticate).await {
        Ok(claims) => claims,
        Err(_) => {
            let _ = tx
                .send(Message::Text(create_auth_timeout_message().into()))
                .await;
            None
        }
    };

    if let Some(claims) = claims {
        let member_id = claims.sub;
        let mut events = state.rooms.subscribe();
        let mut joined_rooms = HashSet::new();
        let mut rate_limiter = RateLimiter::new();
        state
            .add_connection(member_id.clone(), claims.member_type.clone(), tx.clone())
            .await;
        send_response(
            &tx,
            &ServerMessage::AuthSuccess {
                member_id: member_id.clone(),
                member_type: claims.member_type,
            },
        )
        .await;
        let remaining = (claims.exp as i64)
            .saturating_sub(chrono::Utc::now().timestamp())
            .max(0) as u64;
        let expiry = tokio::time::sleep(Duration::from_secs(remaining));
        tokio::pin!(expiry);

        loop {
            tokio::select! {
                _ = &mut expiry => {
                    send_response(&tx, &ServerMessage::AuthError {
                        message: "Token has expired".to_string(), code: Some("TOKEN_EXPIRED".to_string()),
                    }).await;
                    break;
                }
                event = events.recv() => {
                    match event {
                        Ok(event) if joined_rooms.contains(&event.room_id) => {
                            if state.rooms.authorize(&event.room_id, &member_id, false).await.is_err() { continue; }
                            let response = ServerMessage::NewMessage {
                                room_id: event.room_id, message_id: event.message.id,
                                sender_id: event.message.sender, content: event.message.text,
                                reply_to: event.message.reply_to, timestamp: event.timestamp,
                            };
                            if !send_response(&tx, &response).await { break; }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                            send_response(&tx, &ServerMessage::Error {
                                message: "Reload room history after reconnect".to_string(),
                                code: Some("SYNC_REQUIRED".to_string()),
                            }).await;
                            break;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                        _ => {}
                    }
                }
                message = receiver.next() => {
                    let Some(Ok(message)) = message else { break };
                    match message {
                        Message::Text(text) => {
                            if !rate_limiter.check_and_increment() {
                                send_response(&tx, &ServerMessage::Error {
                                    message: "Rate limit exceeded".to_string(), code: Some("RATE_LIMITED".to_string()),
                                }).await;
                                break;
                            }
                            let response = match parse_client_message(&text) {
                                Ok(ClientMessage::JoinRoom { room_id }) => {
                                    match state.rooms.authorize(&room_id, &member_id, false).await {
                                        Ok(_) => {
                                            joined_rooms.insert(room_id.clone());
                                            ServerMessage::RoomJoined { room_id }
                                        }
                                        Err(error) => room_error(error),
                                    }
                                }
                                Ok(ClientMessage::LeaveRoom { room_id }) => {
                                    joined_rooms.remove(&room_id);
                                    ServerMessage::RoomLeft { room_id }
                                }
                                Ok(ClientMessage::SendMessage { room_id, content, reply_to, client_message_id }) => {
                                    if !joined_rooms.contains(&room_id) {
                                        ServerMessage::Error { message: "Join the room first".to_string(), code: Some("ROOM_NOT_JOINED".to_string()) }
                                    } else if let Err(error) = state.rooms.authorize(&room_id, &member_id, false).await {
                                        room_error(error)
                                    } else {
                                        let accepted_room = room_id.clone();
                                        let accepted_key = client_message_id.clone();
                                        match state.rooms.send_message_idempotent(crate::rooms::SendMessageCommand {
                                            room_id, sender: member_id.clone(), text: content, reply_to,
                                        }, client_message_id).await {
                                            Ok(message) => ServerMessage::MessageAccepted { room_id: accepted_room, message_id: message.id, client_message_id: accepted_key },
                                            Err(error) => room_error(error),
                                        }
                                    }
                                }
                                Ok(ClientMessage::Heartbeat { timestamp }) => ServerMessage::HeartbeatAck { timestamp },
                                Ok(ClientMessage::Auth { .. }) => ServerMessage::Error {
                                    message: "Already authenticated".to_string(), code: Some("ALREADY_AUTHENTICATED".to_string()),
                                },
                                Err(_) => ServerMessage::Error {
                                    message: "Invalid message format".to_string(), code: Some("PARSE_ERROR".to_string()),
                                },
                            };
                            if !send_response(&tx, &response).await { break; }
                        }
                        Message::Ping(data) => { if tx.try_send(Message::Pong(data)).is_err() { break; } }
                        Message::Close(_) => break,
                        _ => {}
                    }
                }
            }
        }
        // An older connection must never remove its replacement during reconnect.
        let mut connections = state.connections.write().await;
        if connections
            .get(&member_id)
            .is_some_and(|sender| sender.tx.same_channel(&tx))
        {
            connections.remove(&member_id);
        }
    }
    let _ = tx.try_send(Message::Close(None));
    drop(tx);
    if timeout(Duration::from_secs(1), &mut writer).await.is_err() {
        writer.abort();
    }
}

async fn send_response(tx: &mpsc::Sender<Message>, response: &ServerMessage) -> bool {
    let Ok(json) = serialize_server_message(response) else {
        return false;
    };
    // A slow connection cannot block HTTP writes or other subscribers.
    tx.try_send(Message::Text(json.into())).is_ok()
}

fn room_error(error: crate::rooms::RoomCommandError) -> ServerMessage {
    use crate::rooms::RoomCommandError;
    let (message, code) = match error {
        RoomCommandError::Forbidden => ("Room access denied".to_string(), "FORBIDDEN"),
        RoomCommandError::Conflict => ("Retry key content mismatch".to_string(), "CONFLICT"),
        RoomCommandError::RoomNotFound => ("Room not found".to_string(), "NOT_FOUND"),
        RoomCommandError::Validation(message) => (message, "BAD_REQUEST"),
        RoomCommandError::ServiceUnavailable => {
            ("Service unavailable".to_string(), "SERVICE_UNAVAILABLE")
        }
    };
    ServerMessage::Error {
        message,
        code: Some(code.to_string()),
    }
}

/// Create a WebSocket router with authentication
pub fn websocket_routes() -> axum::Router<WebSocketState> {
    use axum::extract::State;

    axum::Router::new().route(
        "/ws",
        axum::routing::get(
            |State(state): State<WebSocketState>,
             ws: WebSocketUpgrade,
             query: Query<WebSocketQuery>| async move {
                websocket_upgrade_with_state(ws, query, state).await
            },
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn websocket_state_tracks_connections() {
        let state = WebSocketState::new();
        let (tx, _) = mpsc::channel(16);

        assert_eq!(state.connection_count().await, 0);

        state
            .add_connection("user1".to_string(), "human".to_string(), tx.clone())
            .await;
        assert_eq!(state.connection_count().await, 1);

        state
            .add_connection("user2".to_string(), "ai".to_string(), tx)
            .await;
        assert_eq!(state.connection_count().await, 2);

        state.remove_connection("user1").await;
        assert_eq!(state.connection_count().await, 1);
    }

    #[tokio::test]
    async fn websocket_state_replaces_existing_connection() {
        let state = WebSocketState::new();
        let (tx1, mut rx1) = mpsc::channel(16);
        let (tx2, _) = mpsc::channel(16);

        state
            .add_connection("user1".to_string(), "human".to_string(), tx1)
            .await;
        assert_eq!(state.connection_count().await, 1);

        // Adding same user should close old connection
        state
            .add_connection("user1".to_string(), "human".to_string(), tx2)
            .await;
        assert_eq!(state.connection_count().await, 1);

        // Old connection should receive close message
        let msg = rx1.try_recv();
        assert!(matches!(msg, Ok(Message::Close(_))));
    }

    #[tokio::test]
    async fn websocket_state_send_to_user() {
        let state = WebSocketState::new();
        let (tx, mut rx) = mpsc::channel(16);

        state
            .add_connection("user1".to_string(), "human".to_string(), tx)
            .await;

        let sent = state
            .send_to_user("user1", Message::Text("hello".to_string().into()))
            .await;
        assert!(sent);

        let msg = rx.try_recv();
        assert!(matches!(msg, Ok(Message::Text(t)) if t == "hello"));
    }

    #[tokio::test]
    async fn websocket_state_send_to_nonexistent_user() {
        let state = WebSocketState::new();

        let sent = state
            .send_to_user("unknown", Message::Text("hello".to_string().into()))
            .await;
        assert!(!sent);
    }

    #[tokio::test]
    async fn websocket_state_broadcast() {
        let state = WebSocketState::new();
        let (tx1, mut rx1) = mpsc::channel(16);
        let (tx2, mut rx2) = mpsc::channel(16);

        state
            .add_connection("user1".to_string(), "human".to_string(), tx1)
            .await;
        state
            .add_connection("user2".to_string(), "ai".to_string(), tx2)
            .await;

        state
            .broadcast(Message::Text("broadcast".to_string().into()))
            .await;

        let msg1 = rx1.try_recv();
        let msg2 = rx2.try_recv();

        assert!(matches!(msg1, Ok(Message::Text(t)) if t == "broadcast"));
        assert!(matches!(msg2, Ok(Message::Text(t)) if t == "broadcast"));
    }
}
