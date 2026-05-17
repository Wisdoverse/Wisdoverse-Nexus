//! Collaboration bounded context.

mod application;
mod domain;
mod interfaces;

pub use domain::{
    CollaborationRateLimitKey, CollaborationRateLimitPolicy, CollaborationRateLimitScope,
};
pub use interfaces::routes;
