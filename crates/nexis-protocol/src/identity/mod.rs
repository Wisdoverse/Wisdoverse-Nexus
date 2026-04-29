//! Identity domain extensions for Wisdoverse Nexus.

pub use crate::MemberId;
use serde::{Deserialize, Serialize};

/// Unique identifier for a user
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub String);

impl UserId {
    /// Create a new UserId
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the inner string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// User role in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    /// Regular user
    User,
    /// Administrator
    Admin,
    /// System user
    System,
}

impl UserRole {
    /// Check if role has admin privileges
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::System)
    }
}

/// Unique identifier for an AI agent
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl AgentId {
    /// Create a new AgentId
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the inner string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Identity {
    pub id: MemberId,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

impl Identity {
    pub fn new(id: MemberId) -> Self {
        Self {
            id,
            display_name: None,
            avatar_url: None,
        }
    }

    pub fn with_display_name(mut self, name: String) -> Self {
        self.display_name = Some(name);
        self
    }

    pub fn with_avatar(mut self, url: String) -> Self {
        self.avatar_url = Some(url);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MemberId, MemberType};

    #[test]
    fn identity_new_has_no_optional_fields() {
        let id = MemberId::new(MemberType::Human, "alice").unwrap();
        let identity = Identity::new(id);
        assert!(identity.display_name.is_none());
        assert!(identity.avatar_url.is_none());
    }

    #[test]
    fn identity_builder_chain() {
        let id = MemberId::new(MemberType::Agent, "bot-1").unwrap();
        let identity = Identity::new(id)
            .with_display_name("Bot".into())
            .with_avatar("https://img.url/avatar.png".into());

        assert_eq!(identity.display_name.as_deref(), Some("Bot"));
        assert_eq!(
            identity.avatar_url.as_deref(),
            Some("https://img.url/avatar.png")
        );
    }

    #[test]
    fn identity_clone_preserves_all_fields() {
        let id = MemberId::new(MemberType::Ai, "gpt-4").unwrap();
        let original = Identity::new(id.clone()).with_display_name("AI".into());
        let cloned = original.clone();
        assert_eq!(cloned.id, id);
        assert_eq!(cloned.display_name, original.display_name);
    }

    #[test]
    fn user_id_new_and_display() {
        let id = UserId::new("alice@example.com");
        assert_eq!(id.as_str(), "alice@example.com");
        assert_eq!(format!("{}", id), "alice@example.com");
    }

    #[test]
    fn user_role_is_admin() {
        assert!(!UserRole::User.is_admin());
        assert!(UserRole::Admin.is_admin());
        assert!(UserRole::System.is_admin());
    }

    #[test]
    fn agent_id_new_and_display() {
        let id = AgentId::new("gpt-4-turbo");
        assert_eq!(id.as_str(), "gpt-4-turbo");
        assert_eq!(format!("{}", id), "gpt-4-turbo");
    }
}
