use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// Registration request payload
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct PublicRegistrationRequest {
    /// Email address (must be valid format)
    #[validate(email(message = "Must be a valid email address"))]
    pub email: String,
    
    /// Password (must be at least 8 characters)
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    
    /// Optional full name
    pub name: Option<String>,
}

/// Login request payload
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct PublicLoginRequest {
    /// Email address
    #[validate(email(message = "Must be a valid email address"))]
    pub email: String,
    
    /// Password
    pub password: String,
}

/// Login response payload
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublicLoginResponse {
    /// JWT access token
    pub access_token: String,
    
    /// JWT refresh token
    pub refresh_token: String,
    
    /// Token type (Bearer)
    pub token_type: String,
    
    /// Expiration time in seconds
    pub expires_in: i64,
    
    /// User information
    pub user: PublicUserInfo,
}

/// User information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublicUserInfo {
    /// User ID
    pub user_id: String,
    
    /// Email address
    pub email: Option<String>,
    
    /// Full name
    pub name: Option<String>,
    
    /// Profile picture URL
    pub picture: Option<String>,
    
    /// User roles
    pub roles: Vec<String>,
    
    /// Authentication source (local, oidc, etc.)
    pub auth_source: String,
}

/// OIDC authentication start response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublicOidcStartResponse {
    /// Authorization URL to redirect the user to
    pub authorization_url: String,
    
    /// Session ID
    pub session_id: String,
}

/// Token refresh request
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct PublicTokenRefreshRequest {
    /// Refresh token
    pub refresh_token: String,
}

/// Token refresh response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublicTokenRefreshResponse {
    /// New access token
    pub access_token: String,
    
    /// New refresh token
    pub refresh_token: String,
    
    /// Token type (Bearer)
    pub token_type: String,
    
    /// Expiration time in seconds
    pub expires_in: i64,
}

/// JWT token claims
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublicTokenClaims {
    /// Subject (user ID)
    pub sub: String,
    
    /// Issued at timestamp
    pub iat: i64,
    
    /// Expiration timestamp
    pub exp: i64,
    
    /// User roles
    pub roles: Vec<String>,
} 