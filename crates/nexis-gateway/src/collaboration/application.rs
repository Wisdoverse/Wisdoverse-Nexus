//! Application use cases for collaboration gateway routes.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::domain::{
    validate_identifier, validate_required_text, validate_time_window, MAX_CONTENT_LEN,
    MAX_IDENTIFIER_LEN, MAX_NAME_LEN, MAX_TITLE_LEN,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct CollaborationApplication;

impl CollaborationApplication {
    pub const fn new() -> Self {
        Self
    }

    pub fn create_meeting_room(
        self,
        command: CreateMeetingRoomCommand,
    ) -> Result<CreateMeetingRoomResult, CollaborationApplicationError> {
        let _domain_type_marker: Option<nexis_meeting::MeetingRoom> = None;
        let name = validate_required_text("name", &command.name, MAX_NAME_LEN)?;

        Ok(CreateMeetingRoomResult {
            room_id: format!("meeting_{}", Uuid::new_v4().simple()),
            name,
        })
    }

    pub fn join_meeting_room(
        self,
        command: MeetingParticipantCommand,
    ) -> Result<MeetingParticipantResult, CollaborationApplicationError> {
        let _domain_type_marker: Option<nexis_meeting::Participant> = None;
        meeting_participant_result(command)
    }

    pub fn leave_meeting_room(
        self,
        command: MeetingParticipantCommand,
    ) -> Result<MeetingParticipantResult, CollaborationApplicationError> {
        let _domain_type_marker: Option<nexis_meeting::Participant> = None;
        meeting_participant_result(command)
    }

    pub fn create_document(
        self,
        command: CreateDocumentCommand,
    ) -> Result<CreateDocumentResult, CollaborationApplicationError> {
        let _domain_type_marker: Option<nexis_doc::Document> = None;
        let title = validate_required_text("title", &command.title, MAX_TITLE_LEN)?;

        Ok(CreateDocumentResult {
            document_id: format!("doc_{}", Uuid::new_v4().simple()),
            title,
        })
    }

    pub fn sync_document(
        self,
        command: SyncDocumentCommand,
    ) -> Result<DocumentContentResult, CollaborationApplicationError> {
        let _domain_type_marker: Option<nexis_doc::CRDTOperation> = None;
        let document_id =
            validate_identifier("document_id", &command.document_id, MAX_IDENTIFIER_LEN)?;
        let content = validate_required_text("content", &command.content, MAX_CONTENT_LEN)?;

        Ok(DocumentContentResult {
            document_id,
            content,
        })
    }

    pub fn get_document_content(
        self,
        command: GetDocumentContentCommand,
    ) -> Result<DocumentContentResult, CollaborationApplicationError> {
        let _domain_type_marker: Option<nexis_doc::DocSnapshot> = None;
        let document_id =
            validate_identifier("document_id", &command.document_id, MAX_IDENTIFIER_LEN)?;

        Ok(DocumentContentResult {
            document_id,
            content: String::new(),
        })
    }

    pub fn create_task(
        self,
        command: CreateTaskCommand,
    ) -> Result<CreateTaskResult, CollaborationApplicationError> {
        let _domain_type_marker: Option<nexis_task::Task> = None;
        let title = validate_required_text("title", &command.title, MAX_TITLE_LEN)?;

        Ok(CreateTaskResult {
            task_id: format!("task_{}", Uuid::new_v4().simple()),
            title,
        })
    }

    pub fn assign_task(
        self,
        command: AssignTaskCommand,
    ) -> Result<TaskAssignmentResult, CollaborationApplicationError> {
        let _domain_type_marker: Option<nexis_task::Assignment> = None;
        let task_id = validate_identifier("task_id", &command.task_id, MAX_IDENTIFIER_LEN)?;
        let assignee_id =
            validate_identifier("assignee_id", &command.assignee_id, MAX_IDENTIFIER_LEN)?;

        Ok(TaskAssignmentResult {
            task_id,
            assignee_id,
        })
    }

    pub fn complete_task(
        self,
        command: CompleteTaskCommand,
    ) -> Result<CompleteTaskResult, CollaborationApplicationError> {
        let _domain_type_marker: Option<nexis_task::TaskStatus> = None;
        let task_id = validate_identifier("task_id", &command.task_id, MAX_IDENTIFIER_LEN)?;

        Ok(CompleteTaskResult {
            task_id,
            status: "completed".to_string(),
        })
    }

