//! # nexis-billing
//!
//! Async billing & request logging module for Wisdoverse Nexus.
//! Uses SQLite (WAL mode) with `spawn_blocking` wrapper for non-blocking writes.

mod db;
mod logger;
mod pricing;

pub use logger::{RequestLogEntry, RequestLogger, RequestStatus};
pub use pricing::{calculate_cost, get_pricing, ModelPricing};
