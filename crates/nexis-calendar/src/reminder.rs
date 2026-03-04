//! Reminder and scheduling rule models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Reminder rule for an event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReminderRule {
    /// Fire reminder this many minutes before event start.
    pub minutes_before: u32,
    /// Additional repeats after the first reminder fire.
    pub repeat_count: u8,
}

/// Reminder instance for event delivery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reminder {
    /// Reminder identifier.
    pub id: Uuid,
    /// Event this reminder belongs to.
    pub event_id: Uuid,
    /// Rule controlling reminder timing and repeats.
    pub rule: ReminderRule,
    /// Next planned trigger timestamp.
    pub next_trigger_at: DateTime<Utc>,
    /// Last successful trigger timestamp.
    pub last_triggered_at: Option<DateTime<Utc>>,
}
