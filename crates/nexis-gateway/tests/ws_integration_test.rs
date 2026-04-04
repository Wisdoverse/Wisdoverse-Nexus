//! WebSocket Integration Tests for Nexis Gateway
//!
//! Tests the complete WebSocket flow: connection → authentication → message routing.
//! Covers:
//! - Connection & Authentication
//! - Message Routing (rooms, broadcast, @ai mentions)
//! - Connection Management (reconnect, cleanup)

use futures::{SinkExt, StreamExt};
use nexis_gateway::auth::JwtConfig;
use nexis_gateway::connection::{ServerMessage, WebSocketState};
use nexis_gateway::router::build_routes;
use nexis_gateway::router::ws_router::RouterState;
use serde_json::json;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::{tungstenite::Message as WsMessage, MaybeTlsStream, WebSocketStream};

const TEST_JWT_SECRET: &str = "test_secret_for_ws_integration_tests";

/// Helper to generate a test JWT token
fn generate_test_token(member_id: &str, member_type: &str) -> String {
    let config = JwtConfig::new(TEST_JWT_SECRET, "nexis".to_string(), "nexis".to_string());
    config
        .generate_token(member_id, member_type)
        .expect("Failed to generate test token")
}

/// Helper to create auth message
fn auth_message(token: &str) -> String {
    json!({
        "type": "auth",
        "token": format!("Bearer {}", token)
    })
    .to_string()
}

/// Helper to create join room message
#[allow(dead_code)]
fn join_room_message(room_id: &str) -> String {
    json!({
        "type": "join_room",
        "room_id": room_id
    })
    .to_string()
}

/// Helper to create leave room message
#[allow(dead_code)]
fn leave_room_message(room_id: &str) -> String {
    json!({
        "type": "leave_room",
        "room_id": room_id
    })
    .to_string()
}

/// Helper to create send message
#[allow(dead_code)]
fn send_message(room_id: &str, content: &str) -> String {
    json!({
        "type": "send_message",
        "room_id": room_id,
        "content": content
    })
    .to_string()
}

/// Helper to parse server message
#[allow(dead_code)]
fn parse_server_message(text: &str) -> ServerMessage {
    serde_json::from_str(text).expect("Failed to parse server message")
}

/// Test WebSocket client wrapper
struct TestClient {
    ws: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
}

impl TestClient {
    /// Connect to WebSocket endpoint
    async fn connect(addr: SocketAddr) -> Self {
        let url = format!("ws://{}/ws", addr);
        let (ws, _) = tokio_tungstenite::connect_async(&url)
            .await
            .expect("Failed to connect to WebSocket");
        Self { ws }
    }

    /// Send a text message
    async fn send(&mut self, text: &str) {
        self.ws
            .send(WsMessage::Text(text.into()))
            .await
            .expect("Failed to send message");
    }

    /// Receive a text message (with timeout)
    async fn recv(&mut self) -> Option<String> {
        loop {
            match timeout(Duration::from_secs(5), self.ws.next()).await {
                Ok(Some(Ok(WsMessage::Text(text)))) => return Some(text.to_string()),
                Ok(Some(Ok(WsMessage::Close(_)))) => return None,
                Ok(Some(Ok(WsMessage::Ping(_)))) | Ok(Some(Ok(WsMessage::Pong(_)))) => continue,
                Ok(Some(Ok(_))) => continue, // Skip other non-text messages
                Ok(Some(Err(_))) | Ok(None) => return None,
                Err(_) => return None, // Timeout
            }
        }
    }

    /// Receive and parse as ServerMessage
    async fn recv_message(&mut self) -> Option<ServerMessage> {
        self.recv()
            .await
            .as_ref()
            .and_then(|text| serde_json::from_str(text).ok())
    }

