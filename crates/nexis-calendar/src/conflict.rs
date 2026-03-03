//! Conflict detection models.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{CalendarEvent, CalendarResult};

/// Severity of a calendar conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
}

/// Generic time range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeRange {
    /// Build a new range without validating bounds.
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self { start, end }
    }
}

/// Conflict detected between events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conflict {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub member_id: Uuid,
    pub event_a_id: Uuid,
    pub event_b_id: Uuid,
    pub severity: ConflictSeverity,
    pub detected_at: DateTime<Utc>,
}

/// Detect conflicts for a member within a time range.
#[async_trait]
pub trait ConflictDetector {
    async fn detect_conflicts(
        &self,
        member_id: Uuid,
        time_range: TimeRange,
        events: &[CalendarEvent],
    ) -> CalendarResult<Vec<Conflict>>;
}

/// Return overlapping portion of two time ranges, if any.
pub fn detect_overlap(a: TimeRange, b: TimeRange) -> Option<TimeRange> {
    let start = a.start.max(b.start);
    let end = a.end.min(b.end);

    if start < end {
        Some(TimeRange { start, end })
    } else {
        None
    }
}
