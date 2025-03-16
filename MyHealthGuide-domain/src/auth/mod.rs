//! Authentication module for MyHealthGuide API
//!
//! Provides authentication middleware for securing API endpoints using JWT

use axum::{
    extract::State,
    http::{Request, StatusCode, header},
    middleware::Next,
    response::Response,
    body::Body,
    Extension,
};
use std::env;
use tracing::{debug, warn, error};
use serde::{Deserialize, Serialize};
use jwt_simple::prelude::*;
use chrono::Utc;
use crate::auth::logging::{log_auth_event, AuthEvent, AuthEventType, log_token_refresh, log_logout};

#[cfg(feature = "with-api")]
use utoipa::ToSchema;

// Add token module for JWT handling
pub mod token;

// Token blacklist for revocation
pub mod token_blacklist;

// Make the OIDC module public
#[cfg(feature = "with-oidc")]
pub mod oidc;

// Make the routes module public
pub mod routes;

// Include auth0 module
pub mod auth0;

// Include authorization module for RBAC
pub mod authorize;

// Include OIDC tests
#[cfg(test)]
mod oidc_tests;

// Include Routes tests
#[cfg(test)]
mod routes_tests;

// Include logging module
pub mod logging;

/// Authentication claims for JSON Web Tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-api", derive(ToSchema))]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Issuer
    pub iss: String,
    /// Issued at (as timestamp)
    pub iat: i64,
    /// Expiration timestamp
    pub exp: i64,
}

/// User information extracted from authenticated requests
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-api", derive(ToSchema))]
pub struct UserInfo {
    /// User ID
    pub user_id: String,
    /// User roles
    pub roles: Vec<String>,
    /// User email from OIDC provider (if available)
    pub email: Option<String>,
    /// User display name from OIDC provider (if available)
    pub name: Option<String>,
    /// User profile picture URL from OIDC provider (if available)
    pub picture: Option<String>,
    /// Authentication source (e.g., "oidc", "jwt")
    pub auth_source: String,
}

/// Login request body
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-api", derive(ToSchema))]
pub struct LoginRequest {
    /// Username or email
    pub username: String,
    /// Password
    pub password: String,
}

/// Login response body
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-api", derive(ToSchema))]
pub struct LoginResponse {
    /// JWT access token
    pub access_token: String,
    /// JWT refresh token
    pub refresh_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// User information
    pub user: UserInfo,
}

