//! Calendar event domain models.

use chrono::{DateTime, Duration, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Attendee response status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    NeedsAction,
    Accepted,
    Declined,
    Tentative,
}

/// Event attendee metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventAttendee {
    pub member_id: Uuid,
    pub display_name: String,
    pub response_status: ResponseStatus,
    pub optional: bool,
}

/// Calendar event entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub attendees: Vec<EventAttendee>,
    pub source_type: Option<String>,
    pub source_ref_id: Option<Uuid>,
}

impl CalendarEvent {
    /// Whether two events overlap.
    pub fn overlaps_with(&self, other: &CalendarEvent) -> bool {
        self.start_at < other.end_at && other.start_at < self.end_at
    }

    /// Event duration.
    pub fn duration(&self) -> Duration {
        self.end_at - self.start_at
    }

    /// True when event spans one or more full days and is aligned to midnight UTC.
    pub fn is_all_day(&self) -> bool {
        let duration_secs = self.duration().num_seconds();
        let midnight = NaiveTime::MIN;

        self.start_at.time() == midnight
            && self.end_at.time() == midnight
            && duration_secs > 0
            && duration_secs % 86_400 == 0
    }
}
