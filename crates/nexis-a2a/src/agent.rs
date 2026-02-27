//! Agent types for A2A protocol

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Agent identifier (URI-style: agent://org/team/name)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(String);

impl AgentId {
    pub fn new(uri: impl Into<String>) -> Self {
        Self(uri.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(org: &str, team: &str, name: &str) -> Self {
        Self(format!("agent://{org}/{team}/{name}"))
    }

    pub fn org(&self) -> Option<&str> {
        self.0.strip_prefix("agent://")?.split('/').next()
    }

    pub fn team(&self) -> Option<&str> {
        let parts: Vec<&str> = self.0.strip_prefix("agent://")?.split('/').collect();
        parts.get(1).copied()
    }

    pub fn name(&self) -> Option<&str> {
        let parts: Vec<&str> = self.0.strip_prefix("agent://")?.split('/').collect();
        parts.get(2).copied()
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::parse("default", "default", Uuid::new_v4().to_string())
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
    pub org: String,
    pub environment: String,
    pub version: String,
    pub public_key: Option<String>,
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
    pub capability_id: String,
    pub name: String,
    pub input_schema_ref: Option<String>,
    pub output_schema_ref: Option<String>,
    pub tool_requirements: Vec<String>,
    pub latency_p50_ms: Option<u32>,
    pub latency_p95_ms: Option<u32>,
    pub throughput_qps: Option<f32>,
    pub quality_score: Option<f32>,
    pub compliance_tags: Vec<String>,
}

impl Capability {
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
    pub capabilities: Vec<Capability>,
    pub sla: Option<SlaSpec>,
    pub cost_profile: Option<CostProfile>,
}

impl AgentCapabilities {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Capability> {
        self.capabilities.iter().find(|c| c.name == name)
    }

    pub fn find_by_id(&self, id: &str) -> Option<&Capability> {
        self.capabilities.iter().find(|c| c.capability_id == id)
    }
}

/// SLA specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaSpec {
    pub availability_percent: f32,
    pub max_latency_ms: u32,
    pub support_level: String,
}

/// Cost profile for agent operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostProfile {
    pub tier: String,
    pub cost_per_request: Option<f64>,
    pub cost_per_token: Option<f64>,
}

/// Policy reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRef {
    pub policy_id: String,
    pub policy_type: String,
    pub constraints: HashMap<String, serde_json::Value>,
}

/// Complete agent profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub agent_id: AgentId,
    pub name: String,
    pub identity: AgentIdentity,
    pub endpoint: String,
    pub capabilities: AgentCapabilities,
    pub policy_refs: Vec<PolicyRef>,
    pub status: super::discovery::AgentStatus,
    pub last_seen_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AgentProfile {
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

    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.capabilities.capabilities.push(capability);
        self
    }

    pub fn touch(&mut self) {
        self.last_seen_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
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
}