/// Authentication middleware for protected routes
#[cfg(feature = "with-api")]
pub async fn auth_middleware<S>(
    _state: State<S>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // For development mode, bypass authentication if configured
    if cfg!(debug_assertions) && env::var("BYPASS_AUTH").is_ok() {
        debug!("Auth bypass enabled in development mode");
        return next.run(req).await;
    }

    // Get the request path for logging
    let request_path = req.uri().path().to_string();

    // Start timing the authentication process
    let start_time = std::time::Instant::now();

    // Extract the token from the Authorization header
    let auth_header = match req.headers().get(header::AUTHORIZATION) {
        Some(value) => match value.to_str() {
            Ok(auth_str) => auth_str,
            Err(_) => {
                warn!("Invalid Authorization header format");

                // Log auth failure
                let event = AuthEvent::new(AuthEventType::TokenValidation, None, false)
                    .with_details("Invalid Authorization header format")
                    .with_resource(request_path)
                    .with_duration(start_time.elapsed().as_millis() as u64)
                    .with_auth_method("jwt");

                log_auth_event(event);

                return Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::empty())
                    .unwrap_or_default();
            }
        },
        None => {
            debug!("Missing Authorization header");

            // Log missing auth header
            let event = AuthEvent::new(AuthEventType::TokenValidation, None, false)
                .with_details("Missing Authorization header")
                .with_resource(request_path)
                .with_duration(start_time.elapsed().as_millis() as u64)
                .with_auth_method("jwt");

            log_auth_event(event);

            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::empty())
                .unwrap_or_default();
        }
    };

    // Check if it's a Bearer token
    if !auth_header.starts_with("Bearer ") {
        warn!("Authorization header does not contain Bearer token");

        // Log invalid token format
        let event = AuthEvent::new(AuthEventType::TokenValidation, None, false)
            .with_details("Authorization header does not contain Bearer token")
            .with_resource(request_path)
            .with_duration(start_time.elapsed().as_millis() as u64)
            .with_auth_method("jwt");

        log_auth_event(event);

        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())
            .unwrap_or_default();
    }

    let token = &auth_header[7..]; // Skip "Bearer " prefix

    // First try our standard JWT validation
    match token::validate_token(token) {
        Ok(claims) => {
            debug!("Token validated successfully as internal JWT for user: {}", claims.sub);

            // Log successful authentication
            let duration = start_time.elapsed().as_millis() as u64;
            let event = AuthEvent::new(AuthEventType::TokenValidation, Some(&claims.sub), true)
                .with_details("Standard JWT validation successful")
                .with_resource(request_path)
                .with_duration(duration)
                .with_auth_method("jwt");

            log_auth_event(event);

            // Add user info to request extensions
            let user_info = UserInfo {
                user_id: claims.sub.clone(),
                roles: vec!["user".to_string()], // Default role, in a real app would come from token
                email: None,
                name: None,
                picture: None,
                auth_source: "jwt".to_string(),
            };

            req.extensions_mut().insert(user_info);
            req.extensions_mut().insert(claims);

            // Continue with the request
            next.run(req).await
        },
        Err(token::SecurityError::TokenExpired) => {
            warn!("Expired token");

            // Try to extract user ID from expired token for logging
            let user_id = match token::validate_token(token) {
                Ok(claims) => Some(claims.sub),
                Err(_) => None,
            };

            // Log token expired
            let event = AuthEvent::new(AuthEventType::TokenValidation, user_id.as_deref(), false)
                .with_details("JWT token has expired")
                .with_resource(request_path)
                .with_duration(start_time.elapsed().as_millis() as u64)
                .with_auth_method("jwt");

            log_auth_event(event);

            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::empty())
                .unwrap_or_default()
        },
        Err(token::SecurityError::TokenRevoked) => {
            warn!("Revoked token");

            // Try to extract user ID from revoked token for logging
            let user_id = match token::validate_token(token) {
                Ok(claims) => Some(claims.sub),
                Err(_) => None,
            };

            // Log token revoked
            let event = AuthEvent::new(AuthEventType::TokenValidation, user_id.as_deref(), false)
                .with_details("Token has been revoked")
                .with_resource(request_path)
                .with_duration(start_time.elapsed().as_millis() as u64)
                .with_auth_method("jwt");

            log_auth_event(event);

            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::empty())
                .unwrap_or_default()
        },
        Err(e) => {
            debug!("Standard JWT validation failed, trying Auth0 validation: {}", e);

            // If internal JWT validation fails, try Auth0 validation
            match auth0::validate_auth0_token(token).await {
                Ok(user_info) => {
                    debug!("Token validated successfully as Auth0 JWT for user: {}", user_info.user_id);

                    // Log successful Auth0 authentication
                    let duration = start_time.elapsed().as_millis() as u64;
                    let event = AuthEvent::new(AuthEventType::TokenValidation, Some(&user_info.user_id), true)
                        .with_details("Auth0 JWT validation successful")
                        .with_resource(request_path)
                        .with_duration(duration)
                        .with_auth_method("auth0");

                    log_auth_event(event);

                    // Create internal claims for compatibility
                    let claims = Claims {
                        sub: user_info.user_id.clone(),
                        iss: "auth0".to_string(),
                        iat: Utc::now().timestamp(),
                        exp: Utc::now().timestamp() + 3600, // Just a placeholder, the real expiration is in the token
                    };

                    // Add user info to request extensions
                    req.extensions_mut().insert(user_info);
                    req.extensions_mut().insert(claims);

                    // Continue with the request
                    next.run(req).await
                },
                Err(auth0_err) => {
                    error!("Auth0 token validation error: {}", auth0_err);

                    // Log failed Auth0 validation
                    let event = AuthEvent::new(AuthEventType::TokenValidation, None, false)
                        .with_details(format!("Auth0 token validation error: {}", auth0_err))
                        .with_resource(request_path)
                        .with_duration(start_time.elapsed().as_millis() as u64)
                        .with_auth_method("auth0");

                    log_auth_event(event);

                    Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .body(Body::empty())
                        .unwrap_or_default()
                }
            }
        }
    }
}

