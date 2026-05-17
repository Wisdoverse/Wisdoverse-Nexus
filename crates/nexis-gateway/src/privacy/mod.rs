//! Privacy bounded context.

mod application;
mod domain;
mod infrastructure;
mod interfaces;

pub use application::{DeleteMemberDataCommand, PrivacyApplication, PrivacyApplicationError};
pub use domain::{
    DataExport, DeletedItems, DeletionReceipt, MemberExport, MessageExport, RoomExport,
};
#[cfg(feature = "persistence-sqlx")]
pub use infrastructure::purge_expired_deletions;
pub use interfaces::{routes, PrivacyInterfaceState};