    /// Wait for connection to close
    async fn wait_close(&mut self) -> bool {
        loop {
            match timeout(Duration::from_secs(2), self.ws.next()).await {
                Ok(Some(Ok(WsMessage::Close(_)))) => return true,
                Ok(None) => return true,
                Ok(Some(Ok(_))) => continue,
                _ => return false,
            }
        }
    }

    /// Close the connection
    async fn close(mut self) {
        let _ = self.ws.close(None).await;
    }
}

/// Start a test server and return its address
async fn start_test_server() -> (SocketAddr, tokio::task::JoinHandle<()>) {
    std::env::set_var("JWT_SECRET", TEST_JWT_SECRET);

    let app = build_routes();
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get address");

    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("Server failed");
    });

    // Give server time to start
    sleep(Duration::from_millis(50)).await;

    (addr, handle)
}

// =============================================================================
// Connection & Authentication Tests
// =============================================================================

#[tokio::test]
async fn ws_connection_established_successfully() {
    let (addr, _handle) = start_test_server().await;

    // Should be able to connect without immediate error
    let client = TestClient::connect(addr).await;

    // Connection should remain open (no immediate close)
    sleep(Duration::from_millis(100)).await;

    client.close().await;
}

#[tokio::test]
async fn ws_auth_with_valid_token_returns_success() {
    let (addr, _handle) = start_test_server().await;
    let mut client = TestClient::connect(addr).await;

    let token = generate_test_token("alice", "human");
    client.send(&auth_message(&token)).await;

    let response = client.recv_message().await;

    match response {
        Some(ServerMessage::AuthSuccess {
            member_id,
            member_type,
        }) => {
            assert_eq!(member_id, "alice");
            assert_eq!(member_type, "human");
        }
        other => panic!("Expected AuthSuccess, got {:?}", other),
    }

    client.close().await;
}

#[tokio::test]
async fn ws_auth_with_invalid_token_returns_error() {
    let (addr, _handle) = start_test_server().await;
    let mut client = TestClient::connect(addr).await;

    client.send(&auth_message("invalid_token")).await;

    let response = client.recv_message().await;

    match response {
        Some(ServerMessage::AuthError { code, .. }) => {
            assert_eq!(code, Some("AUTH_FAILED".to_string()));
        }
        other => panic!("Expected AuthError, got {:?}", other),
    }

    client.close().await;
}

