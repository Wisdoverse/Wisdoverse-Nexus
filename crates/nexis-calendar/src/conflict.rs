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

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};

    use super::{detect_overlap, TimeRange};

    fn at(hour: u32, minute: u32) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 15, hour, minute, 0)
            .single()
            .expect("fixed timestamp should be valid")
    }

    #[test]
    fn detect_overlap_excludes_touching_boundaries() {
        let a = TimeRange::new(at(9, 0), at(10, 0));
        let b = TimeRange::new(at(10, 0), at(11, 0));

        assert_eq!(detect_overlap(a, b), None);
        assert_eq!(detect_overlap(b, a), None);
    }

    #[test]
    fn detect_overlap_returns_inner_range_for_containment() {
        let outer = TimeRange::new(at(8, 0), at(12, 0));
        let inner = TimeRange::new(at(9, 30), at(10, 15));

        let overlap = detect_overlap(outer, inner).expect("contained range should overlap");
        assert_eq!(overlap, inner);
    }

    #[test]
    fn detect_overlap_is_symmetric_for_partial_overlap() {
        let left = TimeRange::new(at(9, 0), at(10, 45));
        let right = TimeRange::new(at(10, 0), at(11, 0));

        let forward = detect_overlap(left, right).expect("ranges should overlap");
        let reverse = detect_overlap(right, left).expect("ranges should overlap");

        assert_eq!(forward, reverse);
        assert_eq!(forward.start, at(10, 0));
        assert_eq!(forward.end, at(10, 45));
        assert_eq!(forward.end - forward.start, Duration::minutes(45));
    }
}
