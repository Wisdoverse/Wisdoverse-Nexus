//! Authentication module for Nexus Gateway
//!
//! Shared JWT configuration and identity verification for HTTP and WebSocket routes.

#![allow(dead_code)]

use std::sync::OnceLock;

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[cfg(feature = "multi-tenant")]
mod tenant;
#[cfg(feature = "multi-tenant")]
pub use tenant::{TenantContext, TenantError, TenantExtractor, TenantStore};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub aud: String,
    pub member_type: String,
    #[cfg(feature = "multi-tenant")]
    #[serde(default)]
    pub tenant_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Missing authorization header")]
    MissingHeader,
    #[error("Invalid header format")]
    InvalidHeaderFormat,
    #[cfg(feature = "multi-tenant")]
    #[error("Tenant context required")]
    TenantRequired,
}

#[derive(Clone)]
pub struct JwtConfig {
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
    pub issuer: String,
    pub audience: String,
    pub expiry_seconds: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum JwtConfigError {
    #[error("JWT_SECRET is required and must not be blank when NEXIS_ENV=production")]
    MissingProductionSecret,
    #[error("JWT_SECRET must contain valid Unicode")]
    InvalidSecretEncoding,
}

static JWT_CONFIG: OnceLock<JwtConfig> = OnceLock::new();

fn configured_secret(secret: Option<String>, production: bool) -> Result<String, JwtConfigError> {
    if let Some(secret) = secret.filter(|value| !value.trim().is_empty()) {
        return Ok(secret);
    }
    if production {
        return Err(JwtConfigError::MissingProductionSecret);
    }
    tracing::warn!(
        "JWT_SECRET is missing or blank. Using a development-only random secret; \
         externally issued tokens require an explicitly configured JWT_SECRET."
    );
    Ok(uuid::Uuid::new_v4().to_string())
}

impl JwtConfig {
    pub fn new(secret: &str, issuer: String, audience: String) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            issuer,
            audience,
            expiry_seconds: 3600,
        }
    }

    /// Load configuration without exposing secret values in errors or logs.
    pub fn from_env() -> Result<Self, JwtConfigError> {
        let secret = match std::env::var("JWT_SECRET") {
            Ok(value) => Some(value),
            Err(std::env::VarError::NotPresent) => None,
            Err(std::env::VarError::NotUnicode(_)) => {
                return Err(JwtConfigError::InvalidSecretEncoding);
            }
        };
        let production = std::env::var("NEXIS_ENV")
            .is_ok_and(|value| value.trim().eq_ignore_ascii_case("production"));
        let secret = configured_secret(secret, production)?;
        Ok(Self::new(
            &secret,
            std::env::var("JWT_ISSUER").unwrap_or_else(|_| "nexis".to_string()),
            std::env::var("JWT_AUDIENCE").unwrap_or_else(|_| "nexis".to_string()),
        ))
    }

    /// Validate shared HTTP/WS configuration before opening a listener.
    pub fn try_cached() -> Result<&'static Self, JwtConfigError> {
        if let Some(config) = JWT_CONFIG.get() {
            return Ok(config);
        }
        let config = Self::from_env()?;
        Ok(JWT_CONFIG.get_or_init(|| config))
    }

    /// Get the validated, process-wide HTTP/WS configuration.
    pub fn cached() -> &'static Self {
        Self::try_cached().expect("JWT configuration must be valid before gateway startup")
    }

    pub fn generate_token(&self, member_id: &str, member_type: &str) -> Result<String, AuthError> {
        self.generate_token_with_tenant(member_id, member_type, None)
    }

    #[cfg(feature = "multi-tenant")]
    pub fn generate_token_with_tenant(
        &self,
        member_id: &str,
        member_type: &str,
        tenant_id: Option<&str>,
    ) -> Result<String, AuthError> {
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: member_id.to_string(),
            exp: now + self.expiry_seconds as usize,
            iat: now,
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            member_type: member_type.to_string(),
            tenant_id: tenant_id.map(|s| s.to_string()),
        };

        encode(&Header::default(), &claims, &self.encoding_key).map_err(|_| AuthError::InvalidToken)
    }

    #[cfg(not(feature = "multi-tenant"))]
    pub fn generate_token_with_tenant(
        &self,
        member_id: &str,
        member_type: &str,
        _tenant_id: Option<&str>,
    ) -> Result<String, AuthError> {
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: member_id.to_string(),
            exp: now + self.expiry_seconds as usize,
            iat: now,
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            member_type: member_type.to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key).map_err(|_| AuthError::InvalidToken)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        validation.leeway = 0;

        #[cfg(not(feature = "multi-tenant"))]
        if let Ok(data) = decode::<serde_json::Value>(token, &self.decoding_key, &validation) {
            if data
                .claims
                .get("tenant_id")
                .is_some_and(|value| !value.is_null())
            {
                return Err(AuthError::InvalidToken);
            }
        }
        decode::<Claims>(token, &self.decoding_key, &validation)
            .and_then(|data| {
                if data.claims.sub.trim().is_empty()
                    || !matches!(data.claims.member_type.as_str(), "human" | "ai")
                {
                    return Err(jsonwebtoken::errors::ErrorKind::InvalidSubject.into());
                }
                Ok(data.claims)
            })
            .map_err(|e| {
                if e.kind() == &jsonwebtoken::errors::ErrorKind::ExpiredSignature {
                    AuthError::TokenExpired
                } else {
                    AuthError::InvalidToken
                }
            })
    }

    #[cfg(test)]
    pub fn test_token(member_id: &str) -> String {
        let config = Self::new("test-secret", "test".to_string(), "test".to_string());
        config.generate_token(member_id, "human").unwrap()
    }
}

