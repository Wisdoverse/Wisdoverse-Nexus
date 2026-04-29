//! Tenant model for multi-tenant data isolation.
//!
//! A tenant represents an organization or account that owns workspaces.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Subscription plan types for tenants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Plan {
    /// Free tier with limited resources.
    Free,
    /// Professional tier with more resources.
    Pro,
    /// Enterprise tier with unlimited resources.
    Enterprise,
    /// Custom plan for special arrangements.
    Custom,
}

impl Default for Plan {
    fn default() -> Self {
        Self::Free
    }
}

impl std::fmt::Display for Plan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Plan::Free => write!(f, "free"),
            Plan::Pro => write!(f, "pro"),
            Plan::Enterprise => write!(f, "enterprise"),
            Plan::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for Plan {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "free" => Ok(Plan::Free),
            "pro" => Ok(Plan::Pro),
            "enterprise" => Ok(Plan::Enterprise),
            "custom" => Ok(Plan::Custom),
            _ => Err(format!("Unknown plan: {}", s)),
        }
    }
}

/// Resource limits for a tenant based on their plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantLimits {
    /// Maximum number of members across all workspaces.
    pub max_members: i32,
    /// Maximum number of rooms across all workspaces.
    pub max_rooms: i32,
}

impl Default for TenantLimits {
    fn default() -> Self {
        Self {
            max_members: 100,
            max_rooms: 50,
        }
    }
}

impl From<&Plan> for TenantLimits {
    fn from(plan: &Plan) -> Self {
        match plan {
            Plan::Free => Self {
                max_members: 100,
                max_rooms: 50,
            },
            Plan::Pro => Self {
                max_members: 1000,
                max_rooms: 200,
            },
            Plan::Enterprise => Self {
                max_members: 10_000,
                max_rooms: 1000,
            },
            Plan::Custom => Self {
                max_members: i32::MAX,
                max_rooms: i32::MAX,
            },
        }
    }
}

/// Domain model for a tenant (organization/account).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tenant {
    /// Unique identifier (UUID).
    pub id: Uuid,
    /// Display name of the tenant.
    pub name: String,
    /// URL-friendly identifier (unique).
    pub slug: String,
    /// Subscription plan.
    #[serde(default)]
    pub plan: Plan,
    /// Resource limits based on plan.
    #[serde(default)]
    pub limits: TenantLimits,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

impl Tenant {
    /// Create a new tenant with the given name and slug.
    pub fn new(name: impl Into<String>, slug: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            slug: slug.into(),
            plan: Plan::default(),
            limits: TenantLimits::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a tenant with a specific plan.
    pub fn with_plan(mut self, plan: Plan) -> Self {
        self.limits = TenantLimits::from(&plan);
        self.plan = plan;
        self
    }

    /// Check if the tenant can add more members.
    pub fn can_add_members(&self, current_count: i32) -> bool {
        current_count < self.limits.max_members
    }

    /// Check if the tenant can add more rooms.
    pub fn can_add_rooms(&self, current_count: i32) -> bool {
        current_count < self.limits.max_rooms
    }
}

/// Data required to create a new tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTenant {
    /// Display name.
    pub name: String,
    /// URL-friendly slug.
    pub slug: String,
    /// Optional plan override.
    #[serde(default)]
    pub plan: Option<Plan>,
}

/// Data for updating a tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTenant {
    /// New display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New plan.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<Plan>,
    /// Updated limits (for custom plans).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<TenantLimits>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenant_new_creates_with_defaults() {
        let tenant = Tenant::new("Acme Corp", "acme-corp");
        assert_eq!(tenant.name, "Acme Corp");
        assert_eq!(tenant.slug, "acme-corp");
        assert_eq!(tenant.plan, Plan::Free);
        assert!(tenant.id != Uuid::nil());
    }

    #[test]
    fn tenant_with_plan_updates_limits() {
        let tenant = Tenant::new("Acme Corp", "acme-corp").with_plan(Plan::Pro);
        assert_eq!(tenant.plan, Plan::Pro);
        assert_eq!(tenant.limits.max_members, 1000);
        assert_eq!(tenant.limits.max_rooms, 200);
    }

    #[test]
    fn plan_parsing() {
        assert_eq!("free".parse::<Plan>(), Ok(Plan::Free));
        assert_eq!("PRO".parse::<Plan>(), Ok(Plan::Pro));
        assert!("unknown".parse::<Plan>().is_err());
    }

    #[test]
    fn tenant_can_check_limits() {
        let tenant = Tenant::new("Test", "test");
        assert!(tenant.can_add_members(50));
        assert!(!tenant.can_add_members(100));
        assert!(tenant.can_add_rooms(25));
        assert!(!tenant.can_add_rooms(50));
    }
}