/// Configure authentication for the application
#[cfg(feature = "with-api")]
pub fn configure_auth(app: axum::Router) -> axum::Router {
    use tower_http::cors::{Any, CorsLayer};
    use tower_http::set_header::SetResponseHeaderLayer;
    use axum::http::header;

    // Create CORS layer for authentication endpoints
    let auth_cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
        .max_age(std::time::Duration::from_secs(3600));

    // Add security headers
    let security_headers = tower::ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::if_not_present(
            header::STRICT_TRANSPORT_SECURITY,
            header::HeaderValue::from_static("max-age=63072000; includeSubDomains; preload")
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            header::HeaderValue::from_static("nosniff")
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            header::HeaderValue::from_static("DENY")
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            header::HeaderValue::from_static(
                "default-src 'self'; script-src 'self'; connect-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; font-src 'self'; frame-ancestors 'none'; form-action 'self'; base-uri 'self'"
            )
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::HeaderName::from_static("permissions-policy"),
            header::HeaderValue::from_static("camera=(), microphone=(), geolocation=(), interest-cohort=()")
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::HeaderName::from_static("referrer-policy"),
            header::HeaderValue::from_static("strict-origin-when-cross-origin")
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::HeaderName::from_static("x-permitted-cross-domain-policies"),
            header::HeaderValue::from_static("none")
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::HeaderName::from_static("cross-origin-opener-policy"),
            header::HeaderValue::from_static("same-origin")
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::HeaderName::from_static("cross-origin-embedder-policy"),
            header::HeaderValue::from_static("require-corp")
        ));

    // Apply the security headers and CORS to the entire application
    app.layer(auth_cors).layer(security_headers)
}

/// Auth info endpoint
#[cfg(feature = "with-api")]
#[utoipa::path(
    get,
    path = "/auth/info",
    responses(
        (status = 200, description = "Authentication information", body = serde_json::Value)
    ),
    tag = "Authentication",
    security(
        ("jwt_auth" = [])
    )
)]
pub async fn auth_info(
    Extension(user_info): Extension<UserInfo>
) -> axum::Json<serde_json::Value> {
    use serde_json::json;
    axum::Json(json!({
        "message": "Authentication info",
        "user_id": user_info.user_id,
        "roles": user_info.roles,
        "status": "authenticated"
    }))
}

