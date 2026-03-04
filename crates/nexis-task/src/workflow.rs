//! Task workflow state machine and transitions.

use crate::task::TaskStatus;
use thiserror::Error;

/// Side effects emitted by workflow transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionSideEffect {
    /// Notify assignee and watchers that task ownership changed.
    NotifyAssignment,
    /// Notify stakeholders that work is blocked.
    NotifyBlocked,
    /// Notify stakeholders that work has completed.
    NotifyCompleted,
    /// Notify stakeholders that work was cancelled.
    NotifyCancelled,
}

/// Result of applying a state transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionResult {
    /// Previous task status.
    pub from: TaskStatus,
    /// New task status.
    pub to: TaskStatus,
    /// Notification or integration side effects emitted by this transition.
    pub side_effects: Vec<TransitionSideEffect>,
}

/// Error returned for invalid transitions.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TransitionError {
    /// Transition is not permitted by the workflow state machine.
    #[error("invalid task transition: {from:?} -> {to:?}")]
    InvalidTransition { from: TaskStatus, to: TaskStatus },
    /// Block transition was requested without a meaningful reason.
    #[error("block reason cannot be empty")]
    InvalidBlockReason,
}

/// Task workflow contract.
pub trait TaskWorkflow {
    /// Validate and apply a transition from `from` to `to`.
    fn transition(
        &self,
        from: TaskStatus,
        to: TaskStatus,
    ) -> Result<TransitionResult, TransitionError>;
}

/// Default workflow implementation for task lifecycle transitions.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultTaskWorkflow;

impl TaskWorkflow for DefaultTaskWorkflow {
    fn transition(
        &self,
        from: TaskStatus,
        to: TaskStatus,
    ) -> Result<TransitionResult, TransitionError> {
        if !is_allowed_transition(from, to) {
            return Err(TransitionError::InvalidTransition { from, to });
        }

        let mut side_effects = Vec::new();
        match to {
            TaskStatus::Assigned => side_effects.push(TransitionSideEffect::NotifyAssignment),
            TaskStatus::Blocked => side_effects.push(TransitionSideEffect::NotifyBlocked),
            TaskStatus::Completed => side_effects.push(TransitionSideEffect::NotifyCompleted),
            TaskStatus::Cancelled => side_effects.push(TransitionSideEffect::NotifyCancelled),
            TaskStatus::Created | TaskStatus::InProgress => {}
        }

        Ok(TransitionResult {
            from,
            to,
            side_effects,
        })
    }
}

