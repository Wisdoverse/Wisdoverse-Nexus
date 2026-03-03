//! Scheduling helpers.

use chrono::{DateTime, Duration, NaiveTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{CalendarEvent, TimeRange};

/// Member scheduling preferences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchedulePreferences {
    pub working_hours: WorkingHours,
    pub slot_duration: Duration,
}

/// Working-hour window configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkingHours {
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub timezone: String,
}

/// Check if a UTC timestamp falls within configured working hours.
pub fn is_within_working_hours(at: DateTime<Utc>, working_hours: &WorkingHours) -> bool {
    let _timezone = &working_hours.timezone;
    let time = at.time();

    if working_hours.start_time <= working_hours.end_time {
        time >= working_hours.start_time && time < working_hours.end_time
    } else {
        // Overnight schedule like 22:00-06:00.
        time >= working_hours.start_time || time < working_hours.end_time
    }
}

/// Placeholder for future slot-finding implementation.
pub fn find_available_slots(
    _range: TimeRange,
    _existing_events: &[CalendarEvent],
    _preferences: &SchedulePreferences,
) -> Vec<TimeRange> {
    Vec::new()
}
