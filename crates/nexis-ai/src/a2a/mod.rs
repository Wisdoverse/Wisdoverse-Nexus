//! Nexis A2A (Agent-to-Agent) Protocol
//!
//! This crate provides:
//! - Agent identity and capability management
//! - Agent discovery and registration
//! - Inter-agent communication (messages, envelopes)
//! - Multi-agent collaboration (handoff, fan-out/fan-in)
#![allow(clippy::multiple_crate_versions)]

pub mod agent;
pub mod collaboration;
pub mod communication;
pub mod discovery;
pub mod error;

pub use agent::{AgentCapabilities, AgentId, AgentIdentity, AgentProfile, Capability};
pub use collaboration::{
    CollaborationMode, CollaborationRole, FanInResult, FanOutTask, HandoffRequest, HandoffResponse,
    Task, TaskState,
};
pub use communication::{
    AuthContext, Envelope, ErrorInfo, Message, MessageId, MessageType, Payload,
};
pub use discovery::{AgentFilter, AgentRegistry, AgentStatus, DiscoveryQuery, DiscoveryResult};
pub use error::{A2AError, A2AResult};

/// Supported A2A protocol version implemented by this crate.
pub const PROTOCOL_VERSION: &str = "a2a.v1";

/// Convenient re-exports for commonly used A2A types.
pub mod prelude {
    pub use super::agent::{AgentCapabilities, AgentId, AgentIdentity, AgentProfile, Capability};
    pub use super::collaboration::{
        CollaborationMode, CollaborationRole, FanInResult, FanOutTask, HandoffRequest,
        HandoffResponse, Task, TaskState,
    };
    pub use super::communication::{Envelope, Message, MessageId, MessageType, Payload};
    pub use super::discovery::{AgentFilter, AgentRegistry, AgentStatus, DiscoveryQuery};
    pub use super::error::{A2AError, A2AResult};
    pub use super::PROTOCOL_VERSION;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_version_is_valid() {
        assert!(PROTOCOL_VERSION.starts_with("a2a.v"));
    }
}
