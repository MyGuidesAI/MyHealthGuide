use std::error::Error as StdError;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};
use reqwest::Client;
use std::time::{Duration, SystemTime, Instant};
use once_cell::sync::Lazy;
use std::env;
use std::collections::HashMap;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

use crate::auth::UserInfo;
use crate::auth::token::SecurityError;
use crate::auth::logging::log_token_validation;

// Define the types we need for JWKS handling
type JWKSet = HashMap<String, serde_json::Value>;

/// Auth0 JWKS cache
static JWKS_CACHE: Lazy<JwksCache> = Lazy::new(|| {
    JwksCache::new()
});

/// Auth0 JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Auth0Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Issuer
    pub iss: String,
    /// Audience (as a string or array of strings)
    #[serde(default)]
    pub aud: serde_json::Value,
    /// Issued at
    pub iat: i64,
    /// Expiration
    pub exp: i64,
    /// Authorized party
    #[serde(skip_serializing_if = "Option::is_none")]
    pub azp: Option<String>,
    /// Scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Email verified
    #[serde(rename = "email_verified", skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    /// Name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Nickname
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    /// Picture URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    /// Updated at
    #[serde(rename = "updated_at", skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// Roles - might be present directly in some Auth0 configurations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    /// Permissions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Vec<String>>,
    /// All other custom claims that might contain roles or other attributes
    #[serde(flatten)]
    pub custom_claims: std::collections::HashMap<String, serde_json::Value>,
}

/// JWKS Cache for Auth0 JWTs
struct JwksCache {
    /// JWKS keys keyed by issuer URL
    keys: std::sync::Mutex<HashMap<String, (JWKSet, SystemTime)>>,
    /// HTTP client for fetching JWKS
    client: Client,
    /// Cache expiration (default: 24 hours)
    cache_expiration: Duration,
}

impl JwksCache {
    /// Create a new JWKS cache
    pub fn new() -> Self {
        let cache_hours = env::var("JWKS_CACHE_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse::<u64>()
            .unwrap_or(24);

        Self {
            keys: std::sync::Mutex::new(HashMap::new()),
            client: Client::new(),
            cache_expiration: Duration::from_secs(cache_hours * 3600),
        }
    }

    /// Get JWKS for a given issuer
    pub async fn get_jwks(&self, issuer: &str) -> Result<JWKSet, Box<dyn StdError + Send + Sync>> {
        let cached = {
            let cache = self.keys.lock().unwrap();
            cache.get(issuer).cloned()
        };

        match cached {
            Some((jwks, timestamp)) if SystemTime::now().duration_since(timestamp)? < self.cache_expiration => {
                debug!("Using cached JWKS for issuer: {}", issuer);
                Ok(jwks)
            },
            _ => {
                debug!("Fetching JWKS for issuer: {}", issuer);
                // Ensure the issuer URL ends with a slash
                let issuer_url = if issuer.ends_with('/') {
                    issuer.to_string()
                } else {
                    format!("{}/", issuer)
                };

                let jwks_url = format!("{}/.well-known/jwks.json", issuer_url);
                debug!("JWKS URL: {}", jwks_url);

                let response = self.client.get(&jwks_url).send().await?;

                if !response.status().is_success() {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to fetch JWKS: {}", response.status())
                    )));
                }

                let jwks: JWKSet = response.json().await?;

                // Update cache
                {
                    let mut cache = self.keys.lock().unwrap();
                    cache.insert(issuer.to_string(), (jwks.clone(), SystemTime::now()));
                }

                Ok(jwks)
            }
        }
    }
}

