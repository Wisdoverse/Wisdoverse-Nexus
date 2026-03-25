//! Tenant resolution middleware for multi-tenant request handling.
//!
//! This middleware extracts tenant context from various sources:
//! - Subdomain (e.g., acme.nexis.ai)
//! - HTTP header (X-Tenant-ID or X-Tenant-Slug)
//! - URL path prefix (e.g., /t/acme/...)

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

/// Error types for tenant resolution.
#[derive(Debug, thiserror::Error)]
pub enum TenantResolutionError {
    /// No tenant context found in any source.
    #[error("Missing tenant context")]
    MissingTenant,
    /// Invalid tenant identifier format.
    #[error("Invalid tenant identifier: {0}")]
    InvalidIdentifier(String),
    /// Tenant not found in the system.
    #[error("Tenant not found: {0}")]
    TenantNotFound(String),
    /// Header format is invalid.
    #[error("Invalid tenant header format")]
    InvalidHeaderFormat,
}

impl IntoResponse for TenantResolutionError {
    fn into_response(self) -> Response {
        use axum::http::StatusCode;
        let (status, message) = match &self {
            TenantResolutionError::MissingTenant => {
                (StatusCode::BAD_REQUEST, "Missing tenant context")
            }
            TenantResolutionError::InvalidIdentifier(id) => (
                StatusCode::BAD_REQUEST,
                &format!("Invalid tenant identifier: {}", id),
            ),
            TenantResolutionError::TenantNotFound(id) => {
                (StatusCode::NOT_FOUND, &format!("Tenant not found: {}", id))
            }
            TenantResolutionError::InvalidHeaderFormat => {
                (StatusCode::BAD_REQUEST, "Invalid tenant header format")
            }
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

/// Resolved tenant context containing both ID and optional slug.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedTenant {
    /// The tenant's UUID.
    pub id: Uuid,
    /// The tenant's URL-friendly slug (if available).
    pub slug: Option<String>,
}

impl ResolvedTenant {
    /// Create a new resolved tenant with just an ID.
    pub fn from_id(id: Uuid) -> Self {
        Self { id, slug: None }
    }

    /// Create a resolved tenant with both ID and slug.
    pub fn with_slug(id: Uuid, slug: impl Into<String>) -> Self {
        Self {
            id,
            slug: Some(slug.into()),
        }
    }
}

/// Source from which the tenant was resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TenantSource {
    /// Extracted from subdomain (e.g., acme.nexis.ai).
    Subdomain,
    /// Extracted from HTTP header.
    Header,
    /// Extracted from URL path.
    Path,
    /// Extracted from query parameter.
    Query,
    /// Provided explicitly (e.g., from JWT claims).
    Explicit,
}

/// Strategy for resolving tenant from a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionStrategy {
    /// Try subdomain first, then header, then path.
    SubdomainFirst,
    /// Try header first, then subdomain, then path.
    HeaderFirst,
    /// Only use explicit tenant ID (from auth).
    ExplicitOnly,
}

impl Default for ResolutionStrategy {
    fn default() -> Self {
        Self::SubdomainFirst
    }
}

/// Configuration for tenant resolution.
#[derive(Debug, Clone)]
pub struct TenantResolutionConfig {
    /// Resolution strategy to use.
    pub strategy: ResolutionStrategy,
    /// Header name for tenant ID (default: "x-tenant-id").
    pub tenant_id_header: String,
    /// Header name for tenant slug (default: "x-tenant-slug").
    pub tenant_slug_header: String,
    /// Query parameter name for tenant (default: "tenant").
    pub tenant_query_param: String,
    /// Base domain for subdomain extraction (e.g., "nexis.ai").
    pub base_domain: Option<String>,
}

impl Default for TenantResolutionConfig {
    fn default() -> Self {
        Self {
            strategy: ResolutionStrategy::default(),
            tenant_id_header: "x-tenant-id".to_string(),
            tenant_slug_header: "x-tenant-slug".to_string(),
            tenant_query_param: "tenant".to_string(),
            base_domain: None,
        }
    }
}

/// Tenant resolver that extracts tenant context from requests.
#[derive(Debug, Clone)]
pub struct TenantResolver {
    config: TenantResolutionConfig,
    /// Optional tenant lookup function (for validating slugs/IDs).
    #[allow(dead_code)]
    lookup: Option<Arc<dyn TenantLookup + Send + Sync>>,
}

impl TenantResolver {
    /// Create a new tenant resolver with default configuration.
    pub fn new() -> Self {
        Self {
            config: TenantResolutionConfig::default(),
            lookup: None,
        }
    }

