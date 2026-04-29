//! Task reporting models.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Report period granularity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportPeriod {
    Weekly,
    Monthly,
}

/// Generated task report artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskReport {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub period: ReportPeriod,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub content: serde_json::Value,
    pub generated_at: DateTime<Utc>,
}