    pub fn create_calendar_event(
        self,
        command: CreateCalendarEventCommand,
    ) -> Result<CreateCalendarEventResult, CollaborationApplicationError> {
        let _domain_type_marker: Option<nexis_calendar::CalendarEvent> = None;
        let title = validate_required_text("title", &command.title, MAX_TITLE_LEN)?;
        validate_time_window(command.starts_at, command.ends_at)?;
        let _time_range = nexis_calendar::TimeRange::new(command.starts_at, command.ends_at);

        Ok(CreateCalendarEventResult {
            event_id: format!("event_{}", Uuid::new_v4().simple()),
            title,
        })
    }

    pub fn check_calendar_conflicts(
        self,
        command: ConflictCheckCommand,
    ) -> Result<ConflictCheckResult, CollaborationApplicationError> {
        let _domain_type_marker: Option<nexis_calendar::Conflict> = None;
        validate_time_window(command.starts_at, command.ends_at)?;

        Ok(ConflictCheckResult {
            has_conflicts: false,
        })
    }
}

fn meeting_participant_result(
    command: MeetingParticipantCommand,
) -> Result<MeetingParticipantResult, CollaborationApplicationError> {
    let room_id = validate_identifier("room_id", &command.room_id, MAX_IDENTIFIER_LEN)?;
    let user_id = validate_identifier("user_id", &command.user_id, MAX_IDENTIFIER_LEN)?;

    Ok(MeetingParticipantResult { room_id, user_id })
}

#[derive(Debug, Clone)]
pub struct CreateMeetingRoomCommand {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct CreateMeetingRoomResult {
    pub room_id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct MeetingParticipantCommand {
    pub room_id: String,
    pub user_id: String,
}

#[derive(Debug, Clone)]
pub struct MeetingParticipantResult {
    pub room_id: String,
    pub user_id: String,
}

#[derive(Debug, Clone)]
pub struct CreateDocumentCommand {
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct CreateDocumentResult {
    pub document_id: String,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct SyncDocumentCommand {
    pub document_id: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct GetDocumentContentCommand {
    pub document_id: String,
}

#[derive(Debug, Clone)]
pub struct DocumentContentResult {
    pub document_id: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct CreateTaskCommand {
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct CreateTaskResult {
    pub task_id: String,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct AssignTaskCommand {
    pub task_id: String,
    pub assignee_id: String,
}

#[derive(Debug, Clone)]
pub struct TaskAssignmentResult {
    pub task_id: String,
    pub assignee_id: String,
}

#[derive(Debug, Clone)]
pub struct CompleteTaskCommand {
    pub task_id: String,
}

#[derive(Debug, Clone)]
pub struct CompleteTaskResult {
    pub task_id: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct CreateCalendarEventCommand {
    pub title: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CreateCalendarEventResult {
    pub event_id: String,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct ConflictCheckCommand {
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub struct ConflictCheckResult {
    pub has_conflicts: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum CollaborationApplicationError {
    #[error("{0}")]
    Validation(String),
}

impl From<super::domain::CollaborationValidationError> for CollaborationApplicationError {
    fn from(error: super::domain::CollaborationValidationError) -> Self {
        Self::Validation(error.into_message())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_meeting_room_trims_valid_name() {
        let result = CollaborationApplication::new()
            .create_meeting_room(CreateMeetingRoomCommand {
                name: " daily ".to_string(),
            })
            .expect("meeting room should be created");

        assert!(result.room_id.starts_with("meeting_"));
        assert_eq!(result.name, "daily");
    }

    #[test]
    fn create_calendar_event_rejects_invalid_time_window() {
        let starts_at = "2026-03-04T11:00:00Z".parse().unwrap();
        let ends_at = "2026-03-04T10:00:00Z".parse().unwrap();

        let result =
            CollaborationApplication::new().create_calendar_event(CreateCalendarEventCommand {
                title: "Design review".to_string(),
                starts_at,
                ends_at,
            });

        assert!(matches!(
            result,
            Err(CollaborationApplicationError::Validation(message))
                if message == "starts_at must be earlier than ends_at"
        ));
    }
}