fn is_allowed_transition(from: TaskStatus, to: TaskStatus) -> bool {
    matches!(
        (from, to),
        (TaskStatus::Created, TaskStatus::Assigned)
            | (TaskStatus::Created, TaskStatus::Cancelled)
            | (TaskStatus::Assigned, TaskStatus::InProgress)
            | (TaskStatus::Assigned, TaskStatus::Cancelled)
            | (TaskStatus::InProgress, TaskStatus::Completed)
            | (TaskStatus::InProgress, TaskStatus::Blocked)
            | (TaskStatus::InProgress, TaskStatus::Cancelled)
            | (TaskStatus::Blocked, TaskStatus::InProgress)
            | (TaskStatus::Blocked, TaskStatus::Cancelled)
    )
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{DefaultTaskWorkflow, TaskWorkflow, TransitionError, TransitionSideEffect};
    use crate::task::TaskStatus;

    #[test]
    fn allows_happy_path_transitions() {
        let workflow = DefaultTaskWorkflow;

        let first = workflow
            .transition(TaskStatus::Created, TaskStatus::Assigned)
            .expect("created -> assigned should be valid");
        assert_eq!(
            first.side_effects,
            vec![TransitionSideEffect::NotifyAssignment]
        );

        workflow
            .transition(TaskStatus::Assigned, TaskStatus::InProgress)
            .expect("assigned -> in_progress should be valid");

        let last = workflow
            .transition(TaskStatus::InProgress, TaskStatus::Completed)
            .expect("in_progress -> completed should be valid");
        assert_eq!(
            last.side_effects,
            vec![TransitionSideEffect::NotifyCompleted]
        );
    }

    #[test]
    fn rejects_invalid_transition() {
        let workflow = DefaultTaskWorkflow;
        let err = workflow
            .transition(TaskStatus::Created, TaskStatus::Completed)
            .expect_err("created -> completed should be invalid");

        assert_eq!(
            err,
            TransitionError::InvalidTransition {
                from: TaskStatus::Created,
                to: TaskStatus::Completed,
            }
        );
    }

    #[test]
    fn terminal_states_reject_all_outgoing_transitions() {
        let workflow = DefaultTaskWorkflow;
        let targets = [
            TaskStatus::Created,
            TaskStatus::Assigned,
            TaskStatus::InProgress,
            TaskStatus::Blocked,
            TaskStatus::Completed,
            TaskStatus::Cancelled,
        ];

        for to in targets {
            let completed = workflow.transition(TaskStatus::Completed, to);
            assert!(completed.is_err(), "completed -> {to:?} should be invalid");

            let cancelled = workflow.transition(TaskStatus::Cancelled, to);
            assert!(cancelled.is_err(), "cancelled -> {to:?} should be invalid");
        }
    }

    #[test]
    fn blocked_can_resume_and_resume_has_no_side_effect() {
        let workflow = DefaultTaskWorkflow;
        let resumed = workflow
            .transition(TaskStatus::Blocked, TaskStatus::InProgress)
            .expect("blocked -> in_progress should be valid");

        assert!(resumed.side_effects.is_empty());
    }

    #[test]
    fn transition_side_effects_match_target_status() {
        let workflow = DefaultTaskWorkflow;

        let cases = [
            (
                TaskStatus::Created,
                TaskStatus::Assigned,
                vec![TransitionSideEffect::NotifyAssignment],
            ),
            (
                TaskStatus::InProgress,
                TaskStatus::Blocked,
                vec![TransitionSideEffect::NotifyBlocked],
            ),
            (
                TaskStatus::InProgress,
                TaskStatus::Completed,
                vec![TransitionSideEffect::NotifyCompleted],
            ),
            (
                TaskStatus::InProgress,
                TaskStatus::Cancelled,
                vec![TransitionSideEffect::NotifyCancelled],
            ),
            (TaskStatus::Assigned, TaskStatus::InProgress, Vec::new()),
        ];

        for (from, to, expected_side_effects) in cases {
            let result = workflow
                .transition(from, to)
                .expect("transition should be valid for this table case");
            assert_eq!(result.side_effects, expected_side_effects);
        }
    }

    fn is_allowed_transition_for_test(from: TaskStatus, to: TaskStatus) -> bool {
        matches!(
            (from, to),
            (TaskStatus::Created, TaskStatus::Assigned)
                | (TaskStatus::Created, TaskStatus::Cancelled)
                | (TaskStatus::Assigned, TaskStatus::InProgress)
                | (TaskStatus::Assigned, TaskStatus::Cancelled)
                | (TaskStatus::InProgress, TaskStatus::Completed)
                | (TaskStatus::InProgress, TaskStatus::Blocked)
                | (TaskStatus::InProgress, TaskStatus::Cancelled)
                | (TaskStatus::Blocked, TaskStatus::InProgress)
                | (TaskStatus::Blocked, TaskStatus::Cancelled)
        )
    }

    fn expected_effect(to: TaskStatus) -> Vec<TransitionSideEffect> {
        match to {
            TaskStatus::Assigned => vec![TransitionSideEffect::NotifyAssignment],
            TaskStatus::Blocked => vec![TransitionSideEffect::NotifyBlocked],
            TaskStatus::Completed => vec![TransitionSideEffect::NotifyCompleted],
            TaskStatus::Cancelled => vec![TransitionSideEffect::NotifyCancelled],
            TaskStatus::Created | TaskStatus::InProgress => Vec::new(),
        }
    }

    fn arb_status() -> impl Strategy<Value = TaskStatus> {
        prop_oneof![
            Just(TaskStatus::Created),
            Just(TaskStatus::Assigned),
            Just(TaskStatus::InProgress),
            Just(TaskStatus::Blocked),
            Just(TaskStatus::Completed),
            Just(TaskStatus::Cancelled),
        ]
    }

    proptest! {
        #[test]
        fn state_machine_acceptance_matches_matrix(from in arb_status(), to in arb_status()) {
            let workflow = DefaultTaskWorkflow;
            let result = workflow.transition(from, to);

            if is_allowed_transition_for_test(from, to) {
                let transition = result.expect("allowed transition must succeed");
                prop_assert_eq!(transition.from, from);
                prop_assert_eq!(transition.to, to);
                prop_assert_eq!(transition.side_effects, expected_effect(to));
            } else {
                let err = result.expect_err("disallowed transition must fail");
                prop_assert_eq!(err, TransitionError::InvalidTransition { from, to });
            }
        }
    }
}
