//! Middleware components for Nexis Gateway.
//!
//! This module provides middleware for:
//! - `rate_limit`: Token-bucket rate limiting (429 + Retry-After)
//! - `request_id`: Unique request ID generation and propagation
//! - `security_headers`: Security-related HTTP headers
//! - `logging`: Request logging
//! - `tenant`: Multi-tenant context resolution

pub mod logging;
pub mod rate_limit;
pub mod request_id;
pub mod security_headers;

pub use logging::{request_logging, RequestLog, RequestLoggingLayer};

pub use rate_limit::{
    rate_limit_middleware, register_metrics, RateLimitConfig, RateLimitResponse, RateLimiter,
    TokenBucket,
};

pub use request_id::{request_id_middleware, RequestId, X_REQUEST_ID};

pub use security_headers::{
    security_headers_middleware, ContentTypeOptions, FrameOptions, ReferrerPolicy,
    SecurityHeadersConfig,
};

#[cfg(feature = "multi-tenant")]
pub mod tenant;

#[cfg(feature = "multi-tenant")]
pub use tenant::{
    InMemoryTenantStore, MiddlewareTenantContext, ResolutionStrategy, ResolvedTenant, TenantLookup,
    TenantResolutionConfig, TenantResolutionError, TenantResolver, TenantSource,
};
