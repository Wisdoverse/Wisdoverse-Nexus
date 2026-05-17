//! Application service for room and message use cases.

use std::sync::Arc;

use async_trait::async_trait;
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
    RoomNotFound,
    ServiceUnavailable,
}

#[async_trait]
pub trait RoomRepository: Send + Sync {
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
}

impl RoomApplication {
    pub fn new(repository: Arc<dyn RoomRepository>) -> Self {
        Self { repository }
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

        let message = StoredMessage {
            id: format!("msg_{}", Uuid::new_v4().simple()),
            sender: command.sender,
            text: command.text,
            reply_to: command.reply_to,
        };

        self.repository
            .append_message(command.room_id, message)
            .await
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
}
