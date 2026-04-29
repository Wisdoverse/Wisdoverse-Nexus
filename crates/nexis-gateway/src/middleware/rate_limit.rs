//! Token-bucket rate limiting middleware.
//!
//! Protects API endpoints from abuse by enforcing per-IP request limits.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::SystemTime;

use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use dashmap::DashMap;
use lazy_static::lazy_static;
use prometheus::{IntCounterVec, Opts, Registry};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Prometheus metrics
// ---------------------------------------------------------------------------

lazy_static! {
    static ref RATE_LIMIT_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new(
            "nexis_rate_limit_total",
            "Total number of rate limit checks performed"
        ),
        &["ip"]
    )
    .expect("failed to create nexis_rate_limit_total");
    static ref RATE_LIMIT_REJECTED_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new(
            "nexis_rate_limit_rejected_total",
            "Total number of requests rejected by rate limiter"
        ),
        &["ip"]
    )
    .expect("failed to create nexis_rate_limit_rejected_total");
}

/// Register rate-limit metrics with a Prometheus [`Registry`].
///
/// Call this once during application startup. Calling it more than once with
/// the same registry will panic.
pub fn register_metrics(registry: &Registry) {
    registry
        .register(Box::new(RATE_LIMIT_TOTAL.clone()))
        .expect("rate_limit_total already registered");
    registry
        .register(Box::new(RATE_LIMIT_REJECTED_TOTAL.clone()))
        .expect("rate_limit_rejected_total already registered");
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Rate-limit configuration loaded from environment variables.
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed within `window_secs`.
    pub max_requests: u32,
    /// Time window in seconds for the rate limit.
    pub window_secs: u64,
    /// Maximum burst size (initial token capacity).
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl RateLimitConfig {
    /// Load configuration from environment variables with sensible defaults.
    ///
    /// | Variable                | Default | Description                    |
    /// |-------------------------|---------|--------------------------------|
    /// | `NEXIS_RATE_LIMIT_RPM`  | `100`   | Requests per minute (per IP)   |
    pub fn from_env() -> Self {
        let max_requests: u32 = std::env::var("NEXIS_RATE_LIMIT_RPM")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);

        let window_secs: u64 = 60;

        // Burst defaults to max_requests so the bucket starts full.
        let burst_size = max_requests;

        Self {
            max_requests,
            window_secs,
            burst_size,
        }
    }
}

// ---------------------------------------------------------------------------
// Token Bucket
// ---------------------------------------------------------------------------

/// A single token bucket for one client key.
pub struct TokenBucket {
    /// Current number of available tokens.
    tokens: AtomicU32,
    /// Timestamp (seconds since UNIX_EPOCH) of the last refill.
    last_refill: AtomicU64,
}

impl TokenBucket {
    /// Create a new bucket pre-filled with `initial_tokens`.
    fn new(initial_tokens: u32) -> Self {
        let now_secs = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock went backwards")
            .as_secs();

        Self {
            tokens: AtomicU32::new(initial_tokens),
            last_refill: AtomicU64::new(now_secs),
        }
    }

    /// Try to consume one token. Returns `true` if allowed.
    ///
    /// Tokens are refilled at a rate of `max_requests / window_secs` per second,
    /// capped at `burst_size`.
    fn try_acquire(&self, config: &RateLimitConfig) -> bool {
        let now_secs = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock went backwards")
            .as_secs();

        let last = self.last_refill.load(Ordering::Relaxed);
        let elapsed = now_secs.saturating_sub(last);

        let refill_rate = config.max_requests as f64 / config.window_secs as f64;
        let tokens_to_add = (elapsed as f64 * refill_rate).floor() as u32;

        if tokens_to_add > 0 {
            // Atomically add tokens, capped at burst_size.
            let mut current = self.tokens.load(Ordering::Relaxed);
            loop {
                let new_tokens = (current + tokens_to_add).min(config.burst_size);
                match self.tokens.compare_exchange_weak(
                    current,
                    new_tokens,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(actual) => current = actual,
                }
            }
            self.last_refill.store(now_secs, Ordering::Relaxed);
        }

        // Try to consume one token.
        let mut current = self.tokens.load(Ordering::Relaxed);
        loop {
            if current == 0 {
                return false;
            }
            match self.tokens.compare_exchange_weak(
                current,
                current - 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }

    /// Approximate number of seconds until the next token is available.
    fn retry_after_secs(&self, config: &RateLimitConfig) -> u64 {
        let now_secs = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock went backwards")
            .as_secs();

        let last = self.last_refill.load(Ordering::Relaxed);
        let elapsed = now_secs.saturating_sub(last);

        let refill_rate = config.max_requests as f64 / config.window_secs as f64;
        let tokens_added = (elapsed as f64 * refill_rate).floor() as u32;

        if tokens_added > 0 {
            return 1;
        }

        let secs_per_token = if refill_rate > 0.0 {
            (1.0 / refill_rate).ceil() as u64
        } else {
            config.window_secs
        };

        secs_per_token.saturating_sub(elapsed).max(1)
    }
}

// ---------------------------------------------------------------------------
// Rate Limiter (shared state)
// ---------------------------------------------------------------------------

/// A concurrent, sharded rate limiter backed by a [`DashMap`].
pub struct RateLimiter {
    /// Client key → token bucket.
    buckets: DashMap<String, TokenBucket>,
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Create a new rate limiter with the given config.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            buckets: DashMap::new(),
            config,
        }
    }

    /// Create with default config (env vars or hardcoded defaults).
    pub fn from_env() -> Self {
        Self::new(RateLimitConfig::from_env())
    }

