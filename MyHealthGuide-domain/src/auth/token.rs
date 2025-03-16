use thiserror::Error;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Algorithm, Validation};
use std::env;
use tracing::{debug, error, info};
use chrono::{Duration, Utc};
use crate::auth::Claims;
use crate::auth::token_blacklist;

/// Security errors for authentication and token operations
#[derive(Debug, Error)]
pub enum SecurityError {
    /// JWT validation error
    #[error("Token validation error: {0}")]
    TokenValidation(String),

    /// Expired token
    #[error("Token has expired")]
    TokenExpired,

    /// Token not yet valid
    #[error("Token is not yet valid")]
    TokenNotYetValid,

    /// Invalid token structure
    #[error("Invalid token format")]
    InvalidToken,

    /// Configuration error
    #[error("Security configuration error: {0}")]
    ConfigError(String),

    /// Token has been revoked
    #[error("Token has been revoked")]
    TokenRevoked,

    /// Generic error
    #[error("Security error: {0}")]
    Generic(String),

    /// Invalid token format
    #[error("Invalid token format")]
    InvalidFormat,

    /// Malformed token
    #[error("Malformed token")]
    MalformedToken,

    /// Missing JWK
    #[error("Missing JWK")]
    MissingJWK,

    /// Invalid issuer
    #[error("Invalid token issuer")]
    InvalidIssuer,

    /// Invalid audience
    #[error("Invalid token audience")]
    InvalidAudience,
}

/// Token types for authentication
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    /// Short-lived access token
    Access,
    /// Long-lived refresh token
    Refresh,
}

impl TokenType {
    /// Get the expiration duration for this token type
    fn expiration(&self) -> Duration {
        match self {
            TokenType::Access => {
                // Access tokens expire in 15 minutes
                let expiration_minutes = env::var("ACCESS_TOKEN_EXPIRATION_MINUTES")
                    .unwrap_or_else(|_| "15".to_string())
                    .parse::<i64>()
                    .unwrap_or(15);

                Duration::minutes(expiration_minutes)
            },
            TokenType::Refresh => {
                // Refresh tokens expire in 7 days
                let expiration_days = env::var("REFRESH_TOKEN_EXPIRATION_DAYS")
                    .unwrap_or_else(|_| "7".to_string())
                    .parse::<i64>()
                    .unwrap_or(7);

                Duration::days(expiration_days)
            }
        }
    }
}

/// Generate a new JWT token
pub fn generate_token(
    user_id: &str,
    token_type: TokenType,
    _roles: Option<Vec<String>>,
) -> Result<String, SecurityError> {
    // Load JWT secret from environment
    let jwt_secret = env::var("JWT_SECRET").map_err(|e| {
        error!("JWT_SECRET environment variable not found: {}", e);
        SecurityError::ConfigError("JWT_SECRET environment variable not found".to_string())
    })?;

    // Get issuer and audience from environment variables
    let issuer = std::env::var("JWT_ISSUER")
        .unwrap_or_else(|_| "MyHealthGuide-api".to_string());
    let _audience = std::env::var("JWT_AUDIENCE")
        .unwrap_or_else(|_| "MyHealthGuide-client".to_string());

    // Current time and expiration
    let now = Utc::now();
    let expiration = now + token_type.expiration();

    // Create claims
    let claims = Claims {
        sub: user_id.to_string(),
        iss: issuer,
        iat: now.timestamp(),
        exp: expiration.timestamp(),
    };

    // Encode the token
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    ).map_err(|e| {
        error!("Failed to encode JWT token: {}", e);
        SecurityError::TokenValidation(e.to_string())
    })?;

    // Log token generation (but not the token itself)
    info!("Generated {:?} token for user {}", token_type, user_id);
    debug!("Token expiration: {}", expiration);

    Ok(token)
}