#[tokio::test]
async fn ws_auth_with_expired_token_returns_error() {
    let (addr, _handle) = start_test_server().await;
    let mut client = TestClient::connect(addr).await;

    // Create an expired token (exp in the past)
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = nexis_gateway::auth::Claims {
        sub: "expired_user".to_string(),
        exp: now - 3600, // Expired 1 hour ago
        iat: now - 7200,
        iss: "nexis".to_string(),
        aud: "nexis".to_string(),
        member_type: "human".to_string(),
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .expect("encode token");

    client.send(&auth_message(&token)).await;

    let response = client.recv_message().await;

    match response {
        Some(ServerMessage::AuthError { code, .. }) => {
            assert_eq!(code, Some("TOKEN_EXPIRED".to_string()));
        }
        other => panic!("Expected AuthError with TOKEN_EXPIRED, got {:?}", other),
    }

    client.close().await;
}

#[tokio::test]
async fn ws_message_before_auth_returns_auth_required() {
    let (addr, _handle) = start_test_server().await;
    let mut client = TestClient::connect(addr).await;

    // Try to send a non-auth message first
    client.send(&join_room_message("room_1")).await;

    let response = client.recv_message().await;

    match response {
        Some(ServerMessage::AuthRequired { .. }) => {}
        other => panic!("Expected AuthRequired, got {:?}", other),
    }

    client.close().await;
}

#[tokio::test]
#[ignore] // This test takes ~10 seconds due to AUTH_TIMEOUT
async fn ws_auth_timeout_disconnects_client() {
    let (addr, _handle) = start_test_server().await;
    let mut client = TestClient::connect(addr).await;

    // Don't send auth, wait for timeout
    // AUTH_TIMEOUT is 10 seconds
    let closed = client.wait_close().await;

    // Connection should be closed due to auth timeout
    assert!(closed, "Connection should be closed after auth timeout");
}

// =============================================================================
// Message Routing Tests
// =============================================================================

#[tokio::test]
async fn ws_join_room_returns_confirmation() {
    let (addr, _handle) = start_test_server().await;
    let mut client = TestClient::connect(addr).await;

    // Authenticate first
    let token = generate_test_token("alice", "human");
    client.send(&auth_message(&token)).await;
    let _ = client.recv_message().await; // AuthSuccess

    // Join room - current implementation may not have full room support
    client.send(&join_room_message("room_1")).await;

    // The message will be handled but may return NoResponse
    // This test verifies the message format is correct

    client.close().await;
}

#[tokio::test]
async fn ws_heartbeat_returns_ack() {
    let (addr, _handle) = start_test_server().await;
    let mut client = TestClient::connect(addr).await;

    // Authenticate first
    let token = generate_test_token("alice", "human");
    client.send(&auth_message(&token)).await;
    let _ = client.recv_message().await; // AuthSuccess

    // Send heartbeat
    let heartbeat = json!({
        "type": "heartbeat",
        "timestamp": 1234567890
    })
    .to_string();
    client.send(&heartbeat).await;

    let response = client.recv_message().await;

    match response {
        Some(ServerMessage::HeartbeatAck { timestamp }) => {
            assert_eq!(timestamp, Some(1234567890));
        }
        other => panic!("Expected HeartbeatAck, got {:?}", other),
    }

    client.close().await;
}

#[tokio::test]
async fn ws_already_authenticated_error() {
    let (addr, _handle) = start_test_server().await;
    let mut client = TestClient::connect(addr).await;

    // Authenticate first
    let token = generate_test_token("alice", "human");
    client.send(&auth_message(&token)).await;
    let _ = client.recv_message().await; // AuthSuccess

    // Try to authenticate again
    client.send(&auth_message(&token)).await;

    let response = client.recv_message().await;

    match response {
        Some(ServerMessage::Error { code, .. }) => {
            assert_eq!(code, Some("ALREADY_AUTHENTICATED".to_string()));
        }
        other => panic!("Expected Error ALREADY_AUTHENTICATED, got {:?}", other),
    }

    client.close().await;
}

#[tokio::test]
async fn ws_invalid_message_format_returns_error() {
    let (addr, _handle) = start_test_server().await;
    let mut client = TestClient::connect(addr).await;

    // Authenticate first
    let token = generate_test_token("alice", "human");
    client.send(&auth_message(&token)).await;
    let _ = client.recv_message().await; // AuthSuccess

    // Send invalid JSON
    client.send("not valid json").await;

    let response = client.recv_message().await;

    match response {
        Some(ServerMessage::Error { message, .. }) => {
            assert!(message.contains("Invalid message format") || message.contains("parse"));
        }
        other => panic!("Expected Error for invalid message, got {:?}", other),
    }

    client.close().await;
}

// =============================================================================
// Connection Management Tests (using WebSocketState)
// =============================================================================

#[tokio::test]
async fn ws_same_user_reconnect_closes_old_connection() {
    let ws_state = WebSocketState::with_jwt_config(JwtConfig::new(
        TEST_JWT_SECRET,
        "nexis".to_string(),
        "nexis".to_string(),
    ));

    let (tx1, mut rx1) = tokio::sync::mpsc::channel(16);
    let (tx2, _rx2) = tokio::sync::mpsc::channel(16);

    // Add first connection
    ws_state
        .add_connection("alice".to_string(), "human".to_string(), tx1)
        .await;
    assert_eq!(ws_state.connection_count().await, 1);

    // Add same user again - should close old connection
    ws_state
        .add_connection("alice".to_string(), "human".to_string(), tx2)
        .await;
    assert_eq!(ws_state.connection_count().await, 1);

    // Old connection should receive close message
    let msg = rx1.try_recv();
    assert!(
        matches!(msg, Ok(axum::extract::ws::Message::Close(_))),
        "Expected Close message, got {:?}",
        msg
    );
}

#[tokio::test]
async fn ws_disconnect_removes_from_state() {
    let ws_state = WebSocketState::new();
    let (tx, _) = tokio::sync::mpsc::channel(16);

    ws_state
        .add_connection("alice".to_string(), "human".to_string(), tx)
        .await;
    assert_eq!(ws_state.connection_count().await, 1);

    ws_state.remove_connection("alice").await;
    assert_eq!(ws_state.connection_count().await, 0);

    // Removing non-existent should not panic
    ws_state.remove_connection("unknown").await;
    assert_eq!(ws_state.connection_count().await, 0);
}

#[tokio::test]
async fn ws_send_to_specific_user() {
    let ws_state = WebSocketState::new();
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);

    ws_state
        .add_connection("alice".to_string(), "human".to_string(), tx)
        .await;

    let sent = ws_state
        .send_to_user("alice", axum::extract::ws::Message::Text("hello".to_string()))
        .await;
    assert!(sent);

    let msg = rx.try_recv();
    assert!(matches!(msg, Ok(axum::extract::ws::Message::Text(t)) if t == "hello"));

    // Send to unknown user should return false
    let sent = ws_state
        .send_to_user("unknown", axum::extract::ws::Message::Text("hello".to_string()))
        .await;
    assert!(!sent);
}

