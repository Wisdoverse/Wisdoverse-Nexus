//! Unified API error handling for Nexis Gateway.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug)]
pub enum AppError {
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    ValidationError(Vec<ValidationField>),
    RateLimited { retry_after: u64 },
    Internal(String),
    Conflict(String),
    BadRequest(String),
}

#[derive(Debug, Serialize)]
pub struct ValidationField {
    pub field: String,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message, details) = match &self {
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "AUTH_UNAUTHORIZED", msg.clone(), None),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "AUTH_FORBIDDEN", msg.clone(), None),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone(), None),
            AppError::ValidationError(fields) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VALIDATION_ERROR",
                "Request validation failed".to_string(),
                Some(json!(fields)),
            ),
            AppError::RateLimited { retry_after } => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMITED",
                "Too many requests".to_string(),
                Some(json!({ "retry_after_seconds": retry_after })),
            ),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.clone(), None),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone(), None),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone(), None),
        };

        let body = serde_json::to_string(&ApiError {
            error: ErrorDetail {
                code: code.to_string(),
                message,
                details,
                request_id: None,
            },
        })
        .unwrap_or_else(|_| r#"{"error":{"code":"INTERNAL_ERROR","message":"Failed to serialize error"}}"#.to_string());

        let mut response = (status, body).into_response();
        if let AppError::RateLimited { retry_after } = self {
            response.headers_mut().insert(
                "Retry-After",
                retry_after.to_string().parse().unwrap_or_default(),
            );
        }
        response
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::ValidationError(_) => write!(f, "Validation error"),
            AppError::RateLimited { retry_after } => write!(f, "Rate limited, retry after {}s", retry_after),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn unauthorized_returns_401() {
        let err = AppError::Unauthorized("invalid token".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn not_found_returns_404() {
        let err = AppError::NotFound("user not found".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn rate_limited_returns_429_with_header() {
        let err = AppError::RateLimited { retry_after: 60 };
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(resp.headers().get("Retry-After").unwrap(), "60");
    }

    #[test]
    fn validation_error_returns_422() {
        let err = AppError::ValidationError(vec![
            ValidationField { field: "email".to_string(), message: "invalid format".to_string() },
        ]);
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
