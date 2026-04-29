//! Task assignment models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Task assignee identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assignee {
    pub user_id: Uuid,
    pub display_name: String,
    pub timezone: Option<String>,
}

/// Assignment record linking task and assignee.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assignment {
    pub id: Uuid,
    pub task_id: Uuid,
    pub assignee: Assignee,
    pub assigned_by: Uuid,
    pub assigned_at: DateTime<Utc>,
}
