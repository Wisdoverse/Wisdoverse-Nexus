//! Application use cases for privacy and GDPR data subject rights.

use chrono::{Duration, Utc};
use tracing::info;

use crate::rooms::{RoomApplication, RoomCommandError};

use super::domain::{
    DataExport, DeletedItems, DeletionReceipt, MemberExport, MessageExport, RoomExport,
    EXPORT_FORMAT_JSON, RETENTION_DAYS,
};

#[derive(Clone)]
pub struct PrivacyApplication {
    rooms: RoomApplication,
}

impl PrivacyApplication {
    pub fn new(rooms: RoomApplication) -> Self {
        Self { rooms }
    }

    pub async fn export_member_data(&self, member_id: &str) -> DataExport {
        let export = self.rooms.export_member_data(member_id).await;
        let now = Utc::now();
        let messages = export
            .messages
            .into_iter()
            .map(|record| MessageExport {
                id: record.message.id,
                room_id: record.room_id,
                sender: record.message.sender,
                content: record.message.text,
            })
            .collect();
        let rooms = export
            .rooms
            .into_iter()
            .map(|room| RoomExport {
                id: room.id,
                name: room.name,
                topic: room.topic,
            })
            .collect();

        DataExport {
            member: MemberExport {
                id: member_id.to_string(),
                email: format!("{member_id}@nexis.local"),
                created_at: now,
            },
            messages,
            rooms,
            exported_at: now,
            format: EXPORT_FORMAT_JSON.to_string(),
        }
    }

    pub async fn delete_member_data(
        &self,
        command: DeleteMemberDataCommand,
    ) -> Result<DeletionReceipt, PrivacyApplicationError> {
        if !command.confirm {
            return Err(PrivacyApplicationError::ConfirmationRequired);
        }

        info!(
            "Processing GDPR deletion for member: {}, reason: {:?}",
            command.member_id, command.reason
        );

        let deleted_at = Utc::now();
        let deletion = self
            .rooms
            .delete_member_data(&command.member_id)
            .await
            .map_err(PrivacyApplicationError::from)?;

        Ok(DeletionReceipt {
            deleted_at,
            deleted_items: DeletedItems {
                messages: deletion.messages,
                rooms_created: deletion.rooms_created,
            },
            retention_until: deleted_at + Duration::days(RETENTION_DAYS),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteMemberDataCommand {
    pub member_id: String,
    pub confirm: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PrivacyApplicationError {
    #[error("deletion requires explicit confirmation")]
    ConfirmationRequired,
    #[error("privacy service unavailable")]
    ServiceUnavailable,
    #[error("unexpected privacy application error")]
    Unexpected,
}

impl From<RoomCommandError> for PrivacyApplicationError {
    fn from(error: RoomCommandError) -> Self {
        match error {
            RoomCommandError::ServiceUnavailable => Self::ServiceUnavailable,
            RoomCommandError::Validation(_) | RoomCommandError::RoomNotFound => Self::Unexpected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rooms::{CreateRoomCommand, SendMessageCommand};

    #[tokio::test]
    async fn delete_member_data_requires_confirmation() {
        let application = PrivacyApplication::new(RoomApplication::default());

        let result = application
            .delete_member_data(DeleteMemberDataCommand {
                member_id: "alice".to_string(),
                confirm: false,
                reason: None,
            })
            .await;

        assert_eq!(result, Err(PrivacyApplicationError::ConfirmationRequired));
    }

    #[tokio::test]
    async fn export_member_data_maps_room_context() {
        let rooms = RoomApplication::default();
        let room = rooms
            .create_room(CreateRoomCommand {
                name: "General".to_string(),
                creator_id: Some("alice".to_string()),
                topic: Some("Announcements".to_string()),
                #[cfg(feature = "multi-tenant")]
                tenant_id: None,
            })
            .await
            .expect("room should be created");
        rooms
            .send_message(SendMessageCommand {
                room_id: room.id,
                sender: "alice".to_string(),
                text: "hello".to_string(),
                reply_to: None,
            })
            .await
            .expect("message should be stored");

        let application = PrivacyApplication::new(rooms);
        let export = application.export_member_data("alice").await;

        assert_eq!(export.member.id, "alice");
        assert_eq!(export.member.email, "alice@nexis.local");
        assert_eq!(export.messages.len(), 1);
        assert_eq!(export.messages[0].content, "hello");
        assert_eq!(export.rooms.len(), 1);
        assert_eq!(export.format, EXPORT_FORMAT_JSON);
    }
}