/// Validate an Auth0 JWT token
pub async fn validate_auth0_token(token: &str) -> Result<UserInfo, SecurityError> {
    // Start timing for performance tracking
    let start = Instant::now();

    // Extract and decode JWT headers
    let Some(header_b64) = token.split('.').next() else {
        return Err(SecurityError::InvalidFormat);
    };

    // Decode header
    let header_json = match URL_SAFE_NO_PAD.decode(header_b64) {
        Ok(decoded) => match String::from_utf8(decoded) {
            Ok(json_str) => json_str,
            Err(_) => return Err(SecurityError::InvalidFormat)
        },
        Err(_) => return Err(SecurityError::InvalidFormat)
    };

    // Parse header
    let header: serde_json::Value = match serde_json::from_str(&header_json) {
        Ok(json) => json,
        Err(_) => return Err(SecurityError::InvalidFormat)
    };

    // Extract token kid (key ID)
    let kid = match header.get("kid") {
        Some(kid_value) => match kid_value.as_str() {
            Some(kid_str) => kid_str,
            None => return Err(SecurityError::MalformedToken)
        },
        None => return Err(SecurityError::MalformedToken)
    };

    debug!("Auth0 token kid: {}", kid);

    // Extract token claims
    let claims: Auth0Claims = match decode_token_claims(token) {
        Ok(claims) => claims,
        Err(e) => {
            debug!("Failed to decode Auth0 token claims: {}", e);
            return Err(SecurityError::MalformedToken);
        }
    };

    // Get the issuer from Auth0 claims
    let issuer = &claims.iss;

    debug!("Auth0 token issuer: {}", issuer);

    // Check token expiration
    if claims.exp < chrono::Utc::now().timestamp() {
        debug!("Auth0 token expired for user: {}", claims.sub);
        return Err(SecurityError::TokenExpired);
    }

    // Validate issuer
    let auth0_domain = env::var("AUTH0_DOMAIN").unwrap_or_else(|_| "".to_string());
    if !auth0_domain.is_empty() && !issuer.contains(&auth0_domain) {
        warn!("Invalid Auth0 token issuer: {}", issuer);
        return Err(SecurityError::InvalidIssuer);
    }

    // Validate audience
    let audience = env::var("AUTH0_AUDIENCE").unwrap_or_else(|_| "".to_string());
    if !audience.is_empty() {
        let valid_audience = match &claims.aud {
            serde_json::Value::String(aud_str) => aud_str == &audience,
            serde_json::Value::Array(aud_array) => {
                aud_array.iter().any(|aud| aud.as_str().is_some_and(|s| s == audience))
            },
            _ => false
        };

        if !valid_audience {
            warn!("Invalid Auth0 token audience for user {}", claims.sub);
            return Err(SecurityError::InvalidAudience);
        }
    }

    // Get JWKS from cache or fetch from Auth0
    let jwks = match JWKS_CACHE.get_jwks(issuer).await {
        Ok(jwks) => jwks,
        Err(e) => {
            error!("Failed to get JWKS: {}", e);
            return Err(SecurityError::MissingJWK);
        }
    };

    // Find the key matching the token kid
    let _jwk = match jwks.get("keys") {
        Some(serde_json::Value::Array(keys)) => {
            let matching_key = keys.iter().find(|key| {
                key.get("kid").and_then(|k| k.as_str()) == Some(kid)
            });

            match matching_key {
                Some(key) => key,
                None => {
                    warn!("No matching key found for kid: {}", kid);
                    return Err(SecurityError::MissingJWK);
                }
            }
        },
        _ => {
            warn!("Invalid JWKS format");
            return Err(SecurityError::MissingJWK);
        }
    };

    // TODO: Actually verify token signature with JWK
    // For now, we're assuming the token is valid if it passes all the checks above
    // In a production environment, you would use a JWT library to verify the signature

    // Extract roles from the token
    let roles = extract_roles_from_claims(&claims);
    debug!("Auth0 token roles: {:?}", roles);

    // Create user info from token claims
    let user_info = UserInfo {
        user_id: claims.sub.clone(),
        roles,
        email: claims.email.clone(),
        name: claims.name.clone(),
        picture: None, // Auth0 claim structure doesn't have a standard picture field
        auth_source: "auth0".to_string(),
    };

    // Log successful validation
    log_token_validation(&user_info.user_id, "auth0", true);

    // Log performance
    let duration = start.elapsed();
    debug!("Auth0 token validation took {}ms", duration.as_millis());

    Ok(user_info)
}

/// Extract roles from Auth0 claims in various formats
fn extract_roles_from_claims(claims: &Auth0Claims) -> Vec<String> {
    let mut roles = Vec::new();

    // 1. Check for direct roles claim
    if let Some(direct_roles) = &claims.roles {
        roles.extend(direct_roles.clone());
    }

    // 2. Check for namespaced roles in custom claims
    // Auth0 often uses a namespace like "https://myapp.com/roles"
    for (key, value) in &claims.custom_claims {
        if key.ends_with("/roles") || key.contains("roles") {
            if let Some(array) = value.as_array() {
                for item in array {
                    if let Some(role) = item.as_str() {
                        roles.push(role.to_string());
                    }
                }
            }
        }
    }

    // 3. Check permissions (which can be used as fine-grained roles)
    if let Some(permissions) = &claims.permissions {
        // Convert permissions to roles (optional - depends on your authorization model)
        // For example, a permission like "read:reports" could be considered a "report-reader" role
        roles.extend(permissions.clone());
    }

    // 4. Check scopes (OAuth2 scopes can sometimes contain role information)
    if let Some(scope_str) = &claims.scope {
        // Split scope string by spaces
        let scopes: Vec<&str> = scope_str.split(' ').collect();

        // Filter for role-like scopes (those containing the word "role" or your custom prefix)
        for scope in scopes {
            if scope.contains("role:") || scope.starts_with("role_") {
                roles.push(scope.to_string());
            }
        }
    }

    // Always include the default "user" role for authenticated users
    if !roles.contains(&"user".to_string()) {
        roles.push("user".to_string());
    }

    roles
}

