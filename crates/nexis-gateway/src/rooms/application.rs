//! Application service for room and message use cases.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

use super::domain::{Room, StoredMessage};

const MAX_MESSAGE_TEXT_LEN: usize = 32 * 1024;

#[derive(Debug, Clone)]
pub struct CreateRoomCommand {
    pub name: String,
    pub creator_id: Option<String>,
    pub topic: Option<String>,
    #[cfg(feature = "multi-tenant")]
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SendMessageCommand {
    pub room_id: String,
    pub sender: String,
    pub text: String,
    pub reply_to: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InviteMemberCommand {
    pub room_id: String,
    pub member_id: String,
}

#[derive(Debug, Clone)]
pub struct InviteMemberResult {
    pub room_id: String,
    pub member_id: String,
}

#[derive(Debug, Clone)]
pub struct RoomDetails {
    pub room: Room,
    pub messages: Vec<StoredMessage>,
}

#[derive(Debug, Clone)]
pub struct RoomSummary {
    pub id: String,
    pub name: String,
    pub topic: Option<String>,
    pub member_count: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ListRoomsResult {
    pub rooms: Vec<RoomSummary>,
    pub total: usize,
}

#[derive(Debug, Clone)]
pub struct MemberMessageRecord {
    pub room_id: String,
    pub message: StoredMessage,
}

#[derive(Debug, Clone)]
pub struct MemberRoomRecord {
    pub id: String,
    pub name: String,
    pub topic: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MemberDataExport {
    pub messages: Vec<MemberMessageRecord>,
    pub rooms: Vec<MemberRoomRecord>,
}

#[derive(Debug, Clone, Copy)]
pub struct MemberDeletionResult {
    pub messages: i64,
    pub rooms_created: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoomCommandError {
    Validation(String),
    Forbidden,
    Conflict,
    RoomNotFound,
    ServiceUnavailable,
}

#[async_trait]
pub trait RoomRepository: Send + Sync {
    async fn is_member(&self, room_id: &str, member_id: &str) -> bool;
    async fn active_room_count(&self) -> usize;
    async fn create_room(&self, room: Room) -> Result<Room, RoomCommandError>;
    async fn append_message(
        &self,
        room_id: String,
        message: StoredMessage,
    ) -> Result<StoredMessage, RoomCommandError>;
    async fn get_room(&self, id: &str) -> Result<RoomDetails, RoomCommandError>;
    async fn invite_member(
        &self,
        room_id: String,
        member_id: String,
    ) -> Result<InviteMemberResult, RoomCommandError>;
    async fn list_rooms(&self, limit: usize, offset: usize) -> ListRoomsResult;
    async fn delete_room(&self, id: &str) -> Result<(), RoomCommandError>;
    async fn export_member_data(&self, member_id: &str) -> MemberDataExport;
    async fn delete_member_data(
        &self,
        member_id: &str,
    ) -> Result<MemberDeletionResult, RoomCommandError>;
}

#[derive(Clone)]
pub struct RoomApplication {
    repository: Arc<dyn RoomRepository>,
    events: broadcast::Sender<RoomMessageEvent>,
    retries: Arc<Mutex<RetryLedger>>,
    message_order: Arc<Mutex<()>>,
}

type RetryLedger = HashMap<(String, String, String), AcceptedMessage>;

struct AcceptedMessage {
    message: StoredMessage,
    accepted_at: Instant,
}

/// A committed message, shared by HTTP and realtime consumers.
#[derive(Debug, Clone)]
pub struct RoomMessageEvent {
    pub room_id: String,
    pub message: StoredMessage,
    pub timestamp: i64,
}

impl RoomApplication {
    pub fn new(repository: Arc<dyn RoomRepository>) -> Self {
        let (events, _) = broadcast::channel(1024);
        Self {
            repository,
            events,
            retries: Arc::new(Mutex::new(HashMap::new())),
            message_order: Arc::new(Mutex::new(())),
        }
    }

    /// Subscribe to committed messages. Lagging consumers must reload history.
    pub fn subscribe(&self) -> broadcast::Receiver<RoomMessageEvent> {
        self.events.subscribe()
    }

    /// Authorize members for reads/writes and the creator for room administration.
    pub async fn authorize(
        &self,
        room_id: &str,
        member_id: &str,
        owner_only: bool,
    ) -> Result<(), RoomCommandError> {
        let details = self.repository.get_room(room_id).await?;
        if details.room.creator_id.as_deref() == Some(member_id)
            || (!owner_only && self.repository.is_member(room_id, member_id).await)
        {
            Ok(())
        } else {
            Err(RoomCommandError::Forbidden)
        }
    }

    /// Keep retry keys for one hour in this process; reject overflow rather than evict accepted keys.
    pub async fn send_message_idempotent(
        &self,
        command: SendMessageCommand,
        key: Option<String>,
    ) -> Result<StoredMessage, RoomCommandError> {
        let Some(key) = key else {
            return self.send_message(command).await;
        };
        if key.trim().is_empty() || key.len() > 128 {
            return Err(RoomCommandError::Validation(
                "clientMessageId must contain 1–128 bytes".to_string(),
            ));
        }
        let mut retries = self.retries.lock().await;
        retries.retain(|_, accepted| accepted.accepted_at.elapsed() < Duration::from_secs(3600));
        let identity = (command.sender.clone(), command.room_id.clone(), key);
        if let Some(accepted) = retries.get(&identity) {
            if accepted.message.text != command.text
                || accepted.message.reply_to != command.reply_to
            {
                return Err(RoomCommandError::Conflict);
            }
            return Ok(accepted.message.clone());
        }
        if retries.len() >= 10_000 {
            return Err(RoomCommandError::ServiceUnavailable);
        }
        let message = self.send_message(command).await?;
        retries.insert(
            identity,
            AcceptedMessage {
                message: message.clone(),
                accepted_at: Instant::now(),
            },
        );
        Ok(message)
    }

    pub async fn active_room_count(&self) -> usize {
        self.repository.active_room_count().await
    }

    pub async fn create_room(&self, command: CreateRoomCommand) -> Result<Room, RoomCommandError> {
        if command.name.trim().is_empty() {
            return Err(RoomCommandError::Validation(
                "room name cannot be empty".to_string(),
            ));
        }

        let room = Room {
            id: format!("room_{}", Uuid::new_v4().simple()),
            name: command.name,
            creator_id: command.creator_id,
            topic: command.topic,
            #[cfg(feature = "multi-tenant")]
            tenant_id: command.tenant_id,
        };

        self.repository.create_room(room).await
    }

    pub async fn send_message(
        &self,
        command: SendMessageCommand,
    ) -> Result<StoredMessage, RoomCommandError> {
        if command.room_id.trim().is_empty()
            || command.sender.trim().is_empty()
            || command.text.trim().is_empty()
        {
            return Err(RoomCommandError::Validation(
                "roomId, sender, and text are required".to_string(),
            ));
        }

        if command.text.len() > MAX_MESSAGE_TEXT_LEN {
            return Err(RoomCommandError::Validation(
                "text exceeds maximum length of 32768 characters".to_string(),
            ));
        }

        if let Some(reply_to) = &command.reply_to {
            let details = self.repository.get_room(&command.room_id).await?;
            if !details
                .messages
                .iter()
                .any(|message| &message.id == reply_to)
            {
                return Err(RoomCommandError::Validation(
                    "replyTo must reference a message in this room".to_string(),
                ));
            }
        }

        let _order = self.message_order.lock().await;
        let message = StoredMessage {
            id: format!("msg_{}", Uuid::new_v4().simple()),
            sender: command.sender,
            text: command.text,
            reply_to: command.reply_to,
        };

        self.repository
            .append_message(command.room_id.clone(), message.clone())
            .await?;
        // Publish plaintext only after storage succeeds. Storage adapters may encrypt their copy.
        let _ = self.events.send(RoomMessageEvent {
            room_id: command.room_id,
            message: message.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        });
        Ok(message)
    }

    pub async fn get_room(&self, id: &str) -> Result<RoomDetails, RoomCommandError> {
        self.repository.get_room(id).await
    }

    pub async fn invite_member(
        &self,
        command: InviteMemberCommand,
    ) -> Result<InviteMemberResult, RoomCommandError> {
        if command.member_id.trim().is_empty() {
            return Err(RoomCommandError::Validation(
                "memberId is required".to_string(),
            ));
        }

        self.repository
            .invite_member(command.room_id, command.member_id)
            .await
    }

    pub async fn list_rooms(&self, limit: usize, offset: usize) -> ListRoomsResult {
        self.repository.list_rooms(limit, offset).await
    }

    pub async fn list_rooms_for(
        &self,
        member_id: &str,
        limit: usize,
        offset: usize,
    ) -> ListRoomsResult {
        let all = self.repository.list_rooms(usize::MAX, 0).await;
        let mut accessible = Vec::new();
        for room in all.rooms {
            if self.authorize(&room.id, member_id, false).await.is_ok() {
                accessible.push(room);
            }
        }
        let total = accessible.len();
        ListRoomsResult {
            rooms: accessible.into_iter().skip(offset).take(limit).collect(),
            total,
        }
    }

    pub async fn delete_room(&self, id: &str) -> Result<(), RoomCommandError> {
        self.repository.delete_room(id).await
    }

    pub async fn export_member_data(&self, member_id: &str) -> MemberDataExport {
        self.repository.export_member_data(member_id).await
    }

    pub async fn delete_member_data(
        &self,
        member_id: &str,
    ) -> Result<MemberDeletionResult, RoomCommandError> {
        self.repository.delete_member_data(member_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::super::infrastructure::InMemoryRoomRepository;
    use super::*;

    fn application_without_encryption() -> RoomApplication {
        RoomApplication::new(Arc::new(InMemoryRoomRepository::without_encryption()))
    }

    #[tokio::test]
    async fn create_room_rejects_blank_name() {
        let application = application_without_encryption();

        let result = application
            .create_room(CreateRoomCommand {
                name: "  ".to_string(),
                creator_id: None,
                topic: None,
                #[cfg(feature = "multi-tenant")]
                tenant_id: None,
            })
            .await;

        assert_eq!(
            result,
            Err(RoomCommandError::Validation(
                "room name cannot be empty".to_string()
            ))
        );
    }

    #[tokio::test]
    async fn send_message_requires_existing_room() {
        let application = application_without_encryption();

        let result = application
            .send_message(SendMessageCommand {
                room_id: "missing".to_string(),
                sender: "member-1".to_string(),
                text: "hello".to_string(),
                reply_to: None,
            })
            .await;

        assert_eq!(result, Err(RoomCommandError::RoomNotFound));
    }

    #[tokio::test]
    async fn create_room_and_send_message_roundtrip() {
        let application = application_without_encryption();
        let room = application
            .create_room(CreateRoomCommand {
                name: "Engineering".to_string(),
                creator_id: None,
                topic: Some("architecture".to_string()),
                #[cfg(feature = "multi-tenant")]
                tenant_id: None,
            })
            .await
            .expect("room should be created");

        let message = application
            .send_message(SendMessageCommand {
                room_id: room.id.clone(),
                sender: "member-1".to_string(),
                text: "hello".to_string(),
                reply_to: None,
            })
            .await
            .expect("message should be stored");

        let details = application
            .get_room(&room.id)
            .await
            .expect("room should be readable");

        assert_eq!(details.room.name, "Engineering");
        assert_eq!(details.messages, vec![message]);
    }

    #[tokio::test]
    async fn delete_member_data_anonymizes_messages_and_removes_membership() {
        let application = application_without_encryption();
        let room = application
            .create_room(CreateRoomCommand {
                name: "Privacy".to_string(),
                creator_id: None,
                topic: None,
                #[cfg(feature = "multi-tenant")]
                tenant_id: None,
            })
            .await
            .expect("room should be created");

        application
            .invite_member(InviteMemberCommand {
                room_id: room.id.clone(),
                member_id: "member-1".to_string(),
            })
            .await
            .expect("member should be invited");
        application
            .send_message(SendMessageCommand {
                room_id: room.id.clone(),
                sender: "member-1".to_string(),
                text: "delete me".to_string(),
                reply_to: None,
            })
            .await
            .expect("message should be stored");

        let deletion = application
            .delete_member_data("member-1")
            .await
            .expect("member data should be deleted");
        let export = application.export_member_data("member-1").await;
        let details = application
            .get_room(&room.id)
            .await
            .expect("room should still exist");

        assert_eq!(deletion.messages, 1);
        assert_eq!(deletion.rooms_created, 0);
        assert!(export.messages.is_empty());
        assert_eq!(details.messages[0].sender, "[deleted]");
        assert_eq!(
            details.messages[0].text,
            "[Content removed per GDPR request]"
        );
    }

    #[tokio::test]
    async fn delete_member_data_removes_rooms_created_by_member() {
        let application = application_without_encryption();
        let room = application
            .create_room(CreateRoomCommand {
                name: "Owned".to_string(),
                creator_id: Some("member-1".to_string()),
                topic: None,
                #[cfg(feature = "multi-tenant")]
                tenant_id: None,
            })
            .await
            .expect("room should be created");

        let deletion = application
            .delete_member_data("member-1")
            .await
            .expect("member data should be deleted");

        assert_eq!(deletion.rooms_created, 1);
        assert!(matches!(
            application.get_room(&room.id).await,
            Err(RoomCommandError::RoomNotFound)
        ));
    }
    async fn owned_room(application: &RoomApplication) -> Room {
        application
            .create_room(CreateRoomCommand {
                name: "synthetic-room".to_string(),
                creator_id: Some("alice".to_string()),
                topic: None,
                #[cfg(feature = "multi-tenant")]
                tenant_id: None,
            })
            .await
            .unwrap()
    }

    fn message_command(room_id: &str, text: &str) -> SendMessageCommand {
        SendMessageCommand {
            room_id: room_id.to_string(),
            sender: "alice".to_string(),
            text: text.to_string(),
            reply_to: None,
        }
    }

    #[tokio::test]
    async fn authorizes_members_and_reserves_administration_for_owner() {
        let application = application_without_encryption();
        let room = owned_room(&application).await;
        assert_eq!(
            application.authorize(&room.id, "bob", false).await,
            Err(RoomCommandError::Forbidden)
        );
        assert_eq!(application.list_rooms_for("bob", 10, 0).await.total, 0);
        application
            .invite_member(InviteMemberCommand {
                room_id: room.id.clone(),
                member_id: "bob".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(application.authorize(&room.id, "bob", false).await, Ok(()));
        assert_eq!(
            application.authorize(&room.id, "bob", true).await,
            Err(RoomCommandError::Forbidden)
        );
        assert_eq!(application.list_rooms_for("bob", 10, 0).await.total, 1);
        application.delete_room(&room.id).await.unwrap();
        assert_eq!(
            application.authorize(&room.id, "bob", false).await,
            Err(RoomCommandError::RoomNotFound)
        );
    }

    #[tokio::test]
    async fn concurrent_retries_commit_once_and_publish_once() {
        let application = application_without_encryption();
        let room = owned_room(&application).await;
        let mut events = application.subscribe();
        let command = message_command(&room.id, "accepted-once");
        let (first, second) = tokio::join!(
            application.send_message_idempotent(command.clone(), Some("retry-1".to_string())),
            application.send_message_idempotent(command, Some("retry-1".to_string())),
        );
        assert_eq!(first.unwrap(), second.unwrap());
        assert_eq!(
            application.get_room(&room.id).await.unwrap().messages.len(),
            1
        );
        assert_eq!(events.try_recv().unwrap().message.text, "accepted-once");
        assert!(events.try_recv().is_err());
        assert_eq!(
            application
                .send_message_idempotent(
                    message_command(&room.id, "different"),
                    Some("retry-1".to_string())
                )
                .await,
            Err(RoomCommandError::Conflict)
        );
    }

    #[tokio::test]
    async fn rejects_invalid_retry_keys_and_recovers_expired_capacity() {
        let application = application_without_encryption();
        let room = owned_room(&application).await;
        for key in [" ".to_string(), "x".repeat(129)] {
            assert!(matches!(
                application
                    .send_message_idempotent(message_command(&room.id, "test"), Some(key))
                    .await,
                Err(RoomCommandError::Validation(_))
            ));
        }
        let message = application
            .send_message(message_command(&room.id, "seed"))
            .await
            .unwrap();
        {
            let mut ledger = application.retries.lock().await;
            for index in 0..10_000 {
                ledger.insert(
                    ("alice".to_string(), room.id.clone(), index.to_string()),
                    AcceptedMessage {
                        message: message.clone(),
                        accepted_at: Instant::now(),
                    },
                );
            }
        }
        assert_eq!(
            application
                .send_message_idempotent(
                    message_command(&room.id, "overflow"),
                    Some("new-key".to_string())
                )
                .await,
            Err(RoomCommandError::ServiceUnavailable)
        );
        for accepted in application.retries.lock().await.values_mut() {
            accepted.accepted_at = Instant::now() - Duration::from_secs(3601);
        }
        assert!(application
            .send_message_idempotent(
                message_command(&room.id, "recovered"),
                Some("new-key".to_string())
            )
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn failed_writes_do_not_consume_keys_or_emit_events_and_replies_stay_in_room() {
        let application = application_without_encryption();
        let room = owned_room(&application).await;
        let other_room = owned_room(&application).await;
        let mut events = application.subscribe();
        assert_eq!(
            application
                .send_message_idempotent(
                    message_command("missing", "test"),
                    Some("retry".to_string())
                )
                .await,
            Err(RoomCommandError::RoomNotFound)
        );
        assert!(application.retries.lock().await.is_empty());
        assert!(events.try_recv().is_err());
        let original = application
            .send_message(message_command(&room.id, "source"))
            .await
            .unwrap();
        let mut reply = message_command(&other_room.id, "reply");
        reply.reply_to = Some(original.id.clone());
        assert!(matches!(
            application.send_message(reply.clone()).await,
            Err(RoomCommandError::Validation(_))
        ));
        reply.room_id = room.id;
        assert_eq!(
            application.send_message(reply).await.unwrap().reply_to,
            Some(original.id)
        );
    }
}
