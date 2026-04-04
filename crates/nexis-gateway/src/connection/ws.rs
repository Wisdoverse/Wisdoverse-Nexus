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
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::auth::JwtConfig;
use crate::connection::{
    create_auth_timeout_message, parse_client_message, serialize_server_message,
    AuthenticatedSession, ConnectionState, MessageResult, ServerMessage, WebSocketAuthenticator,
    AUTH_TIMEOUT,
};

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

/// WebSocket connection state shared across handlers
#[derive(Clone)]
pub struct WebSocketState {
    /// Active connections by member_id
    connections: ConnectionMap,
    /// JWT authenticator
    authenticator: Arc<WebSocketAuthenticator>,
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
        }
    }

    /// Create with custom JWT config (for testing)
    pub fn with_jwt_config(jwt_config: JwtConfig) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            authenticator: Arc::new(WebSocketAuthenticator::new(jwt_config)),
        }
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
            let _ = old_sender.tx.send(Message::Close(None)).await;
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
            sender.tx.send(message).await.is_ok()
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
            if sender.tx.send(message.clone()).await.is_ok() {
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
                let member_id = claims.sub.clone();
                let member_type = claims.member_type.clone();

                tracing::info!(
                    member_id = %member_id,
                    "WebSocket authenticated via query token"
                );

                ws.on_upgrade(move |socket| {
                    handle_authenticated_socket(socket, state, member_id, member_type)
                })
            }
            Err(e) => {
                // Invalid token - reject connection
                tracing::warn!("WebSocket connection rejected: invalid token - {}", e);
                (StatusCode::UNAUTHORIZED, "Invalid or expired token").into_response()
            }
        }
    } else {
        // No token in query - require first-message authentication
        ws.on_upgrade(move |socket| handle_socket(socket, state))
    }
}

/// Handle WebSocket connection with first-message authentication
async fn handle_socket(socket: WebSocket, state: WebSocketState) {
    use futures::{SinkExt, StreamExt};
    use tokio::time::timeout;

    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<Message>(256);

    // Spawn writer task
    let writer = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if sender.send(message).await.is_err() {
                break;
            }
        }
    });

    // Authentication timeout
    let auth_future = async {
        let mut conn_state = ConnectionState::Unauthenticated;
        let mut session: Option<AuthenticatedSession> = None;

        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    tracing::debug!("Received WebSocket message: {}", text);

                    match parse_client_message(&text) {
                        Ok(client_msg) => {
                            match state.authenticator.process_message(conn_state, &client_msg) {
                                MessageResult::Response(server_msg) => {
                                    // Check if auth success
                                    if let ServerMessage::AuthSuccess {
                                        ref member_id,
                                        ref member_type,
                                    } = server_msg
                                    {
                                        conn_state = ConnectionState::Authenticated;
                                        session = Some(AuthenticatedSession {
                                            member_id: member_id.clone(),
                                            member_type: member_type.clone(),
                                        });
                                    }

                                    if let Ok(json) = serialize_server_message(&server_msg) {
                                        if tx.send(Message::Text(json)).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                                MessageResult::Authenticated(s) => {
                                    conn_state = ConnectionState::Authenticated;
                                    session = Some(s);
                                }
                                MessageResult::CloseConnection => {
                                    tracing::debug!("Connection closed by request");
                                    break;
                                }
                                MessageResult::NoResponse => {}
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse WebSocket message: {}", e);
                            let error_msg = ServerMessage::Error {
                                message: format!("Invalid message format: {}", e),
                                code: Some("PARSE_ERROR".to_string()),
                            };
                            if let Ok(json) = serialize_server_message(&error_msg) {
                                let _ = tx.send(Message::Text(json)).await;
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    tracing::debug!("Client disconnected");
                    break;
                }
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        session
    };

    // Apply authentication timeout
    match timeout(AUTH_TIMEOUT, auth_future).await {
        Ok(Some(session)) => {
            // Authenticated - register connection and continue
            let member_id = session.member_id.clone();
            state
                .add_connection(member_id.clone(), session.member_type, tx.clone())
                .await;

            // Continue handling messages (simplified - full impl would continue the loop)
            tracing::info!(member_id = %member_id, "WebSocket session ended");
            state.remove_connection(&member_id).await;
        }
        Ok(None) => {
            // Connection closed without auth
            tracing::debug!("WebSocket connection closed without authentication");
        }
        Err(_) => {
            // Timeout - send error and close
            tracing::warn!("WebSocket authentication timeout");
            let _ = tx.send(Message::Text(create_auth_timeout_message())).await;
            let _ = tx.send(Message::Close(None)).await;
        }
    }

    writer.abort();
}

/// Handle pre-authenticated WebSocket connection
async fn handle_authenticated_socket(
    socket: WebSocket,
    state: WebSocketState,
    member_id: String,
    member_type: String,
) {
    use futures::{SinkExt, StreamExt};

    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<Message>(256);

    // Send auth success message
    let auth_success = ServerMessage::AuthSuccess {
        member_id: member_id.clone(),
        member_type: member_type.clone(),
    };
    if let Ok(json) = serialize_server_message(&auth_success) {
        let _ = tx.send(Message::Text(json)).await;
    }

    // Register connection
    state
        .add_connection(member_id.clone(), member_type.clone(), tx.clone())
        .await;

    // Spawn writer task
    let writer = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if sender.send(message).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                tracing::debug!(member_id = %member_id, "Received: {}", text);

                match parse_client_message(&text) {
                    Ok(client_msg) => {
                        match state
                            .authenticator
                            .process_message(ConnectionState::Authenticated, &client_msg)
                        {
                            MessageResult::Response(server_msg) => {
                                if let Ok(json) = serialize_server_message(&server_msg) {
                                    if tx.send(Message::Text(json)).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            MessageResult::CloseConnection => break,
                            _ => {}
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse message: {}", e);
                    }
                }
            }
            Ok(Message::Close(_)) => {
                tracing::debug!(member_id = %member_id, "Client disconnected");
                break;
            }
            Err(e) => {
                tracing::error!(member_id = %member_id, "WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    // Cleanup
    state.remove_connection(&member_id).await;
    writer.abort();
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
            .send_to_user("user1", Message::Text("hello".to_string()))
            .await;
        assert!(sent);

        let msg = rx.try_recv();
        assert!(matches!(msg, Ok(Message::Text(t)) if t == "hello"));
    }

    #[tokio::test]
    async fn websocket_state_send_to_nonexistent_user() {
        let state = WebSocketState::new();

        let sent = state
            .send_to_user("unknown", Message::Text("hello".to_string()))
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
            .broadcast(Message::Text("broadcast".to_string()))
            .await;

        let msg1 = rx1.try_recv();
        let msg2 = rx2.try_recv();

        assert!(matches!(msg1, Ok(Message::Text(t)) if t == "broadcast"));
        assert!(matches!(msg2, Ok(Message::Text(t)) if t == "broadcast"));
    }
}