/// Validate a JWT token and return the decoded claims
pub fn validate_token(token: &str) -> Result<Claims, SecurityError> {
    // Load JWT secret from environment
    let jwt_secret = env::var("JWT_SECRET").map_err(|e| {
        error!("JWT_SECRET environment variable not found: {}", e);
        SecurityError::ConfigError("JWT_SECRET environment variable not found".to_string())
    })?;

    // Get issuer and audience from environment variables
    let issuer = std::env::var("JWT_ISSUER")
        .unwrap_or_else(|_| "MyHealthGuide-api".to_string());
    let _audience = std::env::var("JWT_AUDIENCE")
        .unwrap_or_else(|_| "MyHealthGuide-client".to_string());

    // Set up validation
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.set_issuer(&[issuer]);

    // Decode the token
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    ).map_err(|e| {
        match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => SecurityError::TokenExpired,
            jsonwebtoken::errors::ErrorKind::InvalidToken => SecurityError::InvalidToken,
            jsonwebtoken::errors::ErrorKind::InvalidSignature => SecurityError::TokenValidation("Invalid signature".to_string()),
            _ => SecurityError::TokenValidation(e.to_string()),
        }
    })?;

    // Check if token has been revoked
    if is_token_revoked(&token_data.claims.sub)? {
        return Err(SecurityError::TokenRevoked);
    }

    Ok(token_data.claims)
}

/// Check if a token has been revoked
fn is_token_revoked(user_id: &str) -> Result<bool, SecurityError> {
    // Check the token blacklist
    let is_revoked = token_blacklist::blacklist().is_revoked(user_id);
    debug!("Checking if token for user {} is revoked: {}", user_id, is_revoked);
    Ok(is_revoked)
}

/// Revoke a user's tokens
pub fn revoke_token(user_id: &str) -> Result<(), SecurityError> {
    // In a real application, this would add the token to a revocation list
    info!("Revoking tokens for user {}", user_id);

    // Add token to blacklist with an expiration time
    // We'll use a generous expiration time to ensure it's blacklisted long enough
    // In a real app, you might want to use the actual token expiration time
    let expiration = std::time::SystemTime::now() + std::time::Duration::from_secs(86400); // 24 hours
    token_blacklist::blacklist().revoke_token(user_id, expiration);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_env() {
        std::env::set_var("JWT_SECRET", "test_secret_key_for_testing_only");
        std::env::set_var("JWT_ISSUER", "test-issuer");
        std::env::set_var("JWT_AUDIENCE", "test-audience");
    }

    #[test]
    fn test_generate_and_validate_token() {
        setup_test_env();

        let user_id = "test-user-123";
        let token = generate_token(user_id, TokenType::Access, None).unwrap();

        // Token should be a non-empty string
        assert!(!token.is_empty());

        // Should be able to validate the token
        let claims = validate_token(&token).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.iss, "test-issuer");
    }

    #[test]
    fn test_token_expiration() {
        setup_test_env();

        // Generate a token with a fixed expiration time in the past
        let user_id = "test-user-456";

        // Create claims with expiration in the past
        let claims = Claims {
            sub: user_id.to_string(),
            iss: "test-issuer".to_string(),
            iat: Utc::now().timestamp(),
            exp: Utc::now().timestamp() - 3600, // 1 hour in the past
        };

        // Encode token directly with expired claim
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or("test-secret".to_string());
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        ).unwrap();

        // Validating this explicitly expired token should fail
        let result = validate_token(&token);
        assert!(result.is_err(), "Token validation should fail for expired token");

        // Check that it's the right kind of error
        match result {
            Err(SecurityError::TokenExpired) => {}, // Expected error
            err => panic!("Expected TokenExpired error but got: {:?}", err),
        }
    }

    #[test]
    fn test_invalid_token() {
        setup_test_env();

        // Try to validate an invalid token
        let result = validate_token("invalid.token.format");
        assert!(result.is_err());

        match result {
            Err(SecurityError::InvalidToken) | Err(SecurityError::TokenValidation(_)) => {}, // Expected errors
            _ => panic!("Expected InvalidToken or TokenValidation error"),
        }
    }

    #[test]
    fn test_different_token_types() {
        setup_test_env();

        // Reset the expiration times for this test
        std::env::set_var("ACCESS_TOKEN_EXPIRATION_MINUTES", "15");
        std::env::set_var("REFRESH_TOKEN_EXPIRATION_DAYS", "7");

        // Access token should expire in minutes (15 by default)
        let access_token_exp = TokenType::Access.expiration();
        assert_eq!(access_token_exp, Duration::minutes(15));

        // Refresh token should expire in days (7 by default)
        let refresh_token_exp = TokenType::Refresh.expiration();
        assert_eq!(refresh_token_exp, Duration::days(7));
    }
}
