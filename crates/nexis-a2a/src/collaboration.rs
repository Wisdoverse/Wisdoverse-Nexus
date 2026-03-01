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
    #[default]
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// A portable work unit that can be handed off across agents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub task_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub state: TaskState,
    pub created_at: DateTime<Utc>,
}

impl Task {
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HandoffRequest {
    pub from_agent: AgentId,
    pub to_agent: AgentId,
    pub task: Task,
    pub reason: Option<String>,
}

/// Response to a handoff request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffResponse {
    pub accepted: bool,
    pub message: Option<String>,
}

/// A task prepared for fan-out execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FanOutTask {
    pub task: Task,
    pub target_agents: Vec<AgentId>,
}

/// Result of fan-in aggregation after fan-out execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FanInResult {
    pub completed: Vec<Task>,
    pub failed: Vec<Task>,
}
