//! WebSocket message router
//!
//! Routes messages between connections based on room subscriptions.
//! Handles JoinRoom, LeaveRoom, and message broadcasting.
//! Supports AI member integration via @ai mentions.

use axum::extract::ws::Message;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

use crate::handlers::ai::AiHandler;
use nexis_ai::ProviderRegistry;
use nexis_context::ContextManager;

/// A connection's sender channel
pub type ConnectionTx = mpsc::Sender<Message>;

/// Room ID type
pub type RoomId = String;

/// Member ID type
pub type MemberId = String;

/// Router state for message routing
pub struct RouterState {
    /// room_id -> set of member_ids
    rooms: RwLock<HashMap<RoomId, HashSet<MemberId>>>,
    /// member_id -> sender channel
    connections: RwLock<HashMap<MemberId, ConnectionTx>>,
    /// member_id -> set of joined rooms (for cleanup)
    member_rooms: RwLock<HashMap<MemberId, HashSet<RoomId>>>,
    /// AI handler for @ai mentions
    ai_handler: Option<Arc<AiHandler>>,
}

impl std::fmt::Debug for RouterState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RouterState")
            .field(
                "rooms_count",
                &self.rooms.try_read().map(|r| r.len()).unwrap_or(0),
            )
            .field(
                "connections_count",
                &self.connections.try_read().map(|c| c.len()).unwrap_or(0),
            )
            .field("ai_handler_enabled", &self.ai_handler.is_some())
            .finish()
    }
}

impl Default for RouterState {
    fn default() -> Self {
        Self::new()
    }
}

