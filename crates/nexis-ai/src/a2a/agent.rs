//! Agent types for A2A protocol

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Agent identifier (URI-style: <agent://org/team/name>)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(String);

impl AgentId {
    /// Creates an [`AgentId`] from a complete URI.
    #[must_use]
    pub fn new(uri: impl Into<String>) -> Self {
        Self(uri.into())
    }

    /// Returns the underlying agent URI as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Constructs an [`AgentId`] from organization, team, and agent name.
    #[must_use]
    pub fn parse(org: &str, team: &str, name: &str) -> Self {
        Self(format!("agent://{org}/{team}/{name}"))
    }

    /// Returns the organization component from this identifier.
    #[must_use]
    pub fn org(&self) -> Option<&str> {
        self.0.strip_prefix("agent://")?.split('/').next()
    }

    /// Returns the team component from this identifier.
    #[must_use]
    pub fn team(&self) -> Option<&str> {
        let parts: Vec<&str> = self.0.strip_prefix("agent://")?.split('/').collect();
        parts.get(1).copied()
    }

    /// Returns the agent name component from this identifier.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        let parts: Vec<&str> = self.0.strip_prefix("agent://")?.split('/').collect();
        parts.get(2).copied()
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::parse("default", "default", &Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Agent identity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    /// Organization that owns this agent.
    pub org: String,
    /// Deployment environment for this agent.
    pub environment: String,
    /// Agent implementation version.
    pub version: String,
    /// Optional public key used for identity verification.
    pub public_key: Option<String>,
    /// Optional trust domain used for policy or federation boundaries.
    pub trust_domain: Option<String>,
}

impl Default for AgentIdentity {
    fn default() -> Self {
        Self {
            org: "default".to_string(),
            environment: "development".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            public_key: None,
            trust_domain: None,
        }
    }
}

/// A single capability with input/output schemas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Stable capability identifier.
    pub capability_id: String,
    /// Human-readable capability name.
    pub name: String,
    /// Optional JSON schema reference for capability input.
    pub input_schema_ref: Option<String>,
    /// Optional JSON schema reference for capability output.
    pub output_schema_ref: Option<String>,
    /// External tools needed to execute this capability.
    pub tool_requirements: Vec<String>,
    /// P50 latency in milliseconds.
    pub latency_p50_ms: Option<u32>,
    /// P95 latency in milliseconds.
    pub latency_p95_ms: Option<u32>,
    /// Throughput in queries per second.
    pub throughput_qps: Option<f32>,
    /// Optional quality score for ranking or selection.
    pub quality_score: Option<f32>,
    /// Compliance tags attached to this capability.
    pub compliance_tags: Vec<String>,
}

impl Capability {
    /// Creates a capability with required id and name fields.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            capability_id: id.into(),
            name: name.into(),
            input_schema_ref: None,
            output_schema_ref: None,
            tool_requirements: Vec::new(),
            latency_p50_ms: None,
            latency_p95_ms: None,
            throughput_qps: None,
            quality_score: None,
            compliance_tags: Vec::new(),
        }
    }
}

/// Agent capabilities collection
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// Registered capability entries.
    pub capabilities: Vec<Capability>,
    /// Optional service-level objective details.
    pub sla: Option<SlaSpec>,
    /// Optional cost profile for this agent.
    pub cost_profile: Option<CostProfile>,
}

impl AgentCapabilities {
    /// Creates an empty capability collection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a capability and returns the updated collection.
    #[must_use]
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Finds a capability by its human-readable name.
    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<&Capability> {
        self.capabilities.iter().find(|c| c.name == name)
    }

    /// Finds a capability by its stable identifier.
    #[must_use]
    pub fn find_by_id(&self, id: &str) -> Option<&Capability> {
        self.capabilities.iter().find(|c| c.capability_id == id)
    }
}

/// SLA specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaSpec {
    /// Target availability percentage.
    pub availability_percent: f32,
    /// Maximum acceptable latency in milliseconds.
    pub max_latency_ms: u32,
    /// Support tier or level descriptor.
    pub support_level: String,
}