    /// Create a resolver with custom configuration.
    pub fn with_config(config: TenantResolutionConfig) -> Self {
        Self {
            config,
            lookup: None,
        }
    }

    /// Add a tenant lookup function.
    pub fn with_lookup(mut self, lookup: Arc<dyn TenantLookup + Send + Sync>) -> Self {
        self.lookup = Some(lookup);
        self
    }

    /// Resolve tenant from request parts.
    pub async fn resolve(
        &self,
        parts: &mut Parts,
    ) -> Result<ResolvedTenant, TenantResolutionError> {
        match self.config.strategy {
            ResolutionStrategy::SubdomainFirst => self
                .from_subdomain(parts)
                .or_else(|_| self.from_header(parts))
                .or_else(|_| self.from_path(parts))
                .or_else(|_| self.from_query(parts)),
            ResolutionStrategy::HeaderFirst => self
                .from_header(parts)
                .or_else(|_| self.from_subdomain(parts))
                .or_else(|_| self.from_path(parts))
                .or_else(|_| self.from_query(parts)),
            ResolutionStrategy::ExplicitOnly => Err(TenantResolutionError::MissingTenant),
        }
    }

    /// Extract tenant from subdomain.
    fn from_subdomain(&self, parts: &Parts) -> Result<ResolvedTenant, TenantResolutionError> {
        let host = parts
            .headers
            .get("host")
            .and_then(|h| h.to_str().ok())
            .ok_or(TenantResolutionError::MissingTenant)?;

        // Remove port if present
        let host = host.split(':').next().unwrap_or(host);

        // Extract subdomain
        if let Some(base_domain) = &self.config.base_domain {
            if host.ends_with(base_domain) {
                let subdomain = host
                    .strip_suffix(base_domain)
                    .and_then(|s| s.strip_suffix('.'))
                    .filter(|s| !s.is_empty());

                if let Some(slug) = subdomain {
                    // Would need lookup to get ID; for now return slug
                    return Ok(ResolvedTenant {
                        id: Uuid::nil(), // Placeholder; actual ID from lookup
                        slug: Some(slug.to_string()),
                    });
                }
            }
        } else {
            // Simple extraction: first part before first dot
            if let Some(slug) = host.split('.').next() {
                if !slug.is_empty() {
                    return Ok(ResolvedTenant {
                        id: Uuid::nil(),
                        slug: Some(slug.to_string()),
                    });
                }
            }
        }

        Err(TenantResolutionError::MissingTenant)
    }

    /// Extract tenant from headers.
    fn from_header(&self, parts: &Parts) -> Result<ResolvedTenant, TenantResolutionError> {
        // Try tenant ID header first
        if let Some(id_str) = parts
            .headers
            .get(&self.config.tenant_id_header)
            .and_then(|h| h.to_str().ok())
            .filter(|s| !s.is_empty())
        {
            let id = Uuid::parse_str(id_str)
                .map_err(|_| TenantResolutionError::InvalidIdentifier(id_str.to_string()))?;
            return Ok(ResolvedTenant::from_id(id));
        }

        // Try tenant slug header
        if let Some(slug) = parts
            .headers
            .get(&self.config.tenant_slug_header)
            .and_then(|h| h.to_str().ok())
            .filter(|s| !s.is_empty())
        {
            return Ok(ResolvedTenant {
                id: Uuid::nil(), // Would need lookup
                slug: Some(slug.to_string()),
            });
        }

        Err(TenantResolutionError::MissingTenant)
    }

    /// Extract tenant from URL path.
    fn from_path(&self, _parts: &Parts) -> Result<ResolvedTenant, TenantResolutionError> {
        // This would need path extraction; typically done via Path extractor
        // For now, return Missing to allow fallback
        Err(TenantResolutionError::MissingTenant)
    }

    /// Extract tenant from query parameters.
    fn from_query(&self, parts: &Parts) -> Result<ResolvedTenant, TenantResolutionError> {
        let uri = &parts.uri;
        let query = uri.query().unwrap_or("");

        // Parse query parameters manually
        for pair in query.split('&') {
            if let Some(value) = pair.strip_prefix(&format!("{}=", self.config.tenant_query_param))
            {
                let value = urlencoding_decode(value);

                // Try to parse as UUID first
                if let Ok(id) = Uuid::parse_str(&value) {
                    return Ok(ResolvedTenant::from_id(id));
                }

                // Otherwise treat as slug
                return Ok(ResolvedTenant {
                    id: Uuid::nil(),
                    slug: Some(value),
                });
            }
        }

        Err(TenantResolutionError::MissingTenant)
    }
}