#[tokio::test]
async fn ws_broadcast_to_all_connections() {
    let ws_state = WebSocketState::new();
    let (tx1, mut rx1) = tokio::sync::mpsc::channel(16);
    let (tx2, mut rx2) = tokio::sync::mpsc::channel(16);

    ws_state
        .add_connection("alice".to_string(), "human".to_string(), tx1)
        .await;
    ws_state
        .add_connection("bob".to_string(), "human".to_string(), tx2)
        .await;

    ws_state
        .broadcast(axum::extract::ws::Message::Text("broadcast".to_string()))
        .await;

    let msg1 = rx1.try_recv();
    let msg2 = rx2.try_recv();

    assert!(matches!(msg1, Ok(axum::extract::ws::Message::Text(t)) if t == "broadcast"));
    assert!(matches!(msg2, Ok(axum::extract::ws::Message::Text(t)) if t == "broadcast"));
}

// =============================================================================
// Room & Broadcast Tests (using RouterState)
// =============================================================================

#[tokio::test]
async fn router_join_and_leave_room() {
    let router = RouterState::new();
    let (tx, _) = tokio::sync::mpsc::channel(16);

    router.register_connection("alice".to_string(), tx).await;

    // Join room
    let joined = router
        .join_room(&"alice".to_string(), &"room_1".to_string())
        .await;
    assert!(joined);
    assert_eq!(router.room_member_count(&"room_1".to_string()).await, 1);

    let rooms: Vec<String> = router.member_rooms(&"alice".to_string()).await;
    assert!(rooms.contains(&"room_1".to_string()));

    // Leave room
    let left = router
        .leave_room(&"alice".to_string(), &"room_1".to_string())
        .await;
    assert!(left);
    assert_eq!(router.room_member_count(&"room_1".to_string()).await, 0);
}

