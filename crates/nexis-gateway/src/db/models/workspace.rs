//! Workspace model for organizing tenant resources.
//!
//! A workspace belongs to a tenant and contains members and rooms.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Domain model for a workspace within a tenant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    /// Unique identifier (UUID).
    pub id: Uuid,
    /// Parent tenant ID.
    pub tenant_id: Uuid,
    /// Display name of the workspace.
    pub name: String,
    /// URL-friendly identifier (unique within tenant).
    pub slug: String,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

impl Workspace {
    /// Create a new workspace within a tenant.
    pub fn new(tenant_id: Uuid, name: impl Into<String>, slug: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            name: name.into(),
            slug: slug.into(),
            created_at: Utc::now(),
        }
    }
}

/// Data required to create a new workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkspace {
    /// Parent tenant ID.
    pub tenant_id: Uuid,
    /// Display name.
    pub name: String,
    /// URL-friendly slug.
    pub slug: String,
}

/// Data for updating a workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkspace {
    /// New display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New slug (within tenant scope).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
}

#[cfg(feature = "persistence-sqlx")]
pub mod repository {
    //! SQLx-based repository for workspace persistence.

    use super::*;
    use crate::db::{DatabasePool, RepositoryError};
    use async_trait::async_trait;
    use sqlx::Row;

    /// Persistence operations for workspaces.
    #[async_trait]
    pub trait WorkspaceRepository: Send + Sync {
        /// Create and persist a workspace.
        async fn create(
            &self,
            tenant_id: Uuid,
            name: &str,
            slug: &str,
        ) -> Result<Workspace, RepositoryError>;

        /// Get a workspace by ID within tenant scope.
        async fn get(
            &self,
            tenant_id: Uuid,
            id: Uuid,
        ) -> Result<Option<Workspace>, RepositoryError>;

        /// Get a workspace by slug within tenant scope.
        async fn get_by_slug(
            &self,
            tenant_id: Uuid,
            slug: &str,
        ) -> Result<Option<Workspace>, RepositoryError>;

        /// List all workspaces for a tenant.
        async fn list(&self, tenant_id: Uuid) -> Result<Vec<Workspace>, RepositoryError>;

        /// Delete a workspace.
        async fn delete(&self, tenant_id: Uuid, id: Uuid) -> Result<bool, RepositoryError>;
    }

    /// SQLx/PostgreSQL implementation of [`WorkspaceRepository`].
    #[derive(Debug, Clone)]
    pub struct SqlxWorkspaceRepository {
        pool: DatabasePool,
    }

    impl SqlxWorkspaceRepository {
        /// Build a repository over an existing pool.
        pub fn new(pool: DatabasePool) -> Self {
            Self { pool }
        }
    }

    #[async_trait]
    impl WorkspaceRepository for SqlxWorkspaceRepository {
        async fn create(
            &self,
            tenant_id: Uuid,
            name: &str,
            slug: &str,
        ) -> Result<Workspace, RepositoryError> {
            let row = sqlx::query(
                r#"
                INSERT INTO workspaces (tenant_id, name, slug)
                VALUES ($1, $2, $3)
                RETURNING id, tenant_id, name, slug, created_at
                "#,
            )
            .bind(tenant_id)
            .bind(name)
            .bind(slug)
            .fetch_one(&self.pool)
            .await?;

            Ok(Workspace {
                id: row.get("id"),
                tenant_id: row.get("tenant_id"),
                name: row.get("name"),
                slug: row.get("slug"),
                created_at: row.get("created_at"),
            })
        }

        async fn get(
            &self,
            tenant_id: Uuid,
            id: Uuid,
        ) -> Result<Option<Workspace>, RepositoryError> {
            let row = sqlx::query(
                r#"
                SELECT id, tenant_id, name, slug, created_at
                FROM workspaces
                WHERE id = $1 AND tenant_id = $2
                "#,
            )
            .bind(id)
            .bind(tenant_id)
            .fetch_optional(&self.pool)
            .await?;

            Ok(row.map(|row| Workspace {
                id: row.get("id"),
                tenant_id: row.get("tenant_id"),
                name: row.get("name"),
                slug: row.get("slug"),
                created_at: row.get("created_at"),
            }))
        }

        async fn get_by_slug(
            &self,
            tenant_id: Uuid,
            slug: &str,
        ) -> Result<Option<Workspace>, RepositoryError> {
            let row = sqlx::query(
                r#"
                SELECT id, tenant_id, name, slug, created_at
                FROM workspaces
                WHERE slug = $1 AND tenant_id = $2
                "#,
            )
            .bind(slug)
            .bind(tenant_id)
            .fetch_optional(&self.pool)
            .await?;

            Ok(row.map(|row| Workspace {
                id: row.get("id"),
                tenant_id: row.get("tenant_id"),
                name: row.get("name"),
                slug: row.get("slug"),
                created_at: row.get("created_at"),
            }))
        }

        async fn list(&self, tenant_id: Uuid) -> Result<Vec<Workspace>, RepositoryError> {
            let rows = sqlx::query(
                r#"
                SELECT id, tenant_id, name, slug, created_at
                FROM workspaces
                WHERE tenant_id = $1
                ORDER BY created_at ASC
                "#,
            )
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?;

            Ok(rows
                .into_iter()
                .map(|row| Workspace {
                    id: row.get("id"),
                    tenant_id: row.get("tenant_id"),
                    name: row.get("name"),
                    slug: row.get("slug"),
                    created_at: row.get("created_at"),
                })
                .collect())
        }

        async fn delete(&self, tenant_id: Uuid, id: Uuid) -> Result<bool, RepositoryError> {
            let result = sqlx::query(
                r#"
                DELETE FROM workspaces
                WHERE id = $1 AND tenant_id = $2
                "#,
            )
            .bind(id)
            .bind(tenant_id)
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
    fn workspace_new_creates_with_defaults() {
        let tenant_id = Uuid::new_v4();
        let workspace = Workspace::new(tenant_id, "Engineering", "engineering");
        assert_eq!(workspace.tenant_id, tenant_id);
        assert_eq!(workspace.name, "Engineering");
        assert_eq!(workspace.slug, "engineering");
        assert!(workspace.id != Uuid::nil());
    }
}
