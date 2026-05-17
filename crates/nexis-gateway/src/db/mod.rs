//! Database connection and schema bootstrap.
//!
//! This module owns database infrastructure shared by gateway adapters.
//! Domain-facing repository ports live in their bounded-context modules.

#[cfg(feature = "multi-tenant")]
pub mod models;

#[cfg(feature = "persistence-sqlx")]
use sqlx::{postgres::PgPoolOptions, PgPool};
use thiserror::Error;

/// Database connection pool type used by gateway persistence.
#[cfg(feature = "persistence-sqlx")]
pub type DatabasePool = PgPool;

/// Placeholder pool type when SQLx persistence is disabled.
#[cfg(not(feature = "persistence-sqlx"))]
#[derive(Debug, Clone, Copy, Default)]
pub struct DatabasePool;

/// SQL schema for the `rooms` table.
pub const ROOMS_TABLE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS rooms (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    creator_id TEXT,
    topic TEXT,
    tenant_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);"#;

/// SQL schema for the `messages` table.
pub const MESSAGES_TABLE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    room_id TEXT NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    sender_id TEXT NOT NULL,
    content TEXT NOT NULL,
    reply_to TEXT,
    tenant_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);"#;

/// SQL schema for the `members` table.
pub const MEMBERS_TABLE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS members (
    id TEXT PRIMARY KEY,
    "type" TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);"#;

/// SQL schema for the `room_members` table.
pub const ROOM_MEMBERS_TABLE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS room_members (
    room_id TEXT NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    member_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (room_id, member_id)
);"#;

/// Index for room listing by creation time.
pub const ROOMS_CREATED_AT_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_rooms_created_at ON rooms(created_at);"#;

/// Index for rooms by creator.
pub const ROOMS_CREATOR_ID_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_rooms_creator_id ON rooms(creator_id);"#;

/// Composite index for tenant-scoped room lookups.
pub const ROOMS_TENANT_ID_ID_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_rooms_tenant_id_id ON rooms(tenant_id, id);"#;

/// Composite index for room message lookups ordered by creation time.
pub const MESSAGES_ROOM_CREATED_AT_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_messages_room_created_at ON messages(room_id, created_at);"#;

/// Index for messages by sender.
pub const MESSAGES_SENDER_ID_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_messages_sender_id ON messages(sender_id);"#;

/// Composite index for tenant-scoped message lookups.
pub const MESSAGES_TENANT_ID_ID_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_messages_tenant_id_id ON messages(tenant_id, id);"#;

/// Composite index for tenant-scoped room message lookups.
pub const MESSAGES_TENANT_ID_ROOM_ID_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_messages_tenant_id_room_id ON messages(tenant_id, room_id);"#;

/// Index for room membership lookups by member.
pub const ROOM_MEMBERS_MEMBER_ID_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_room_members_member_id ON room_members(member_id);"#;

/// Unique lookup index for members by email.
pub const MEMBERS_EMAIL_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_members_email ON members(email);"#;

/// Error type returned by database infrastructure operations.
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// Database query failed.
    #[cfg(feature = "persistence-sqlx")]
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    /// SQLx persistence feature is disabled.
    #[error("persistence-sqlx feature is disabled")]
    SqlxDisabled,
}

/// Create a PostgreSQL connection pool for gateway persistence.
#[cfg(feature = "persistence-sqlx")]
pub async fn init_pool(database_url: &str) -> Result<DatabasePool, RepositoryError> {
    Ok(PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?)
}

/// Create a PostgreSQL connection pool for gateway persistence.
#[cfg(not(feature = "persistence-sqlx"))]
pub async fn init_pool(_database_url: &str) -> Result<DatabasePool, RepositoryError> {
    Err(RepositoryError::SqlxDisabled)
}

/// Initialize required tables if they do not exist.
#[cfg(feature = "persistence-sqlx")]
pub async fn initialize_schema(pool: &DatabasePool) -> Result<(), RepositoryError> {
    sqlx::query(ROOMS_TABLE_SCHEMA).execute(pool).await?;
    sqlx::query(MESSAGES_TABLE_SCHEMA).execute(pool).await?;
    sqlx::query(MEMBERS_TABLE_SCHEMA).execute(pool).await?;
    sqlx::query(ROOM_MEMBERS_TABLE_SCHEMA).execute(pool).await?;
    sqlx::query(ROOMS_CREATED_AT_INDEX).execute(pool).await?;
    sqlx::query(ROOMS_CREATOR_ID_INDEX).execute(pool).await?;
    sqlx::query(ROOMS_TENANT_ID_ID_INDEX).execute(pool).await?;
    sqlx::query(MESSAGES_ROOM_CREATED_AT_INDEX)
        .execute(pool)
        .await?;
    sqlx::query(MESSAGES_SENDER_ID_INDEX).execute(pool).await?;
    sqlx::query(MESSAGES_TENANT_ID_ID_INDEX)
        .execute(pool)
        .await?;
    sqlx::query(MESSAGES_TENANT_ID_ROOM_ID_INDEX)
        .execute(pool)
        .await?;
    sqlx::query(ROOM_MEMBERS_MEMBER_ID_INDEX)
        .execute(pool)
        .await?;
    sqlx::query(MEMBERS_EMAIL_INDEX).execute(pool).await?;
    Ok(())
}

/// Initialize required tables if they do not exist.
#[cfg(not(feature = "persistence-sqlx"))]
pub async fn initialize_schema(_pool: &DatabasePool) -> Result<(), RepositoryError> {
    Err(RepositoryError::SqlxDisabled)
}