pub struct AuthenticatedUser {
    pub member_id: String,
    pub member_type: String,
    pub claims: Claims,
    #[cfg(feature = "multi-tenant")]
    pub tenant_context: Option<TenantContext>,
}

/// Verified identity for externally issued JWTs. This endpoint does not issue tokens.
pub async fn session(user: AuthenticatedUser) -> axum::Json<serde_json::Value> {
    let mut value = serde_json::json!({
        "memberId": user.member_id,
        "memberType": user.member_type,
        "expiresAt": user.claims.exp.saturating_mul(1000),
    });
    #[cfg(feature = "multi-tenant")]
    if let Some(tenant_id) = user.claims.tenant_id {
        value["tenantId"] = tenant_id.into();
    }
    // Keep one response shape across feature configurations.
    value["refreshSupported"] = false.into();
    axum::Json(value)
}

impl AuthenticatedUser {
    /// Alias for member_id for convenience
    pub fn user_id(&self) -> &str {
        &self.member_id
    }
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok());

        let Some(header_value) = auth_header else {
            return Err(StatusCode::UNAUTHORIZED);
        };

        if !header_value.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let token = &header_value[7..];

        // Use test config in test environment, cached production config otherwise
        #[cfg(test)]
        let config = JwtConfig::new("test-secret", "test".to_string(), "test".to_string());

        #[cfg(not(test))]
        let config = JwtConfig::cached();

        let claims = config
            .verify_token(token)
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        #[cfg(feature = "multi-tenant")]
        {
            let tenant_context = extract_tenant_from_claims(&claims);
            Ok(AuthenticatedUser {
                member_id: claims.sub.clone(),
                member_type: claims.member_type.clone(),
                claims,
                tenant_context,
            })
        }

        #[cfg(not(feature = "multi-tenant"))]
        {
            Ok(AuthenticatedUser {
                member_id: claims.sub.clone(),
                member_type: claims.member_type.clone(),
                claims,
            })
        }
    }
}

#[cfg(feature = "multi-tenant")]
pub fn extract_tenant_from_claims(claims: &Claims) -> Option<TenantContext> {
    claims.tenant_id.as_ref().map(|tid| TenantContext {
        tenant_id: tid.clone(),
    })
}

