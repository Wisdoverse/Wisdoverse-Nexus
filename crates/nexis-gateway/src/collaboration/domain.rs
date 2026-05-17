//! Domain rules for collaboration gateway use cases.

use chrono::{DateTime, Utc};

pub const MAX_NAME_LEN: usize = 128;
pub const MAX_TITLE_LEN: usize = 200;
pub const MAX_IDENTIFIER_LEN: usize = 128;
pub const MAX_CONTENT_LEN: usize = 100_000;
pub const MAX_RATE_LIMIT_SUBJECT_LEN: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct CollaborationValidationError {
    message: String,
}

impl CollaborationValidationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn into_message(self) -> String {
        self.message
    }
}

pub fn validate_required_text(
    field: &str,
    value: &str,
    max_len: usize,
) -> Result<String, CollaborationValidationError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CollaborationValidationError::new(format!(
            "{field} is required"
        )));
    }
    if trimmed.len() > max_len {
        return Err(CollaborationValidationError::new(format!(
            "{field} exceeds maximum length of {max_len} characters"
        )));
    }

    Ok(trimmed.to_string())
}

pub fn validate_identifier(
    field: &str,
    value: &str,
    max_len: usize,
) -> Result<String, CollaborationValidationError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CollaborationValidationError::new(format!(
            "{field} is required"
        )));
    }
    if trimmed.len() > max_len {
        return Err(CollaborationValidationError::new(format!(
            "{field} exceeds maximum length of {max_len} characters"
        )));
    }
    if !trimmed
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err(CollaborationValidationError::new(format!(
            "{field} contains invalid characters; allowed: a-z, A-Z, 0-9, _ and -"
        )));
    }

    Ok(trimmed.to_string())
}

pub fn validate_time_window(
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
) -> Result<(), CollaborationValidationError> {
    if starts_at >= ends_at {
        return Err(CollaborationValidationError::new(
            "starts_at must be earlier than ends_at",
        ));
    }

    Ok(())
}

/// Fixed-window rate-limit policy for collaboration operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollaborationRateLimitPolicy {
    pub max_requests: u32,
    pub window_seconds: u64,
}

impl CollaborationRateLimitPolicy {
    /// Create a new rate-limit policy.
    pub fn new(max_requests: u32, window_seconds: u64) -> Result<Self, String> {
        if max_requests == 0 {
            return Err("max_requests must be greater than 0".to_string());
        }
        if window_seconds == 0 {
            return Err("window_seconds must be greater than 0".to_string());
        }

        Ok(Self {
            max_requests,
            window_seconds,
        })
    }

    /// Return true when `request_count` exceeds the configured maximum.
    pub const fn is_exceeded(self, request_count: u32) -> bool {
        request_count > self.max_requests
    }
}

/// Scope key used by collaboration endpoint throttling.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CollaborationRateLimitScope {
    Meetings,
    Documents,
    Tasks,
    Calendar,
}

/// Identity used for per-subject collaboration rate limiting.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CollaborationRateLimitKey {
    pub scope: CollaborationRateLimitScope,
    pub subject: String,
}

impl CollaborationRateLimitKey {
    /// Build a validated scope+subject key.
    pub fn new(
        scope: CollaborationRateLimitScope,
        subject: impl Into<String>,
    ) -> Result<Self, String> {
        let subject = validate_identifier("subject", &subject.into(), MAX_RATE_LIMIT_SUBJECT_LEN)
            .map_err(CollaborationValidationError::into_message)?;
        Ok(Self { scope, subject })
    }
}
