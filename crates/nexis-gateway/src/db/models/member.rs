//! Member model with workspace-scoped identity.
//!
//! A member belongs to a workspace and can be a human, AI agent, or bot.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Types of members in the system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemberType {
    /// Human user.
    Human,
    /// AI agent (e.g., Claude, GPT).
    Agent,
    /// Bot or automated system.
    Bot,
    /// System service account.
    System,
}

impl std::fmt::Display for MemberType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemberType::Human => write!(f, "human"),
            MemberType::Agent => write!(f, "agent"),
            MemberType::Bot => write!(f, "bot"),
            MemberType::System => write!(f, "system"),
        }
    }
}

impl std::str::FromStr for MemberType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "human" => Ok(MemberType::Human),
            "agent" => Ok(MemberType::Agent),
            "bot" => Ok(MemberType::Bot),
            "system" => Ok(MemberType::System),
            _ => Err(format!("Unknown member type: {}", s)),
        }
    }
}

impl Default for MemberType {
    fn default() -> Self {
        Self::Human
    }
}

/// Domain model for a workspace member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Member {
    /// Unique identifier.
    pub id: Uuid,
    /// Workspace this member belongs to.
    pub workspace_id: Uuid,
    /// External identity ID (from auth provider, e.g., nexis:human:alice@example.com).
    pub external_id: Option<String>,
    /// Type of member.
    pub member_type: MemberType,
    /// Display name for the member.
    pub display_name: Option<String>,
    /// Legacy email field (for backward compatibility).
    pub email: Option<String>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

impl Member {
    /// Create a new member in a workspace.
    pub fn new(workspace_id: Uuid, member_type: MemberType, display_name: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            workspace_id,
            external_id: None,
            member_type,
            display_name,
            email: None,
            created_at: Utc::now(),
        }
    }

    /// Create a member with an external ID.
    pub fn with_external_id(mut self, external_id: impl Into<String>) -> Self {
        self.external_id = Some(external_id.into());
        self
    }

    /// Create a member with an email (for backward compatibility).
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Check if this is a human member.
    pub fn is_human(&self) -> bool {
        self.member_type == MemberType::Human
    }

    /// Check if this is an AI agent.
    pub fn is_agent(&self) -> bool {
        self.member_type == MemberType::Agent
    }
}

/// Data required to create a new member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMember {
    /// Workspace ID.
    pub workspace_id: Uuid,
    /// External identity ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    /// Member type.
    #[serde(default)]
    pub member_type: MemberType,
    /// Display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Email (for backward compatibility).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// Data for updating a member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMember {
    /// New display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// New external ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
}

#[cfg(feature = "persistence-sqlx")]
pub mod repository {
    //! SQLx-based repository for member persistence with workspace scope.

    use super::*;
    use crate::db::{DatabasePool, RepositoryError};
    use async_trait::async_trait;
    use sqlx::Row;

    /// Persistence operations for workspace members.
    #[async_trait]
    pub trait WorkspaceMemberRepository: Send + Sync {
        /// Create and persist a member.
        async fn create(&self, member: CreateMember) -> Result<Member, RepositoryError>;

        /// Get a member by ID within workspace scope.
        async fn get(
            &self,
            workspace_id: Uuid,
            id: Uuid,
        ) -> Result<Option<Member>, RepositoryError>;

        /// Get a member by external ID within workspace scope.
        async fn get_by_external_id(
            &self,
            workspace_id: Uuid,
            external_id: &str,
        ) -> Result<Option<Member>, RepositoryError>;

        /// List all members in a workspace.
        async fn list(&self, workspace_id: Uuid) -> Result<Vec<Member>, RepositoryError>;

        /// Update a member.
        async fn update(
            &self,
            workspace_id: Uuid,
            id: Uuid,
            update: UpdateMember,
        ) -> Result<Option<Member>, RepositoryError>;

        /// Delete a member.
        async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<bool, RepositoryError>;
    }

    /// SQLx/PostgreSQL implementation of [`WorkspaceMemberRepository`].
    #[derive(Debug, Clone)]
    pub struct SqlxWorkspaceMemberRepository {
        pool: DatabasePool,
    }

    impl SqlxWorkspaceMemberRepository {
        /// Build a repository over an existing pool.
        pub fn new(pool: DatabasePool) -> Self {
            Self { pool }
        }
    }

    #[async_trait]
    impl WorkspaceMemberRepository for SqlxWorkspaceMemberRepository {
        async fn create(&self, member: CreateMember) -> Result<Member, RepositoryError> {
            let row = sqlx::query(
                r#"
                INSERT INTO members (workspace_id, external_id, "type", display_name, email)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id, workspace_id, external_id, "type", display_name, email, created_at
                "#,
            )
            .bind(member.workspace_id)
            .bind(&member.external_id)
            .bind(member.member_type.to_string())
            .bind(&member.display_name)
            .bind(&member.email)
            .fetch_one(&self.pool)
            .await?;

            Ok(Member {
                id: row.get("id"),
                workspace_id: row.get("workspace_id"),
                external_id: row.get("external_id"),
                member_type: row.get::<String, _>("type").parse().unwrap_or_default(),
                display_name: row.get("display_name"),
                email: row.get("email"),
                created_at: row.get("created_at"),
            })
        }

