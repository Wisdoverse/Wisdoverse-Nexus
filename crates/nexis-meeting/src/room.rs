//! Meeting room domain models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Participant;

/// Meeting room lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomState {
    Initializing,
    Active,
    Ended,
}

/// Runtime room configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomConfig {
    pub max_participants: u16,
    pub sfu_enabled: bool,
    pub recording_enabled: bool,
}

/// Meeting room aggregate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingRoom {
    pub id: Uuid,
    pub title: String,
    pub host_id: Uuid,
    pub state: RoomState,
    pub config: RoomConfig,
    pub participants: Vec<Participant>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
}
