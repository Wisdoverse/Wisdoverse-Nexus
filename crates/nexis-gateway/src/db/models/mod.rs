//! Database models for multi-tenant data structures.
//!
//! This module provides domain models and repositories for:
//! - `Tenant`: Organization/account level isolation
//! - `Workspace`: Subdivision within a tenant
//! - `Member`: Users and agents within a workspace

pub mod member;
pub mod tenant;
pub mod workspace;

pub use member::{CreateMember, Member, MemberType, UpdateMember};
pub use tenant::{CreateTenant, Plan, Tenant, TenantLimits, UpdateTenant};
pub use workspace::{CreateWorkspace, UpdateWorkspace, Workspace};

#[cfg(feature = "persistence-sqlx")]
pub use member::repository::{SqlxWorkspaceMemberRepository, WorkspaceMemberRepository};
#[cfg(feature = "persistence-sqlx")]
pub use workspace::repository::{SqlxWorkspaceRepository, WorkspaceRepository};