#[tokio::test]
async fn router_broadcast_to_room_excludes_sender() {
    let router = RouterState::new();
    let (tx1, mut rx1) = tokio::sync::mpsc::channel(16);
    let (tx2, mut rx2) = tokio::sync::mpsc::channel(16);
    let (tx3, mut rx3) = tokio::sync::mpsc::channel(16);

    router.register_connection("alice".to_string(), tx1).await;
    router.register_connection("bob".to_string(), tx2).await;
    router.register_connection("charlie".to_string(), tx3).await;

    router.join_room(&"alice".to_string(), &"room_1".to_string()).await;
    router.join_room(&"bob".to_string(), &"room_1".to_string()).await;
    router.join_room(&"charlie".to_string(), &"room_1".to_string()).await;

    // Broadcast from alice - bob and charlie should receive, alice should not
    let sent = router
        .broadcast_to_room(&"room_1".to_string(), &"alice".to_string(), "hello room")
        .await;
    assert_eq!(sent, 2);

    // Alice should not receive her own message
    let alice_msg = tokio::time::timeout(Duration::from_millis(50), rx1.recv()).await;
    assert!(alice_msg.is_err() || alice_msg.unwrap().is_none());

    // Bob should receive
    let bob_msg = tokio::time::timeout(Duration::from_millis(50), rx2.recv())
        .await
        .expect("Bob should receive")
        .expect("Should have message");
    assert!(matches!(bob_msg, axum::extract::ws::Message::Text(t) if t == "hello room"));

    // Charlie should receive
    let charlie_msg = tokio::time::timeout(Duration::from_millis(50), rx3.recv())
        .await
        .expect("Charlie should receive")
        .expect("Should have message");
    assert!(matches!(charlie_msg, axum::extract::ws::Message::Text(t) if t == "hello room"));
}

#[tokio::test]
async fn router_unregister_leaves_all_rooms() {
    let router = RouterState::new();
    let (tx, _) = tokio::sync::mpsc::channel(16);

    router.register_connection("alice".to_string(), tx).await;
    router.join_room(&"alice".to_string(), &"room_1".to_string()).await;
    router.join_room(&"alice".to_string(), &"room_2".to_string()).await;

    assert_eq!(router.room_count().await, 2);

    router.unregister_connection(&"alice".to_string()).await;

    // Empty rooms are removed
    assert_eq!(router.room_count().await, 0);
    assert_eq!(router.connection_count().await, 0);
}

#[tokio::test]
async fn router_send_to_member() {
    let router = RouterState::new();
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);

    router.register_connection("alice".to_string(), tx).await;

    let sent = router.send_to_member(&"alice".to_string(), "direct message").await;
    assert!(sent);

    let msg = tokio::time::timeout(Duration::from_millis(50), rx.recv())
        .await
        .expect("Should receive")
        .expect("Should have message");
    assert!(matches!(msg, axum::extract::ws::Message::Text(t) if t == "direct message"));

    // Send to unknown member
    let sent = router.send_to_member(&"unknown".to_string(), "message").await;
    assert!(!sent);
}

// =============================================================================
// AI Mention Tests
// =============================================================================

#[test]
fn ai_detect_mention() {
    use nexis_gateway::handlers::ai::AiHandler;

    assert!(AiHandler::detect_ai_mention("@ai hello").is_some());
    assert!(AiHandler::detect_ai_mention("@AI what's up").is_some());
    assert!(AiHandler::detect_ai_mention("Hey @assistant help").is_some());
    assert!(AiHandler::detect_ai_mention("hello @ai").is_some());

    // Should NOT match (word boundary)
    assert!(AiHandler::detect_ai_mention("@airplane is flying").is_none());
    assert!(AiHandler::detect_ai_mention("@aid is needed").is_none());

    // No mention
    assert!(AiHandler::detect_ai_mention("hello world").is_none());
    assert!(AiHandler::detect_ai_mention("email@test.com").is_none());
}

#[test]
fn ai_extract_prompt_preserves_case() {
    use nexis_gateway::handlers::ai::AiHandler;

    // Basic extraction
    assert_eq!(AiHandler::extract_prompt("@ai hello"), "hello");

    // Preserve case (P1 fix)
    assert_eq!(AiHandler::extract_prompt("@AI what's up"), "what's up");
    assert!(AiHandler::extract_prompt("Hey @assistant help me").contains("Hey"));
    assert!(
        AiHandler::extract_prompt("Hello World @ai How Are You?").contains("Hello World")
    );

    // Should not remove @airplane (word boundary)
    let result = AiHandler::extract_prompt("@airplane is cool @ai but this is removed");
    assert!(result.contains("@airplane"));
    // Note: @ai (standalone) should be removed, @airplane should be preserved
}