/// Decode token claims without validation
pub fn decode_token_claims(token: &str) -> Result<Auth0Claims, Box<dyn StdError + Send + Sync>> {
    // Get the claims part (second section of JWT)
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid token format"
        )));
    }

    // Decode and parse claims
    let claims_b64 = parts[1];
    let claims_json = match URL_SAFE_NO_PAD.decode(claims_b64) {
        Ok(decoded) => match String::from_utf8(decoded) {
            Ok(json_str) => json_str,
            Err(e) => return Err(Box::new(e))
        },
        Err(e) => return Err(Box::new(e))
    };

    // Parse the JSON claims
    let claims: Auth0Claims = match serde_json::from_str(&claims_json) {
        Ok(claims) => claims,
        Err(e) => return Err(Box::new(e))
    };

    Ok(claims)
}

/// Custom claims validator
#[allow(dead_code)]
fn validate_claims(audience: &str, issuer: &str) -> impl Fn(serde_json::Value) -> Result<(), SecurityError> + Clone {
    let audience = audience.to_string();
    let issuer = issuer.to_string();

    move |claims_json| {
        // Parse the claims
        let claims: serde_json::Value = match claims_json {
            serde_json::Value::Object(map) => serde_json::Value::Object(map),
            _ => return Err(SecurityError::MalformedToken)
        };

        // Check issuer
        let claim_issuer = claims.get("iss").and_then(|v| v.as_str());
        let issuer_str = issuer.as_str();
        if claim_issuer != Some(issuer_str) {
            debug!("Issuer mismatch: {:?} != {}", claim_issuer, issuer_str);
            return Err(SecurityError::InvalidIssuer);
        }

        // Check audience (Auth0 tokens might have either a string or array audience)
        let audience_str = audience.as_str();
        let audience_match = match claims.get("aud") {
            Some(serde_json::Value::String(aud)) => aud == audience_str,
            Some(serde_json::Value::Array(aud_array)) => {
                aud_array.iter().any(|a| a.as_str() == Some(audience_str))
            },
            _ => false,
        };

        if !audience_match {
            debug!("Audience mismatch: audience doesn't contain {}", audience_str);
            return Err(SecurityError::InvalidAudience);
        }

        Ok(())
    }
}

/// Parse token claims without verification (used in tests)
pub fn parse_token_claims(token: &str) -> Result<Auth0Claims, Box<dyn StdError + Send + Sync>> {
    // This is just an alias for decode_token_claims for backward compatibility
    decode_token_claims(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_token_claims() {
        // This is a sample token structure, not a real token
        let token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwiaXNzIjoiaHR0cHM6Ly9leGFtcGxlLmF1dGgwLmNvbS8iLCJhdWQiOlsibXktYXBpIl0sImlhdCI6MTUxNjIzOTAyMiwiZXhwIjo5OTk5OTk5OTk5fQ.signature";

        let claims = parse_token_claims(token).unwrap();
        assert_eq!(claims.sub, "1234567890");
        assert_eq!(claims.iss, "https://example.auth0.com/");
    }

    #[test]
    fn test_extract_roles_from_claims() {
        // Test with direct roles
        let claims = Auth0Claims {
            sub: "user123".to_string(),
            iss: "https://example.auth0.com/".to_string(),
            aud: serde_json::json!(["api"]),
            iat: 0,
            exp: 0,
            azp: None,
            scope: None,
            email: None,
            email_verified: None,
            name: None,
            nickname: None,
            picture: None,
            updated_at: None,
            roles: Some(vec!["admin".to_string(), "editor".to_string()]),
            permissions: None,
            custom_claims: HashMap::new(),
        };

        let roles = extract_roles_from_claims(&claims);
        assert!(roles.contains(&"admin".to_string()));
        assert!(roles.contains(&"editor".to_string()));
        assert!(roles.contains(&"user".to_string()));

        // Test with namespaced roles
        let mut custom_claims = HashMap::new();
        custom_claims.insert(
            "https://myapp.com/roles".to_string(),
            serde_json::json!(["manager", "approver"])
        );

        let claims = Auth0Claims {
            sub: "user123".to_string(),
            iss: "https://example.auth0.com/".to_string(),
            aud: serde_json::json!(["api"]),
            iat: 0,
            exp: 0,
            azp: None,
            scope: None,
            email: None,
            email_verified: None,
            name: None,
            nickname: None,
            picture: None,
            updated_at: None,
            roles: None,
            permissions: None,
            custom_claims,
        };

        let roles = extract_roles_from_claims(&claims);
        assert!(roles.contains(&"manager".to_string()));
        assert!(roles.contains(&"approver".to_string()));
        assert!(roles.contains(&"user".to_string()));
    }
}
