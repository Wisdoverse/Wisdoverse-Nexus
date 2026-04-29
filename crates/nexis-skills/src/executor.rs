//! Skill execution models.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

static EXECUTION_SEQ: AtomicU64 = AtomicU64::new(1);

/// Runtime context passed to skill executors.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Optional tenant or workspace identifier.
    pub scope_id: Option<String>,
    /// Optional requesting agent identifier.
    pub agent_id: Option<String>,
    /// Arbitrary key-value context.
    pub values: HashMap<String, Value>,
}

/// Current state of skill execution.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}

/// Materialized execution record for an invoked skill.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillExecution {
    /// Unique execution identifier.
    pub id: String,
    /// Skill id or name used for execution.
    pub skill: String,
    /// Serialized execution input.
    pub input: Value,
    /// Serialized execution output, when available.
    pub output: Option<Value>,
    /// Current execution status.
    pub status: ExecutionStatus,
    /// Optional execution error message.
    pub error: Option<String>,
    /// Execution start timestamp in unix milliseconds.
    pub started_at_ms: u128,
    /// Execution completion timestamp in unix milliseconds.
    pub completed_at_ms: Option<u128>,
}

impl SkillExecution {
    /// Create a pending execution record.
    #[must_use]
    pub fn new(skill: impl Into<String>, input: Value) -> Self {
        let sequence = EXECUTION_SEQ.fetch_add(1, Ordering::Relaxed);
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis());
        Self {
            id: format!("exec-{sequence}"),
            skill: skill.into(),
            input,
            output: None,
            status: ExecutionStatus::Pending,
            error: None,
            started_at_ms: now_ms,
            completed_at_ms: None,
        }
    }
}
