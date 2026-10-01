//! Governed agent-run state and supported policy, independent of runtime adapters.

use serde::{Deserialize, Serialize};

pub const CONTRACT_VERSION: &str = "1.0";
pub const AGENT_ID: &str = "nexis:ai:room-assistant";
pub const MAX_OUTPUT_TOKENS: u32 = 4096;
pub const MAX_CONTEXT_BYTES: usize = 32_768;
pub const MAX_CONTEXT_MESSAGES: usize = 20;
pub const MAX_EVENTS: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl RunStatus {
    pub fn terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceMessage {
    pub room_id: String,
    pub message_id: String,
    pub sender: String,
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    /// Provider-reported counts; absent means unknown, never an estimated count.
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub output_bytes: usize,
    pub tool_calls: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunEvent {
    pub sequence: u64,
    pub kind: String,
    pub text: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRun {
    pub contract_version: String,
    pub id: String,
    pub room_id: String,
    pub invoked_by: String,
    pub agent_id: String,
    pub agent_name: String,
    pub trace_id: String,
    pub status: RunStatus,
    pub answer: String,
    pub source_message_ids: Vec<String>,
    pub usage: Usage,
    pub max_output_tokens: u32,
    /// UTF-8 byte cap conservatively bounds generated tokens, independently of provider compliance.
    pub max_output_bytes: usize,
    pub deadline_ms: u64,
    pub created_at: i64,
    pub finished_at: Option<i64>,
    pub latency_ms: Option<u64>,
    pub error_code: Option<String>,
    pub events: Vec<RunEvent>,
}

impl AgentRun {
    pub fn event(&mut self, kind: &str, text: Option<String>, timestamp: i64) {
        self.events.push(RunEvent {
            sequence: self.events.last().map_or(1, |event| event.sequence + 1),
            kind: kind.to_string(),
            text,
            timestamp,
        });
    }

    pub fn finish(&mut self, status: RunStatus, code: Option<&str>, timestamp: i64) {
        if self.status.terminal() {
            return;
        }
        self.status = status;
        self.error_code = code.map(ToString::to_string);
        self.finished_at = Some(timestamp);
        self.latency_ms = u64::try_from(timestamp.saturating_sub(self.created_at)).ok();
        let kind = match status {
            RunStatus::Completed => "completed",
            RunStatus::Failed => "failed",
            RunStatus::Cancelled => "cancelled",
            _ => return,
        };
        self.event(kind, None, timestamp);
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InvokeAgent {
    pub client_run_id: String,
    pub prompt: String,
    #[serde(default)]
    pub source_message_ids: Vec<String>,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default = "default_tokens")]
    pub max_output_tokens: u32,
    #[serde(default = "default_deadline")]
    pub deadline_ms: u64,
}

const fn default_tokens() -> u32 {
    1024
}
const fn default_deadline() -> u64 {
    60_000
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    InvalidRequest,
    Forbidden,
    NotFound,
    Disabled,
    Capacity,
    Provider,
    Budget,
    ToolDenied,
    Timeout,
    Conflict,
    UnsupportedMode,
}

impl RunError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest => "INVALID_AGENT_REQUEST",
            Self::Forbidden => "AGENT_ACCESS_DENIED",
            Self::NotFound => "AGENT_RUN_NOT_FOUND",
            Self::Disabled => "AGENT_DISABLED",
            Self::Capacity => "AGENT_CAPACITY_EXCEEDED",
            Self::Provider => "AGENT_PROVIDER_FAILED",
            Self::Budget => "AGENT_BUDGET_EXCEEDED",
            Self::ToolDenied => "AGENT_TOOL_DENIED",
            Self::Timeout => "AGENT_TIMEOUT",
            Self::Conflict => "AGENT_RETRY_CONFLICT",
            Self::UnsupportedMode => "AGENT_UNSUPPORTED_TENANCY",
        }
    }
}

impl std::fmt::Display for RunError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for RunError {}
