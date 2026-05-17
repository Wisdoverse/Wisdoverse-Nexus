//! Legacy GDPR handler module.
//!
//! Privacy HTTP endpoints now live in `crate::privacy::interfaces` and are
//! mounted through `crate::privacy::routes`.

#[cfg(feature = "persistence-sqlx")]
pub use crate::privacy::purge_expired_deletions;
