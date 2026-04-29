//! Wisdoverse Nexus Calendar - calendar event, conflict, and reminder models.

pub mod conflict;
pub mod error;
pub mod event;
pub mod reminder;
pub mod schedule;

pub use conflict::{detect_overlap, Conflict, ConflictDetector, ConflictSeverity, TimeRange};
pub use error::{CalendarError, CalendarResult};
pub use event::{CalendarEvent, EventAttendee, ResponseStatus};
pub use reminder::{Reminder, ReminderRule};
pub use schedule::{
    find_available_slots, is_within_working_hours, SchedulePreferences, WorkingHours,
};

/// Prelude for common imports.
pub mod prelude {
    pub use crate::conflict::{
        detect_overlap, Conflict, ConflictDetector, ConflictSeverity, TimeRange,
    };
    pub use crate::error::{CalendarError, CalendarResult};
    pub use crate::event::{CalendarEvent, EventAttendee, ResponseStatus};
    pub use crate::reminder::{Reminder, ReminderRule};
    pub use crate::schedule::{
        find_available_slots, is_within_working_hours, SchedulePreferences, WorkingHours,
    };
}
