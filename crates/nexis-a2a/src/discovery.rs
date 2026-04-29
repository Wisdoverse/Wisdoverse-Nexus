//! Discovery and registry types for A2A agents.

use serde::{Deserialize, Serialize};

use crate::agent::AgentProfile;

/// Agent health and availability state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AgentStatus {
    /// Agent is healthy and ready to accept work.
    #[default]
    Healthy,
    /// Agent is available but currently saturated.
    Busy,
    /// Agent is serving traffic in a degraded state.
    Degraded,
    /// Agent is unavailable.
    Offline,
}

/// Filter criteria used during agent discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AgentFilter {
    /// Optional status to match.
    pub status: Option<AgentStatus>,
    /// Optional capability id or name that must be supported.
    pub required_capability: Option<String>,
    /// Optional owning organization filter.
    pub org: Option<String>,
    /// Optional owning team filter.
    pub team: Option<String>,
}

/// Discovery query parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DiscoveryQuery {
    /// Filter criteria to apply.
    pub filter: AgentFilter,
    /// Maximum number of results to return.
    pub limit: Option<usize>,
    /// Zero-based offset into the matching result set.
    pub offset: Option<usize>,
}

/// Discovery response payload.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryResult {
    /// Matching agent profiles for this query page.
    pub agents: Vec<AgentProfile>,
    /// Total count of matching profiles before pagination.
    pub total: usize,
}

/// In-memory registry for discovered/registered agents.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentRegistry {
    /// In-memory list of known agent profiles.
    pub agents: Vec<AgentProfile>,
}

impl AgentRegistry {
    /// Creates an empty in-memory registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an agent profile in this registry.
    pub fn register(&mut self, profile: AgentProfile) {
        self.agents.push(profile);
    }

    /// Queries the registry using filter and pagination parameters.
    #[must_use]
    pub fn query(&self, query: &DiscoveryQuery) -> DiscoveryResult {
        let filtered = self
            .agents
            .iter()
            .filter(|profile| {
                query
                    .filter
                    .status
                    .is_none_or(|status| profile.status == status)
            })
            .filter(|profile| {
                query
                    .filter
                    .org
                    .as_deref()
                    .is_none_or(|org| profile.identity.org == org)
            })
            .filter(|profile| {
                query
                    .filter
                    .required_capability
                    .as_deref()
                    .is_none_or(|required| {
                        profile.capabilities.find_by_name(required).is_some()
                            || profile.capabilities.find_by_id(required).is_some()
                    })
            })
            .cloned()
            .collect::<Vec<_>>();

        let total = filtered.len();
        let start = query.offset.unwrap_or(0).min(total);
        let end = query
            .limit
            .map_or(total, |limit| start.saturating_add(limit).min(total));

        DiscoveryResult {
            agents: filtered[start..end].to_vec(),
            total,
        }
    }
}
