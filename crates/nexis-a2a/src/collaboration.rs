//! Collaboration primitives for multi-agent workflows.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agent::AgentId;

/// Collaboration topology for a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CollaborationMode {
    /// One agent delegates work to another.
    Handoff,
    /// One task is fanned out to many agents and merged back.
    #[default]
    FanOutFanIn,
    /// Work is processed in staged sequence.
    Pipeline,
}

/// Role of an agent in collaboration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CollaborationRole {
    /// Agent coordinating or delegating work.
    #[default]
    Coordinator,
    /// Agent executing a delegated task.
    Worker,
    /// Agent reviewing or validating outcomes.
    Reviewer,
}

/// Lifecycle state for a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TaskState {
    /// Task has been created but not started.
    #[default]
    Pending,
    /// Task is actively being processed.
    InProgress,
    /// Task completed successfully.
    Completed,
    /// Task terminated with failure.
    Failed,
}

/// A portable work unit that can be handed off across agents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier.
    pub task_id: Uuid,
    /// Short task title.
    pub title: String,
    /// Optional task details.
    pub description: Option<String>,
    /// Current lifecycle state.
    pub state: TaskState,
    /// Task creation timestamp in UTC.
    pub created_at: DateTime<Utc>,
}

impl Task {
    /// Creates a new pending task with a generated identifier.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            task_id: Uuid::new_v4(),
            title: title.into(),
            description: None,
            state: TaskState::Pending,
            created_at: Utc::now(),
        }
    }
}

/// Request to hand a task from one agent to another.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffRequest {
    /// Delegating agent.
    pub from_agent: AgentId,
    /// Target agent.
    pub to_agent: AgentId,
    /// Task being delegated.
    pub task: Task,
    /// Optional reason for the delegation decision.
    pub reason: Option<String>,
}

/// Response to a handoff request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffResponse {
    /// Whether the receiving agent accepted the task.
    pub accepted: bool,
    /// Optional context message about the acceptance decision.
    pub message: Option<String>,
}

/// A task prepared for fan-out execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FanOutTask {
    /// Task to execute across multiple agents.
    pub task: Task,
    /// Agent ids selected as fan-out targets.
    pub target_agents: Vec<AgentId>,
}

/// Result of fan-in aggregation after fan-out execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FanInResult {
    /// Tasks completed successfully.
    pub completed: Vec<Task>,
    /// Tasks that failed during fan-out execution.
    pub failed: Vec<Task>,
}