#[cfg(feature = "multi-tenant")]
pub fn check_tenant_access(
    user_tenant: &TenantContext,
    resource_tenant: &str,
) -> Result<(), TenantError> {
    if user_tenant.tenant_id == resource_tenant {
        Ok(())
    } else {
        Err(TenantError::CrossTenantAccess {
            user_tenant: user_tenant.tenant_id.clone(),
            resource_tenant: resource_tenant.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_requires_a_nonblank_signing_secret() {
        for secret in [None, Some(String::new()), Some(" \t\n".to_string())] {
            assert!(matches!(
                configured_secret(secret, true),
                Err(JwtConfigError::MissingProductionSecret)
            ));
        }
    }

    #[test]
    fn configured_signing_secret_is_preserved_in_both_modes() {
        for production in [false, true] {
            assert_eq!(
                configured_secret(Some("explicit-secret".to_string()), production).unwrap(),
                "explicit-secret"
            );
        }
    }

    #[test]
    fn development_secret_is_ephemeral() {
        let first = configured_secret(None, false).unwrap();
        let second = configured_secret(Some(" ".to_string()), false).unwrap();
        assert!(!first.is_empty());
        assert_ne!(first, second);
    }

    #[test]
    fn jwt_config_generates_and_verifies_token() {
        let config = JwtConfig::new(
            "test_secret_key_that_is_long_enough",
            "nexis-test".to_string(),
            "nexis".to_string(),
        );

        let token = config
            .generate_token("nexis:human:alice@example.com", "human")
            .unwrap();
        let claims = config.verify_token(&token).unwrap();

        assert_eq!(claims.sub, "nexis:human:alice@example.com");
        assert_eq!(claims.member_type, "human");
        assert_eq!(claims.iss, "nexis-test");
    }

    #[test]
    fn invalid_token_is_rejected() {
        let config = JwtConfig::new(
            "test_secret_key",
            "nexis-test".to_string(),
            "nexis".to_string(),
        );

        let result = config.verify_token("invalid_token");
        assert!(result.is_err());
    }

    #[cfg(feature = "multi-tenant")]
    mod multi_tenant_tests {
        use super::*;

        #[test]
        fn jwt_config_generates_token_with_tenant() {
            let config = JwtConfig::new(
                "test_secret_key_that_is_long_enough",
                "nexis-test".to_string(),
                "nexis".to_string(),
            );

            let token = config
                .generate_token_with_tenant(
                    "nexis:human:alice@example.com",
                    "human",
                    Some("tenant_acme"),
                )
                .unwrap();
            let claims = config.verify_token(&token).unwrap();

            assert_eq!(claims.sub, "nexis:human:alice@example.com");
            assert_eq!(claims.member_type, "human");
            assert_eq!(claims.tenant_id, Some("tenant_acme".to_string()));
        }

        #[test]
        fn jwt_config_generates_token_without_tenant() {
            let config = JwtConfig::new(
                "test_secret_key_that_is_long_enough",
                "nexis-test".to_string(),
                "nexis".to_string(),
            );

            let token = config
                .generate_token_with_tenant("nexis:human:alice@example.com", "human", None)
                .unwrap();
            let claims = config.verify_token(&token).unwrap();

            assert_eq!(claims.tenant_id, None);
        }

        #[test]
        fn extract_tenant_from_claims_returns_context() {
            let config = JwtConfig::new(
                "test_secret_key_that_is_long_enough",
                "nexis-test".to_string(),
                "nexis".to_string(),
            );

            let token = config
                .generate_token_with_tenant("user1", "human", Some("tenant_123"))
                .unwrap();
            let claims = config.verify_token(&token).unwrap();

            let tenant_ctx = extract_tenant_from_claims(&claims);
            assert!(tenant_ctx.is_some());
            assert_eq!(tenant_ctx.unwrap().tenant_id, "tenant_123");
        }

        #[test]
        fn extract_tenant_from_claims_returns_none_when_missing() {
            let config = JwtConfig::new(
                "test_secret_key_that_is_long_enough",
                "nexis-test".to_string(),
                "nexis".to_string(),
            );

            let token = config.generate_token("user1", "human").unwrap();
            let claims = config.verify_token(&token).unwrap();

            let tenant_ctx = extract_tenant_from_claims(&claims);
            assert!(tenant_ctx.is_none());
        }

        #[test]
        fn check_tenant_access_allows_same_tenant() {
            let user_tenant = TenantContext {
                tenant_id: "tenant_123".to_string(),
            };
            let result = check_tenant_access(&user_tenant, "tenant_123");
            assert!(result.is_ok());
        }

        #[test]
        fn check_tenant_access_rejects_cross_tenant() {
            let user_tenant = TenantContext {
                tenant_id: "tenant_123".to_string(),
            };
            let result = check_tenant_access(&user_tenant, "tenant_456");
            assert!(result.is_err());
            match result {
                Err(TenantError::CrossTenantAccess {
                    user_tenant,
                    resource_tenant,
                }) => {
                    assert_eq!(user_tenant, "tenant_123");
                    assert_eq!(resource_tenant, "tenant_456");
                }
                _ => panic!("Expected CrossTenantAccess error"),
            }
        }
    }
}