/// Refresh token endpoint
#[cfg(feature = "with-api")]
#[utoipa::path(
    post,
    path = "/auth/refresh",
    responses(
        (status = 200, description = "Token refreshed successfully", body = serde_json::Value),
        (status = 401, description = "Invalid refresh token", body = serde_json::Value)
    ),
    request_body(
        content = serde_json::Value,
        description = "No body required. Send the refresh token in the Authorization header as a Bearer token.",
        content_type = "application/json"
    ),
    tag = "Authentication"
)]
pub async fn refresh_token(
    headers: axum::http::HeaderMap,
) -> Result<axum::Json<serde_json::Value>, (StatusCode, axum::Json<serde_json::Value>)> {
    use serde_json::json;

    // Start timing the refresh operation
    let start_time = std::time::Instant::now();

    // Extract refresh token from header
    let auth_header = match headers.get(header::AUTHORIZATION) {
        Some(value) => match value.to_str() {
            Ok(auth_str) => auth_str,
            Err(_) => {
                // Log invalid header format
                let event = AuthEvent::new(AuthEventType::TokenRefresh, None, false)
                    .with_details("Invalid Authorization header format")
                    .with_duration(start_time.elapsed().as_millis() as u64)
                    .with_auth_method("refresh_token");

                log_auth_event(event);

                return Err((
                    StatusCode::UNAUTHORIZED,
                    axum::Json(json!({
                        "error": "invalid_request",
                        "error_description": "Invalid Authorization header format"
                    }))
                ));
            }
        },
        None => {
            // Log missing header
            let event = AuthEvent::new(AuthEventType::TokenRefresh, None, false)
                .with_details("Missing Authorization header")
                .with_duration(start_time.elapsed().as_millis() as u64)
                .with_auth_method("refresh_token");

            log_auth_event(event);

            return Err((
                StatusCode::UNAUTHORIZED,
                axum::Json(json!({
                    "error": "invalid_request",
                    "error_description": "Missing Authorization header"
                }))
            ));
        }
    };

    // Check if it's a Bearer token
    if !auth_header.starts_with("Bearer ") {
        // Log invalid token format
        let event = AuthEvent::new(AuthEventType::TokenRefresh, None, false)
            .with_details("Authorization header must start with Bearer")
            .with_duration(start_time.elapsed().as_millis() as u64)
            .with_auth_method("refresh_token");

        log_auth_event(event);

        return Err((
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({
                "error": "invalid_request",
                "error_description": "Authorization header must start with Bearer"
            }))
        ));
    }

    let refresh_token = &auth_header[7..]; // Skip "Bearer " prefix

    // Validate refresh token
    match token::validate_token(refresh_token) {
        Ok(claims) => {
            debug!("Refresh token valid for user: {}", claims.sub);

            // Generate a new access token
            match token::generate_token(&claims.sub, token::TokenType::Access, None) {
                Ok(new_token) => {
                    // Log successful token refresh
                    let _duration = start_time.elapsed().as_millis() as u64;
                    log_token_refresh(&claims.sub, true, None);

                    Ok(axum::Json(json!({
                        "access_token": new_token,
                        "token_type": "Bearer",
                        "expires_in": 900, // 15 minutes in seconds
                        "user_id": claims.sub
                    })))
                },
                Err(e) => {
                    error!("Failed to generate new access token: {}", e);

                    // Log token generation failure
                    let _duration = start_time.elapsed().as_millis() as u64;
                    log_token_refresh(&claims.sub, false, Some(&format!("Failed to generate new token: {}", e)));

                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(json!({
                            "error": "server_error",
                            "error_description": "Failed to generate new token"
                        }))
                    ))
                }
            }
        },
        Err(e) => {
            warn!("Invalid refresh token: {}", e);

            // Extract user ID from the token if possible for logging
            let user_id = match token::validate_token(refresh_token) {
                Ok(claims) => claims.sub,
                Err(_) => "unknown".to_string()
            };

            // Log token validation failure
            let duration = start_time.elapsed().as_millis() as u64;
            let event = AuthEvent::new(AuthEventType::TokenRefresh, Some(&user_id), false)
                .with_details(format!("Invalid or expired refresh token: {}", e))
                .with_duration(duration)
                .with_auth_method("refresh_token");

            log_auth_event(event);

            Err((
                StatusCode::UNAUTHORIZED,
                axum::Json(json!({
                    "error": "invalid_token",
                    "error_description": "Invalid or expired refresh token"
                }))
            ))
        }
    }
}

