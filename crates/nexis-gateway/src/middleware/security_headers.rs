//! Security Headers Middleware for Nexis Gateway
//!
//! Adds security-related HTTP headers to all responses.

use axum::{
    body::Body,
    http::{header, Request, Response},
    middleware::Next,
};
use std::time::Duration;

/// Security headers configuration
#[derive(Debug, Clone)]
pub struct SecurityHeadersConfig {
    /// Content-Security-Policy header value
    pub content_security_policy: Option<String>,
    /// X-Frame-Options header value
    pub frame_options: FrameOptions,
    /// X-Content-Type-Options header value
    pub content_type_options: ContentTypeOptions,
    /// Referrer-Policy header value
    pub referrer_policy: ReferrerPolicy,
    /// Permissions-Policy header value
    pub permissions_policy: Option<String>,
    /// HSTS max-age in seconds (None to disable)
    pub hsts_max_age: Option<Duration>,
    /// Whether to include subdomains in HSTS
    pub hsts_include_subdomains: bool,
    /// Whether to preload HSTS
    pub hsts_preload: bool,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self {
            content_security_policy: Some(
                "default-src 'self'; \
                 script-src 'self' 'unsafe-inline'; \
                 style-src 'self' 'unsafe-inline'; \
                 img-src 'self' data: https:; \
                 font-src 'self'; \
                 connect-src 'self' wss: https:; \
                 frame-ancestors 'none';"
                    .to_string(),
            ),
            frame_options: FrameOptions::Deny,
            content_type_options: ContentTypeOptions::NoSniff,
            referrer_policy: ReferrerPolicy::StrictOriginWhenCrossOrigin,
            permissions_policy: Some(
                "geolocation=(), \
                 microphone=(), \
                 camera=(), \
                 payment=(), \
                 usb=()"
                    .to_string(),
            ),
            hsts_max_age: Some(Duration::from_secs(31536000)), // 1 year
            hsts_include_subdomains: true,
            hsts_preload: false,
        }
    }
}

/// X-Frame-Options header values
#[derive(Debug, Clone, Copy)]
pub enum FrameOptions {
    Deny,
    SameOrigin,
    AllowFrom(String),
}

impl std::fmt::Display for FrameOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deny => write!(f, "DENY"),
            Self::SameOrigin => write!(f, "SAMEORIGIN"),
            Self::AllowFrom(origin) => write!(f, "ALLOW-FROM {}", origin),
        }
    }
}

/// X-Content-Type-Options header values
#[derive(Debug, Clone, Copy)]
pub enum ContentTypeOptions {
    NoSniff,
}

impl std::fmt::Display for ContentTypeOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSniff => write!(f, "nosniff"),
        }
    }
}

/// Referrer-Policy header values
#[derive(Debug, Clone, Copy)]
pub enum ReferrerPolicy {
    NoReferrer,
    NoReferrerWhenDowngrade,
    Origin,
    OriginWhenCrossOrigin,
    SameOrigin,
    StrictOrigin,
    StrictOriginWhenCrossOrigin,
    UnsafeUrl,
}

impl std::fmt::Display for ReferrerPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoReferrer => write!(f, "no-referrer"),
            Self::NoReferrerWhenDowngrade => write!(f, "no-referrer-when-downgrade"),
            Self::Origin => write!(f, "origin"),
            Self::OriginWhenCrossOrigin => write!(f, "origin-when-cross-origin"),
            Self::SameOrigin => write!(f, "same-origin"),
            Self::StrictOrigin => write!(f, "strict-origin"),
            Self::StrictOriginWhenCrossOrigin => write!(f, "strict-origin-when-cross-origin"),
            Self::UnsafeUrl => write!(f, "unsafe-url"),
        }
    }
}

/// Security headers middleware
pub async fn security_headers_middleware(request: Request<Body>, next: Next) -> Response<Body> {
    let config = SecurityHeadersConfig::default();
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // X-Content-Type-Options
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        config.content_type_options.to_string().parse().unwrap(),
    );

    // X-Frame-Options
    headers.insert(
        header::X_FRAME_OPTIONS,
        config.frame_options.to_string().parse().unwrap(),
    );

    // Referrer-Policy
    headers.insert(
        header::REFERRER_POLICY,
        config.referrer_policy.to_string().parse().unwrap(),
    );

    // Content-Security-Policy
    if let Some(csp) = &config.content_security_policy {
        headers.insert(header::CONTENT_SECURITY_POLICY, csp.parse().unwrap());
    }

    // Permissions-Policy
    if let Some(pp) = &config.permissions_policy {
        headers.insert("Permissions-Policy", pp.parse().unwrap());
    }

    // Strict-Transport-Security (HSTS)
    if let Some(max_age) = config.hsts_max_age {
        let mut hsts_value = format!("max-age={}", max_age.as_secs());
        if config.hsts_include_subdomains {
            hsts_value.push_str("; includeSubDomains");
        }
        if config.hsts_preload {
            hsts_value.push_str("; preload");
        }
        headers.insert(
            header::STRICT_TRANSPORT_SECURITY,
            hsts_value.parse().unwrap(),
        );
    }

    // X-XSS-Protection (deprecated but still useful for older browsers)
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());

    // Cache-Control for API responses
    headers.insert(header::CACHE_CONTROL, "no-store".parse().unwrap());

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn adds_security_headers() {
        let app = Router::new()
            .route("/test", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(security_headers_middleware));

        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Check headers are present
        assert!(response
            .headers()
            .contains_key(header::X_CONTENT_TYPE_OPTIONS));
        assert!(response.headers().contains_key(header::X_FRAME_OPTIONS));
        assert!(response.headers().contains_key(header::REFERRER_POLICY));
        assert!(response
            .headers()
            .contains_key(header::CONTENT_SECURITY_POLICY));
        assert!(response.headers().contains_key("Permissions-Policy"));
        assert!(response
            .headers()
            .contains_key(header::STRICT_TRANSPORT_SECURITY));
    }

    #[test]
    fn frame_options_display() {
        assert_eq!(FrameOptions::Deny.to_string(), "DENY");
        assert_eq!(FrameOptions::SameOrigin.to_string(), "SAMEORIGIN");
    }

    #[test]
    fn referrer_policy_display() {
        assert_eq!(
            ReferrerPolicy::StrictOriginWhenCrossOrigin.to_string(),
            "strict-origin-when-cross-origin"
        );
    }
}