        async fn get(
            &self,
            workspace_id: Uuid,
            id: Uuid,
        ) -> Result<Option<Member>, RepositoryError> {
            let row = sqlx::query(
                r#"
                SELECT id, workspace_id, external_id, "type", display_name, email, created_at
                FROM members
                WHERE id = $1 AND workspace_id = $2
                "#,
            )
            .bind(id)
            .bind(workspace_id)
            .fetch_optional(&self.pool)
            .await?;

            Ok(row.map(|row| Member {
                id: row.get("id"),
                workspace_id: row.get("workspace_id"),
                external_id: row.get("external_id"),
                member_type: row.get::<String, _>("type").parse().unwrap_or_default(),
                display_name: row.get("display_name"),
                email: row.get("email"),
                created_at: row.get("created_at"),
            }))
        }

        async fn get_by_external_id(
            &self,
            workspace_id: Uuid,
            external_id: &str,
        ) -> Result<Option<Member>, RepositoryError> {
            let row = sqlx::query(
                r#"
                SELECT id, workspace_id, external_id, "type", display_name, email, created_at
                FROM members
                WHERE external_id = $1 AND workspace_id = $2
                "#,
            )
            .bind(external_id)
            .bind(workspace_id)
            .fetch_optional(&self.pool)
            .await?;

            Ok(row.map(|row| Member {
                id: row.get("id"),
                workspace_id: row.get("workspace_id"),
                external_id: row.get("external_id"),
                member_type: row.get::<String, _>("type").parse().unwrap_or_default(),
                display_name: row.get("display_name"),
                email: row.get("email"),
                created_at: row.get("created_at"),
            }))
        }

        async fn list(&self, workspace_id: Uuid) -> Result<Vec<Member>, RepositoryError> {
            let rows = sqlx::query(
                r#"
                SELECT id, workspace_id, external_id, "type", display_name, email, created_at
                FROM members
                WHERE workspace_id = $1
                ORDER BY created_at ASC
                "#,
            )
            .bind(workspace_id)
            .fetch_all(&self.pool)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| Member {
                    id: row.get("id"),
                    workspace_id: row.get("workspace_id"),
                    external_id: row.get("external_id"),
                    member_type: row.get::<String, _>("type").parse().unwrap_or_default(),
                    display_name: row.get("display_name"),
                    email: row.get("email"),
                    created_at: row.get("created_at"),
                })
                .collect())
        }

        async fn update(
            &self,
            workspace_id: Uuid,
            id: Uuid,
            update: UpdateMember,
        ) -> Result<Option<Member>, RepositoryError> {
            let row = sqlx::query(
                r#"
                UPDATE members
                SET display_name = COALESCE($1, display_name),
                    external_id = COALESCE($2, external_id)
                WHERE id = $3 AND workspace_id = $4
                RETURNING id, workspace_id, external_id, "type", display_name, email, created_at
                "#,
            )
            .bind(&update.display_name)
            .bind(&update.external_id)
            .bind(id)
            .bind(workspace_id)
            .fetch_optional(&self.pool)
            .await?;

            Ok(row.map(|row| Member {
                id: row.get("id"),
                workspace_id: row.get("workspace_id"),
                external_id: row.get("external_id"),
                member_type: row.get::<String, _>("type").parse().unwrap_or_default(),
                display_name: row.get("display_name"),
                email: row.get("email"),
                created_at: row.get("created_at"),
            }))
        }

        async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<bool, RepositoryError> {
            let result = sqlx::query(
                r#"
                DELETE FROM members
                WHERE id = $1 AND workspace_id = $2
                "#,
            )
            .bind(id)
            .bind(workspace_id)
            .execute(&self.pool)
            .await?;

            Ok(result.rows_affected() > 0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn member_new_creates_with_defaults() {
        let workspace_id = Uuid::new_v4();
        let member = Member::new(workspace_id, MemberType::Human, Some("Alice".to_string()));
        assert_eq!(member.workspace_id, workspace_id);
        assert_eq!(member.member_type, MemberType::Human);
        assert_eq!(member.display_name, Some("Alice".to_string()));
        assert!(member.external_id.is_none());
        assert!(member.id != Uuid::nil());
    }

    #[test]
    fn member_with_external_id() {
        let workspace_id = Uuid::new_v4();
        let member = Member::new(workspace_id, MemberType::Human, Some("Alice".to_string()))
            .with_external_id("nexis:human:alice@example.com");
        assert_eq!(
            member.external_id,
            Some("nexis:human:alice@example.com".to_string())
        );
    }

    #[test]
    fn member_type_checks() {
        let workspace_id = Uuid::new_v4();
        let human = Member::new(workspace_id, MemberType::Human, None);
        let agent = Member::new(workspace_id, MemberType::Agent, None);

        assert!(human.is_human());
        assert!(!human.is_agent());
        assert!(agent.is_agent());
        assert!(!agent.is_human());
    }

    #[test]
    fn member_type_parsing() {
        assert_eq!("human".parse::<MemberType>(), Ok(MemberType::Human));
        assert_eq!("AGENT".parse::<MemberType>(), Ok(MemberType::Agent));
        assert!("unknown".parse::<MemberType>().is_err());
    }
}
