//! Meeting recording models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of media track included in recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackType {
    Audio,
    Video,
    Screen,
    Mixed,
}

/// Recording lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingState {
    Pending,
    Recording,
    Processing,
    Ready,
    Failed,
}

/// Meeting recording metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recording {
    pub id: Uuid,
    pub room_id: Uuid,
    pub track_type: TrackType,
    pub state: RecordingState,
    pub storage_url: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
}
