//! Wisdoverse Nexus Task - task lifecycle, assignment, and reporting models.

pub mod assignment;
pub mod error;
pub mod reminder;
pub mod report;
pub mod task;
pub mod workflow;

pub use assignment::{Assignee, Assignment};
pub use error::{TaskError, TaskResult};
pub use reminder::{InMemoryReminderService, Reminder, ReminderService, ReminderType};
pub use report::{ReportPeriod, TaskReport};
pub use task::{Task, TaskPriority, TaskSource, TaskStatus};
pub use workflow::{
    DefaultTaskWorkflow, TaskWorkflow, TransitionError, TransitionResult, TransitionSideEffect,
};

/// Prelude for common imports.
pub mod prelude {
    pub use crate::assignment::{Assignee, Assignment};
    pub use crate::error::{TaskError, TaskResult};
    pub use crate::reminder::{InMemoryReminderService, Reminder, ReminderService, ReminderType};
    pub use crate::report::{ReportPeriod, TaskReport};
    pub use crate::task::{Task, TaskPriority, TaskSource, TaskStatus};
    pub use crate::workflow::{
        DefaultTaskWorkflow, TaskWorkflow, TransitionError, TransitionResult, TransitionSideEffect,
    };
}
