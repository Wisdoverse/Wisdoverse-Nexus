//! Discovery and registry types for A2A agents.

use serde::{Deserialize, Serialize};

use crate::agent::AgentProfile;

/// Agent health and availability state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AgentStatus {
    #[default]
    Healthy,
    Busy,
    Degraded,
    Offline,
}

/// Filter criteria used during agent discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AgentFilter {
    pub status: Option<AgentStatus>,
    pub required_capability: Option<String>,
    pub org: Option<String>,
    pub team: Option<String>,
}

/// Discovery query parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DiscoveryQuery {
    pub filter: AgentFilter,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Discovery response payload.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryResult {
    pub agents: Vec<AgentProfile>,
    pub total: usize,
}

/// In-memory registry for discovered/registered agents.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentRegistry {
    pub agents: Vec<AgentProfile>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, profile: AgentProfile) {
        self.agents.push(profile);
    }

    pub fn query(&self, query: &DiscoveryQuery) -> DiscoveryResult {
        let filtered = self
            .agents
            .iter()
            .filter(|profile| {
                if let Some(status) = query.filter.status {
                    profile.status == status
                } else {
                    true
                }
            })
            .filter(|profile| {
                if let Some(org) = query.filter.org.as_deref() {
                    profile.identity.org == org
                } else {
                    true
                }
            })
            .filter(|profile| {
                if let Some(required) = query.filter.required_capability.as_deref() {
                    profile.capabilities.find_by_name(required).is_some()
                        || profile.capabilities.find_by_id(required).is_some()
                } else {
                    true
                }
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