impl Default for TenantResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple URL encoding decoder.
fn urlencoding_decode(s: &str) -> String {
    s.replace("%20", " ")
        .replace("%2F", "/")
        .replace("%3A", ":")
        .replace("%40", "@")
}

/// Trait for looking up tenant information.
#[async_trait::async_trait]
pub trait TenantLookup {
    /// Look up a tenant by ID.
    async fn lookup_by_id(&self, id: Uuid) -> Option<ResolvedTenant>;
    /// Look up a tenant by slug.
    async fn lookup_by_slug(&self, slug: &str) -> Option<ResolvedTenant>;
}

/// Extractor that provides resolved tenant context.
#[derive(Debug, Clone)]
pub struct MiddlewareMiddlewareMiddlewareTenantContext {
    pub tenant: ResolvedTenant,
    pub source: TenantSource,
}

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for TenantContext
where
    S: Send + Sync,
    TenantResolver: FromRef<S>,
{
    type Rejection = TenantResolutionError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let resolver = TenantResolver::from_ref(state);
        let tenant = resolver.resolve(parts).await?;

        // Determine source based on what succeeded
        // This is simplified; in practice you'd track the source
        let source = TenantSource::Header; // Default assumption

        Ok(MiddlewareMiddlewareTenantContext { tenant, source })
    }
}

/// In-memory tenant store for testing.
#[derive(Debug, Clone, Default)]
pub struct InMemoryTenantStore {
    tenants: Arc<std::sync::RwLock<Vec<(Uuid, String)>>>,
}

impl InMemoryTenantStore {
    /// Create a new empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tenant.
    pub fn register(&self, id: Uuid, slug: String) {
        let mut tenants = self.tenants.write().unwrap();
        if !tenants.iter().any(|(i, _)| i == &id) {
            tenants.push((id, slug));
        }
    }

    /// Look up a tenant by ID.
    pub fn get_by_id(&self, id: Uuid) -> Option<String> {
        let tenants = self.tenants.read().unwrap();
        tenants
            .iter()
            .find(|(i, _)| i == &id)
            .map(|(_, s)| s.clone())
    }

    /// Look up a tenant by slug.
    pub fn get_by_slug(&self, slug: &str) -> Option<Uuid> {
        let tenants = self.tenants.read().unwrap();
        tenants.iter().find(|(_, s)| s == slug).map(|(i, _)| *i)
    }
}

#[async_trait::async_trait]
impl TenantLookup for InMemoryTenantStore {
    async fn lookup_by_id(&self, id: Uuid) -> Option<ResolvedTenant> {
        self.get_by_id(id)
            .map(|slug| ResolvedTenant::with_slug(id, slug))
    }

    async fn lookup_by_slug(&self, slug: &str) -> Option<ResolvedTenant> {
        self.get_by_slug(slug)
            .map(|id| ResolvedTenant::with_slug(id, slug))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_tenant_from_id() {
        let id = Uuid::new_v4();
        let tenant = ResolvedTenant::from_id(id);
        assert_eq!(tenant.id, id);
        assert!(tenant.slug.is_none());
    }

    #[test]
    fn resolved_tenant_with_slug() {
        let id = Uuid::new_v4();
        let tenant = ResolvedTenant::with_slug(id, "acme-corp");
        assert_eq!(tenant.id, id);
        assert_eq!(tenant.slug, Some("acme-corp".to_string()));
    }

    #[test]
    fn tenant_resolver_default_config() {
        let resolver = TenantResolver::new();
        assert_eq!(resolver.config.strategy, ResolutionStrategy::SubdomainFirst);
        assert_eq!(resolver.config.tenant_id_header, "x-tenant-id");
    }

    #[test]
    fn in_memory_tenant_store() {
        let store = InMemoryTenantStore::new();
        let id = Uuid::new_v4();
        store.register(id, "acme-corp".to_string());

        assert_eq!(store.get_by_id(id), Some("acme-corp".to_string()));
        assert_eq!(store.get_by_slug("acme-corp"), Some(id));
        assert_eq!(store.get_by_slug("unknown"), None);
    }

    #[test]
    fn urlencoding_decode_basic() {
        assert_eq!(urlencoding_decode("hello%20world"), "hello world");
        assert_eq!(urlencoding_decode("user%40example.com"), "user@example.com");
    }
}
