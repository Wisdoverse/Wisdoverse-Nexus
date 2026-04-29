//! Meeting summary and action item models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Follow-up action generated from a meeting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionItem {
    pub id: Uuid,
    pub text: String,
    pub assignee_id: Option<Uuid>,
    pub due_at: Option<DateTime<Utc>>,
    pub completed: bool,
}

/// Post-meeting summary artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeetingSummary {
    pub id: Uuid,
    pub room_id: Uuid,
    pub summary: String,
    pub action_items: Vec<ActionItem>,
    pub generated_at: DateTime<Utc>,
}
