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
    /// Member identifier.
    pub member_id: Uuid,
    /// Display name at event creation time.
    pub display_name: String,
    /// RSVP state from the attendee.
    pub response_status: ResponseStatus,
    /// Whether attendance is optional.
    pub optional: bool,
}

/// Calendar event entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarEvent {
    /// Event identifier.
    pub id: Uuid,
    /// Owning tenant identifier.
    pub tenant_id: Uuid,
    /// Event organizer identifier.
    pub owner_id: Uuid,
    /// Event title.
    pub title: String,
    /// Event start timestamp (UTC).
    pub start_at: DateTime<Utc>,
    /// Event end timestamp (UTC).
    pub end_at: DateTime<Utc>,
    /// Attendee list.
    pub attendees: Vec<EventAttendee>,
    /// Optional upstream source type (for example `meeting`).
    pub source_type: Option<String>,
    /// Optional upstream source entity ID.
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
