//! Meeting participant and media state models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Role of a participant in a room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantRole {
    Host,
    Member,
    Guest,
    AiAgent,
}

/// Current participant media state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaState {
    pub audio_muted: bool,
    pub video_enabled: bool,
    pub screen_sharing: bool,
}

/// User or agent connected to a meeting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Participant {
    pub id: Uuid,
    pub display_name: String,
    pub role: ParticipantRole,
    pub media: MediaState,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
}