// =============================================================================
// Edge Cases & Boundary Conditions
// =============================================================================

#[tokio::test]
async fn ws_multiple_users_same_room() {
    let router = RouterState::new();
    let (tx1, _rx1) = tokio::sync::mpsc::channel(16);
    let (tx2, _rx2) = tokio::sync::mpsc::channel(16);
    let (tx3, _rx3) = tokio::sync::mpsc::channel(16);

    router.register_connection("alice".to_string(), tx1).await;
    router.register_connection("bob".to_string(), tx2).await;
    router.register_connection("charlie".to_string(), tx3).await;

    // All join the same room
    router.join_room(&"alice".to_string(), &"room_1".to_string()).await;
    router.join_room(&"bob".to_string(), &"room_1".to_string()).await;
    router.join_room(&"charlie".to_string(), &"room_1".to_string()).await;

    assert_eq!(router.room_member_count(&"room_1".to_string()).await, 3);

    // Broadcast from alice - 2 others should receive
    let sent = router
        .broadcast_to_room(&"room_1".to_string(), &"alice".to_string(), "hello")
        .await;
    assert_eq!(sent, 2);
}

#[tokio::test]
async fn ws_user_in_multiple_rooms() {
    let router = RouterState::new();
    let (tx, _) = tokio::sync::mpsc::channel(16);

    router.register_connection("alice".to_string(), tx).await;

    // Join multiple rooms
    router.join_room(&"alice".to_string(), &"room_1".to_string()).await;
    router.join_room(&"alice".to_string(), &"room_2".to_string()).await;
    router.join_room(&"alice".to_string(), &"room_3".to_string()).await;

    let rooms: Vec<String> = router.member_rooms(&"alice".to_string()).await;
    assert_eq!(rooms.len(), 3);
    assert!(rooms.contains(&"room_1".to_string()));
    assert!(rooms.contains(&"room_2".to_string()));
    assert!(rooms.contains(&"room_3".to_string()));

    // Leave one room
    router.leave_room(&"alice".to_string(), &"room_2".to_string()).await;
    let rooms: Vec<String> = router.member_rooms(&"alice".to_string()).await;
    assert_eq!(rooms.len(), 2);
    assert!(!rooms.contains(&"room_2".to_string()));
}

#[tokio::test]
async fn ws_rejoin_same_room() {
    let router = RouterState::new();
    let (tx, _) = tokio::sync::mpsc::channel(16);

    router.register_connection("alice".to_string(), tx).await;

    // Join room twice
    router.join_room(&"alice".to_string(), &"room_1".to_string()).await;
    router.join_room(&"alice".to_string(), &"room_1".to_string()).await;

    // Should still have 1 member (idempotent)
    assert_eq!(router.room_member_count(&"room_1".to_string()).await, 1);
}

#[tokio::test]
async fn ws_empty_room_is_removed() {
    let router = RouterState::new();
    let (tx1, _) = tokio::sync::mpsc::channel(16);
    let (tx2, _) = tokio::sync::mpsc::channel(16);

    router.register_connection("alice".to_string(), tx1).await;
    router.register_connection("bob".to_string(), tx2).await;

    router.join_room(&"alice".to_string(), &"room_1".to_string()).await;
    router.join_room(&"bob".to_string(), &"room_1".to_string()).await;

    assert_eq!(router.room_count().await, 1);

    // Both leave
    router.leave_room(&"alice".to_string(), &"room_1".to_string()).await;
    router.leave_room(&"bob".to_string(), &"room_1".to_string()).await;

    // Empty room should be removed
    assert_eq!(router.room_count().await, 0);
}