/// Logout endpoint
#[cfg(feature = "with-api")]
#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 200, description = "Logged out successfully", body = serde_json::Value),
        (status = 401, description = "Not authenticated", body = serde_json::Value)
    ),
    tag = "Authentication",
    security(
        ("jwt_auth" = [])
    )
)]
pub async fn logout(
    Extension(user_info): Extension<UserInfo>
) -> axum::Json<serde_json::Value> {
    use serde_json::json;

    // Revoke the user's token
    if let Err(e) = token::revoke_token(&user_info.user_id) {
        error!("Failed to revoke token: {}", e);
    }

    // Log logout event
    log_logout(&user_info.user_id);

    axum::Json(json!({
        "message": "Logged out successfully",
        "status": "success"
    }))
}

/// Login endpoint - authenticate user with username and password
#[cfg_attr(feature = "with-api", utoipa::path(
    post,
    path = "/auth/login",
    tag = "Authentication",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful. Use the returned access_token in the Authorization header as 'Bearer {token}' for authenticated requests.", body = LoginResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    ),
    operation_id = "login"
))]
pub async fn login(
    axum::Json(login_req): axum::Json<LoginRequest>
) -> Result<axum::Json<LoginResponse>, (StatusCode, axum::Json<serde_json::Value>)> {
    use serde_json::json;

    // Start timing for login
    let start_time = std::time::Instant::now();

    // For testing purposes, accept a hardcoded test user
    // In a real application, this would validate against a database
    if login_req.username == "testuser" && login_req.password == "testpassword" {
        // Generate a user ID (in a real app would come from the database)
        let user_id = "test-user-123".to_string();

        // Generate tokens
        let access_token = match token::generate_token(&user_id, token::TokenType::Access, Some(vec!["user".to_string()])) {
            Ok(token) => token,
            Err(e) => {
                error!("Failed to generate access token: {}", e);

                // Log token generation failure
                let event = AuthEvent::new(AuthEventType::Login, Some(&user_id), false)
                    .with_details(format!("Failed to generate token: {}", e))
                    .with_duration(start_time.elapsed().as_millis() as u64)
                    .with_auth_method("password");

                log_auth_event(event);

                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(json!({ "error": "Failed to generate token" }))
                ));
            }
        };

        let refresh_token = match token::generate_token(&user_id, token::TokenType::Refresh, Some(vec!["user".to_string()])) {
            Ok(token) => token,
            Err(e) => {
                error!("Failed to generate refresh token: {}", e);

                // Log refresh token generation failure
                let event = AuthEvent::new(AuthEventType::Login, Some(&user_id), false)
                    .with_details(format!("Failed to generate refresh token: {}", e))
                    .with_duration(start_time.elapsed().as_millis() as u64)
                    .with_auth_method("password");

                log_auth_event(event);

                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(json!({ "error": "Failed to generate token" }))
                ));
            }
        };

        // Create user info
        let user_info = UserInfo {
            user_id: user_id.clone(),
            roles: vec!["user".to_string()],
            email: Some(login_req.username.clone()),
            name: Some("Test User".to_string()),
            picture: None,
            auth_source: "password".to_string(),
        };

        // Return tokens and user info
        let response = LoginResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            user: user_info,
        };

        // Log successful login
        let event = AuthEvent::new(AuthEventType::Login, Some(&user_id), true)
            .with_details("Login successful")
            .with_duration(start_time.elapsed().as_millis() as u64)
            .with_auth_method("password");

        log_auth_event(event);

        Ok(axum::Json(response))
    } else {
        // Log failed login attempt
        let event = AuthEvent::new(AuthEventType::FailedLogin, Some(&login_req.username), false)
            .with_details("Invalid username or password")
            .with_duration(start_time.elapsed().as_millis() as u64)
            .with_auth_method("password");

        log_auth_event(event);

        // Invalid credentials
        Err((
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({ "error": "Invalid username or password" }))
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_middleware_exists() {
        // Simple test to verify the middleware function exists
        // Just check that the function can be referenced
        let _func = auth_middleware::<()>;
        assert!(true, "Function exists and can be referenced");
    }
}