/// Cost profile for agent operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostProfile {
    /// Billing tier name.
    pub tier: String,
    /// Optional per-request cost.
    pub cost_per_request: Option<f64>,
    /// Optional per-token cost for tokenized workloads.
    pub cost_per_token: Option<f64>,
}

/// Policy reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRef {
    /// Policy identifier.
    pub policy_id: String,
    /// Policy category or type.
    pub policy_type: String,
    /// Policy-specific constraints.
    pub constraints: HashMap<String, serde_json::Value>,
}

/// Complete agent profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    /// Stable agent identifier.
    pub agent_id: AgentId,
    /// Human-readable display name.
    pub name: String,
    /// Identity metadata for this agent.
    pub identity: AgentIdentity,
    /// Primary endpoint used to reach the agent.
    pub endpoint: String,
    /// Advertised capabilities and service metadata.
    pub capabilities: AgentCapabilities,
    /// Referenced policy bindings.
    pub policy_refs: Vec<PolicyRef>,
    /// Current availability status.
    pub status: super::discovery::AgentStatus,
    /// Last observed heartbeat timestamp.
    pub last_seen_at: DateTime<Utc>,
    /// Creation timestamp for the profile record.
    pub created_at: DateTime<Utc>,
    /// Additional implementation-specific metadata.
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AgentProfile {
    /// Creates a new agent profile with default identity and healthy status.
    #[must_use]
    pub fn new(agent_id: AgentId, name: impl Into<String>, endpoint: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            agent_id,
            name: name.into(),
            identity: AgentIdentity::default(),
            endpoint: endpoint.into(),
            capabilities: AgentCapabilities::new(),
            policy_refs: Vec::new(),
            status: super::discovery::AgentStatus::Healthy,
            last_seen_at: now,
            created_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Appends a capability and returns the updated profile.
    #[must_use]
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.capabilities.capabilities.push(capability);
        self
    }

    /// Updates `last_seen_at` to the current UTC timestamp.
    pub fn touch(&mut self) {
        self.last_seen_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn agent_id_parse_components() {
        let id = AgentId::parse("acme", "platform", "orchestrator");
        assert_eq!(id.org(), Some("acme"));
        assert_eq!(id.team(), Some("platform"));
        assert_eq!(id.name(), Some("orchestrator"));
    }

    #[test]
    fn agent_profile_builder() {
        let profile = AgentProfile::new(
            AgentId::parse("test", "team", "agent"),
            "Test Agent",
            "https://agent.test.io",
        )
        .with_capability(Capability::new("cap-1", "code-review"));

        assert_eq!(profile.capabilities.capabilities.len(), 1);
        assert_eq!(profile.name, "Test Agent");
    }

    #[test]
    fn capabilities_lookup() {
        let caps = AgentCapabilities::new()
            .with_capability(Capability::new("cap-1", "code-review"))
            .with_capability(Capability::new("cap-2", "security-scan"));

        assert!(caps.find_by_name("code-review").is_some());
        assert!(caps.find_by_id("cap-2").is_some());
        assert!(caps.find_by_name("nonexistent").is_none());
    }

    proptest! {
        #[test]
        fn agent_id_roundtrips_parse_display_and_serde(
            org in "[a-zA-Z0-9][a-zA-Z0-9_-]{0,15}",
            team in "[a-zA-Z0-9][a-zA-Z0-9_-]{0,15}",
            name in "[a-zA-Z0-9][a-zA-Z0-9_-]{0,31}"
        ) {
            let id = AgentId::parse(&org, &team, &name);

            prop_assert_eq!(id.org(), Some(org.as_str()));
            prop_assert_eq!(id.team(), Some(team.as_str()));
            prop_assert_eq!(id.name(), Some(name.as_str()));
            prop_assert_eq!(id.to_string(), format!("agent://{org}/{team}/{name}"));

            let serialized = serde_json::to_string(&id).expect("serialization should succeed");
            let deserialized: AgentId =
                serde_json::from_str(&serialized).expect("deserialization should succeed");
            prop_assert_eq!(deserialized, id);
        }
    }
}
