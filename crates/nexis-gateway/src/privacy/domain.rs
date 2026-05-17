//! Privacy domain types for data subject rights.

use chrono::{DateTime, Utc};

/// Data export assembled for a member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataExport {
    pub member: MemberExport,
    pub messages: Vec<MessageExport>,
    pub rooms: Vec<RoomExport>,
    pub exported_at: DateTime<Utc>,
    pub format: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberExport {
    pub id: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageExport {
    pub id: String,
    pub room_id: String,
    pub sender: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomExport {
    pub id: String,
    pub name: String,
    pub topic: Option<String>,
}

/// Result of a confirmed privacy deletion request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletionReceipt {
    pub deleted_at: DateTime<Utc>,
    pub deleted_items: DeletedItems,
    pub retention_until: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DeletedItems {
    pub messages: i64,
    pub rooms_created: i64,
}

pub const EXPORT_FORMAT_JSON: &str = "json";
pub const RETENTION_DAYS: i64 = 30;
