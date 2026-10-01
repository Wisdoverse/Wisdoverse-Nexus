//! Explicitly invoked, bounded and permission-scoped room agents.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;

pub use application::{AgentApplication, AgentProvider, ProviderEvent, ProviderRequest, RunActor};
pub use interfaces::{routes, AgentInterfaceState};