    /// Check whether a request from `client_key` is allowed.
    ///
    /// Returns `Ok(())` if allowed, or `Err(RateLimitResponse)` with a 429
    /// payload including a `Retry-After` header.
    pub fn check(&self, client_key: &str) -> Result<(), RateLimitResponse> {
        RATE_LIMIT_TOTAL.with_label_values(&[client_key]).inc();

        let bucket = self
            .buckets
            .entry(client_key.to_owned())
            .or_insert_with(|| TokenBucket::new(self.config.burst_size));

        if bucket.try_acquire(&self.config) {
            Ok(())
        } else {
            RATE_LIMIT_REJECTED_TOTAL
                .with_label_values(&[client_key])
                .inc();
            let retry_after = bucket.retry_after_secs(&self.config);
            Err(RateLimitResponse { retry_after })
        }
    }

    /// Reset / remove the bucket for a specific client.
    pub fn reset(&self, client_key: &str) {
        self.buckets.remove(client_key);
    }

    /// Remove all buckets. Useful in tests.
    pub fn clear(&self) {
        self.buckets.clear();
    }

    /// Number of active buckets (rough gauge of unique clients).
    pub fn bucket_count(&self) -> usize {
        self.buckets.len()
    }
}

// ---------------------------------------------------------------------------
// 429 response type
// ---------------------------------------------------------------------------

/// A ready-to-return 429 response with `Retry-After`.
#[derive(Debug)]
pub struct RateLimitResponse {
    retry_after: u64,
}

impl IntoResponse for RateLimitResponse {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "error": {
                "code": "RATE_LIMITED",
                "message": "Too many requests. Please retry later.",
                "retry_after": self.retry_after,
            }
        });

        (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, self.retry_after.to_string())],
            axum::Json(body),
        )
            .into_response()
    }
}

// ---------------------------------------------------------------------------
// Client key extraction
// ---------------------------------------------------------------------------

/// Extract a client identity from the request.
///
/// Priority:
/// 1. `X-Forwarded-For` (first IP)
/// 2. `X-Real-IP`
/// 3. Fallback: `"unknown"`
fn client_key(req: &Request) -> String {
    if let Some(forwarded) = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(first_ip) = forwarded.split(',').next() {
            return format!("ip:{}", first_ip.trim());
        }
    }

    if let Some(ip) = req.headers().get("X-Real-IP").and_then(|v| v.to_str().ok()) {
        return format!("ip:{ip}");
    }

    "ip:unknown".to_owned()
}

// ---------------------------------------------------------------------------
// Axum middleware
// ---------------------------------------------------------------------------

/// Axum middleware function that enforces rate limits.
///
/// Use with `axum::middleware::from_fn_with_state`:
///
/// ```text
/// let limiter = std::sync::Arc::new(RateLimiter::from_env());
/// let app = Router::new()
///     .route("/*", get(handler))
///     .layer(middleware::from_fn_with_state(limiter.clone(), rate_limit_middleware));
/// ```
pub async fn rate_limit_middleware(
    axum::extract::State(limiter): axum::extract::State<std::sync::Arc<RateLimiter>>,
    req: Request,
    next: Next,
) -> Response {
    let key = client_key(&req);

    match limiter.check(&key) {
        Ok(()) => next.run(req).await,
        Err(rejection) => rejection.into_response(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_matches_env_defaults() {
        let cfg = RateLimitConfig::default();
        assert_eq!(cfg.max_requests, 100);
        assert_eq!(cfg.window_secs, 60);
        assert_eq!(cfg.burst_size, 100);
    }

    #[test]
    fn allows_requests_up_to_burst() {
        let cfg = RateLimitConfig {
            max_requests: 5,
            window_secs: 60,
            burst_size: 5,
        };
        let limiter = RateLimiter::new(cfg);

        for _ in 0..5 {
            assert!(limiter.check("client-a").is_ok());
        }
        assert!(limiter.check("client-a").is_err());
    }

    #[test]
    fn independent_clients() {
        let cfg = RateLimitConfig {
            max_requests: 2,
            window_secs: 60,
            burst_size: 2,
        };
        let limiter = RateLimiter::new(cfg);

        for _ in 0..2 {
            assert!(limiter.check("a").is_ok());
            assert!(limiter.check("b").is_ok());
        }

        assert!(limiter.check("a").is_err());
        assert!(limiter.check("b").is_err());
    }

    #[test]
    fn reset_allows_immediate_retry() {
        let cfg = RateLimitConfig {
            max_requests: 1,
            window_secs: 60,
            burst_size: 1,
        };
        let limiter = RateLimiter::new(cfg);

        assert!(limiter.check("x").is_ok());
        assert!(limiter.check("x").is_err());

        limiter.reset("x");
        assert!(limiter.check("x").is_ok());
    }

    #[test]
    fn rate_limit_response_has_retry_after_header() {
        let resp = RateLimitResponse { retry_after: 5 }.into_response();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        let val = resp.headers().get("retry-after").unwrap().to_str().unwrap();
        assert_eq!(val, "5");
    }

    #[test]
    fn client_key_uses_forwarded_for_when_present() {
        let req = Request::builder()
            .header("X-Forwarded-For", "1.2.3.4")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(client_key(&req), "ip:1.2.3.4");
    }

    #[test]
    fn client_key_falls_back_to_forwarded_for() {
        let req = Request::builder()
            .header("X-Forwarded-For", "10.0.0.1, 172.16.0.1")
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(client_key(&req), "ip:10.0.0.1");
    }

    #[test]
    fn client_key_unknown_fallback() {
        let req = Request::builder().body(axum::body::Body::empty()).unwrap();
        assert_eq!(client_key(&req), "ip:unknown");
    }
}