impl RouterState {
    /// Create a new router state
    pub fn new() -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
            connections: RwLock::new(HashMap::new()),
            member_rooms: RwLock::new(HashMap::new()),
            ai_handler: None,
        }
    }

    /// Wrap in Arc for sharing
    pub fn shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Create router with AI handler
    pub fn with_ai_handler(
        context_manager: Arc<ContextManager>,
        provider_registry: Arc<ProviderRegistry>,
    ) -> Self {
        let ai_handler = AiHandler::with_defaults(context_manager, provider_registry);
        Self {
            rooms: RwLock::new(HashMap::new()),
            connections: RwLock::new(HashMap::new()),
            member_rooms: RwLock::new(HashMap::new()),
            ai_handler: Some(Arc::new(ai_handler)),
        }
    }

    /// Wrap with AI handler in Arc
    pub fn shared_with_ai(
        context_manager: Arc<ContextManager>,
        provider_registry: Arc<ProviderRegistry>,
    ) -> Arc<Self> {
        Arc::new(Self::with_ai_handler(context_manager, provider_registry))
    }

    /// Register a connection with its sender channel
    pub async fn register_connection(&self, member_id: MemberId, tx: ConnectionTx) {
        let mut connections = self.connections.write().await;
        if let Some(old_tx) = connections.insert(member_id.clone(), tx) {
            warn!(
                member_id = %member_id,
                "Replacing existing connection (old connection will be closed)"
            );
            // Try to close the old connection gracefully
            let _ = old_tx.send(Message::Close(None)).await;
        }
        info!(member_id = %member_id, "Connection registered");
    }

    /// Unregister a connection and leave all rooms
    pub async fn unregister_connection(&self, member_id: &MemberId) {
        // Leave all rooms
        let rooms_to_leave: Vec<RoomId> = {
            let member_rooms = self.member_rooms.read().await;
            member_rooms
                .get(member_id)
                .map(|r| r.iter().cloned().collect())
                .unwrap_or_default()
        };

        for room_id in rooms_to_leave {
            self.leave_room(member_id, &room_id).await;
        }

        // Remove connection
        let mut connections = self.connections.write().await;
        connections.remove(member_id);
        info!(member_id = %member_id, "Connection unregistered");
    }

    /// Join a room
    pub async fn join_room(&self, member_id: &MemberId, room_id: &RoomId) -> bool {
        // Add member to room
        {
            let mut rooms = self.rooms.write().await;
            rooms
                .entry(room_id.clone())
                .or_default()
                .insert(member_id.clone());
        }

        // Track room membership for cleanup
        {
            let mut member_rooms = self.member_rooms.write().await;
            member_rooms
                .entry(member_id.clone())
                .or_default()
                .insert(room_id.clone());
        }

        let member_count = self.room_member_count(room_id).await;
        info!(
            member_id = %member_id,
            room_id = %room_id,
            member_count = member_count,
            "Member joined room"
        );
        true
    }

    /// Leave a room
    pub async fn leave_room(&self, member_id: &MemberId, room_id: &RoomId) -> bool {
        let mut left = false;

        // Remove member from room
        {
            let mut rooms = self.rooms.write().await;
            if let Some(members) = rooms.get_mut(room_id) {
                left = members.remove(member_id);
                // Clean up empty rooms
                if members.is_empty() {
                    rooms.remove(room_id);
                    debug!(room_id = %room_id, "Room removed (empty)");
                }
            }
        }

        // Update member's room set
        {
            let mut member_rooms = self.member_rooms.write().await;
            if let Some(rooms) = member_rooms.get_mut(member_id) {
                rooms.remove(room_id);
                if rooms.is_empty() {
                    member_rooms.remove(member_id);
                }
            }
        }

        if left {
            let member_count = self.room_member_count(room_id).await;
            info!(
                member_id = %member_id,
                room_id = %room_id,
                remaining_members = member_count,
                "Member left room"
            );
        }
        left
    }

    /// Broadcast a message to all members in a room (except sender)
    pub async fn broadcast_to_room(
        &self,
        room_id: &RoomId,
        sender_id: &MemberId,
        message: &str,
    ) -> usize {
        let connections = self.connections.read().await;
        let rooms = self.rooms.read().await;

        let Some(members) = rooms.get(room_id) else {
            debug!(room_id = %room_id, "Room not found for broadcast");
            return 0;
        };

        let mut sent = 0;
        let mut failed = 0;

        for member_id in members {
            // Don't send back to sender
            if member_id == sender_id {
                continue;
            }

            if let Some(tx) = connections.get(member_id) {
                match tx.send(Message::Text(message.to_string().into())).await {
                    Ok(_) => sent += 1,
                    Err(e) => {
                        warn!(
                            member_id = %member_id,
                            error = %e,
                            "Failed to send message to member"
                        );
                        failed += 1;
                    }
                }
            }
        }

        if sent > 0 || failed > 0 {
            debug!(
                room_id = %room_id,
                sender_id = %sender_id,
                sent = sent,
                failed = failed,
                "Broadcast complete"
            );
        }

        sent
    }

    /// Send a message to a specific member
    pub async fn send_to_member(&self, member_id: &MemberId, message: &str) -> bool {
        let connections = self.connections.read().await;
        if let Some(tx) = connections.get(member_id) {
            tx.send(Message::Text(message.to_string().into()))
                .await
                .is_ok()
        } else {
            false
        }
    }

    /// Get the number of members in a room
    pub async fn room_member_count(&self, room_id: &RoomId) -> usize {
        let rooms = self.rooms.read().await;
        rooms.get(room_id).map(|m| m.len()).unwrap_or(0)
    }

    /// Get rooms a member has joined
    pub async fn member_rooms(&self, member_id: &MemberId) -> Vec<RoomId> {
        let member_rooms = self.member_rooms.read().await;
        member_rooms
            .get(member_id)
            .map(|r| r.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get total connection count
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Get total room count
    pub async fn room_count(&self) -> usize {
        self.rooms.read().await.len()
    }

    /// Process a chat message with AI detection
    ///
    /// If the message contains @ai or @assistant, spawns an async
    /// task to handle the AI request and broadcast the response.
    pub async fn process_chat_message(
        self: Arc<Self>,
        room_id: RoomId,
        sender_id: MemberId,
        message: String,
    ) {
        // Add message to context
        if let Some(ref handler) = self.ai_handler {
            handler
                .add_to_context(&room_id, nexis_context::MessageRole::User, message.clone())
                .await;
        }

        // Check for AI mention
        if AiHandler::detect_ai_mention(&message).is_some() {
            let handler = self.ai_handler.clone();
            let router = self.clone();
            let room = room_id.clone();
            let sender = sender_id.clone();
            let msg = message.clone();

            // Spawn async task to handle AI request
            tokio::spawn(async move {
                if let Some(ref handler) = handler {
                    debug!(
                        room_id = %room,
                        sender_id = %sender,
                        "Processing AI mention"
                    );

                    match handler.handle_message(&room, &sender, &msg).await {
                        Some(response) => {
                            // Add AI response to context
                            handler
                                .add_to_context(
                                    &room,
                                    nexis_context::MessageRole::Assistant,
                                    response.content.clone(),
                                )
                                .await;

                            // Format and broadcast response
                            let ai_message = response.to_chat_message();
                            debug!(
                                room_id = %room,
                                response_len = ai_message.len(),
                                "Broadcasting AI response"
                            );

                            // Broadcast to room (AI is sender, so broadcast to everyone else)
                            router
                                .broadcast_to_room(&room, &response.agent_name, &ai_message)
                                .await;
                        }
                        None => {
                            warn!(
                                room_id = %room,
                                "AI handler returned no response"
                            );
                        }
                    }
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn register_and_unregister_connection() {
        let router = RouterState::new();
        let (tx, _rx) = mpsc::channel(16);

        router.register_connection("alice".to_string(), tx).await;
        assert_eq!(router.connection_count().await, 1);

        router.unregister_connection(&"alice".to_string()).await;
        assert_eq!(router.connection_count().await, 0);
    }

    #[tokio::test]
    async fn join_and_leave_room() {
        let router = RouterState::new();
        let (tx, _) = mpsc::channel(16);

        router.register_connection("alice".to_string(), tx).await;

        // Join room
        let joined = router
            .join_room(&"alice".to_string(), &"room_1".to_string())
            .await;
        assert!(joined);
        assert_eq!(router.room_member_count(&"room_1".to_string()).await, 1);

        let rooms = router.member_rooms(&"alice".to_string()).await;
        assert_eq!(rooms, vec!["room_1"]);

        // Leave room
        let left = router
            .leave_room(&"alice".to_string(), &"room_1".to_string())
            .await;
        assert!(left);
        assert_eq!(router.room_member_count(&"room_1".to_string()).await, 0);
    }

    #[tokio::test]
    async fn broadcast_to_room() {
        let router = RouterState::new();
        let (tx1, mut rx1) = mpsc::channel(16);
        let (tx2, mut rx2) = mpsc::channel(16);
        let (tx3, mut rx3) = mpsc::channel(16);

        router.register_connection("alice".to_string(), tx1).await;
        router.register_connection("bob".to_string(), tx2).await;
        router.register_connection("charlie".to_string(), tx3).await;

        router
            .join_room(&"alice".to_string(), &"room_1".to_string())
            .await;
        router
            .join_room(&"bob".to_string(), &"room_1".to_string())
            .await;
        router
            .join_room(&"charlie".to_string(), &"room_1".to_string())
            .await;

        // Broadcast from alice
        let sent = router
            .broadcast_to_room(&"room_1".to_string(), &"alice".to_string(), "hello")
            .await;
        assert_eq!(sent, 2); // bob and charlie

        // Alice should not receive her own message
        let alice_msg = timeout(Duration::from_millis(50), rx1.recv()).await;
        assert!(alice_msg.is_err() || alice_msg.unwrap().is_none());

        // Bob and Charlie should receive
        let bob_msg = timeout(Duration::from_millis(50), rx2.recv())
            .await
            .expect("Bob should receive")
            .expect("Should have message");
        assert!(matches!(bob_msg, Message::Text(t) if t == "hello"));

        let charlie_msg = timeout(Duration::from_millis(50), rx3.recv())
            .await
            .expect("Charlie should receive")
            .expect("Should have message");
        assert!(matches!(charlie_msg, Message::Text(t) if t == "hello"));
    }

    #[tokio::test]
    async fn unregister_leaves_all_rooms() {
        let router = RouterState::new();
        let (tx, _) = mpsc::channel(16);

        router.register_connection("alice".to_string(), tx).await;
        router
            .join_room(&"alice".to_string(), &"room_1".to_string())
            .await;
        router
            .join_room(&"alice".to_string(), &"room_2".to_string())
            .await;

        assert_eq!(router.room_count().await, 2);

        router.unregister_connection(&"alice".to_string()).await;

        assert_eq!(router.room_count().await, 0); // Empty rooms are removed
        assert_eq!(router.connection_count().await, 0);
    }
}
