use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::{error, debug};
use std::sync::Arc;
use std::collections::HashMap;

#[cfg(feature = "with-api")]
use utoipa::ToSchema;

use crate::auth::oidc::OidcClient;
use crate::auth::logging::{log_auth_event, AuthEvent, AuthEventType};
use crate::auth::token;
use crate::auth::LoginResponse;
use crate::auth::UserInfo;

/// Query parameters for the OIDC callback endpoint
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "with-api", derive(ToSchema))]
pub struct OidcCallbackParams {
    /// Authorization code from OIDC provider
    pub code: String,
    /// State token for CSRF protection
    pub state: String,
}

/// Response for OIDC login endpoint
#[cfg(any(feature = "with-api", test))]
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-api", derive(ToSchema))]
pub struct OidcLoginResponse {
    pub auth_url: String,
}

/// OIDC authorization error response
#[cfg(any(feature = "with-api", test))]
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "with-api", derive(ToSchema))]
pub struct OidcErrorResponse {
    pub error: String,
}

/// Create a router with OIDC routes
#[cfg(any(feature = "with-api", test))]
#[cfg_attr(feature = "with-api", utoipa::path(
    get,
    path = "/auth/oidc/login",
    tag = "Authentication",
    responses(
        (status = 200, description = "Login URL generated successfully", body = OidcLoginResponse),
        (status = 500, description = "Failed to generate login URL", body = OidcErrorResponse)
    )
))]
pub fn oidc_routes() -> Router<Arc<OidcClient>> {
    Router::new()
        .route("/login", get(login_handler))
        .route("/callback", get(callback_handler))
        .route("/test", get(test_handler))
}

/// Handle login route - redirects to the OIDC provider
#[axum::debug_handler]
async fn login_handler(
    State(client): State<Arc<OidcClient>>,
) -> Response {
    // Debug sessions
    // client.debug_sessions();
    
    // Log the auth flow initiation
    let start_time = std::time::Instant::now();
    
    match client.start_auth_flow().await {
        Ok((auth_url, session)) => {
            debug!("Generated auth URL. Session ID: {}, CSRF token: {}, Nonce: {}", 
                   session.id, session.csrf_token, session.nonce);
            
            // Log successful auth flow start
            let duration = start_time.elapsed().as_millis() as u64;
            let event = AuthEvent::new(AuthEventType::Login, None, true)
                .with_details(format!("Started OIDC auth flow with session ID: {}", session.id))
                .with_duration(duration)
                .with_auth_method("oidc");
            
            log_auth_event(event);
            
            (StatusCode::OK, Json(OidcLoginResponse { auth_url })).into_response()
        }
        Err(e) => {
            error!("Failed to generate OIDC login URL: {:?}", e);
            
            // Log failed auth flow start
            let duration = start_time.elapsed().as_millis() as u64;
            let event = AuthEvent::new(AuthEventType::Login, None, false)
                .with_details(format!("Failed to start OIDC auth flow: {}", e))
                .with_duration(duration)
                .with_auth_method("oidc");
            
            log_auth_event(event);
            
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OidcErrorResponse { error: format!("Failed to generate login URL: {}", e) }),
            ).into_response()
        }
    }
}

/// Handle callback route from OIDC provider
#[axum::debug_handler]
async fn callback_handler(
    State(client): State<Arc<OidcClient>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    // Debug sessions
    // client.debug_sessions();
    
    debug!("Received OIDC callback with params: {:?}", params);
    
    // Start timing for the callback process
    let start_time = std::time::Instant::now();
    
    // Log the callback event
    let callback_event = AuthEvent::new(AuthEventType::OidcCallback, None, true)
        .with_details(format!("Received OIDC callback with state: {}", 
                             params.get("state").unwrap_or(&"none".to_string())));
    
    log_auth_event(callback_event);
    
    // Check for error parameters
    if let Some(error) = params.get("error") {
        let error_description = params.get("error_description")
            .map(|desc| format!("{}: {}", error, desc))
            .unwrap_or_else(|| error.clone());
        
        // Log the error
        let event = AuthEvent::new(AuthEventType::FailedLogin, None, false)
            .with_details(format!("OIDC error: {}", error_description))
            .with_duration(start_time.elapsed().as_millis() as u64)
            .with_auth_method("oidc");
        
        log_auth_event(event);
        
        return (
            StatusCode::BAD_REQUEST,
            Json(OidcErrorResponse { error: error_description }),
        ).into_response();
    }
    
    // Get required parameters
    let code = match params.get("code") {
        Some(code) => code,
        None => {
            // Log missing code parameter
            let event = AuthEvent::new(AuthEventType::FailedLogin, None, false)
                .with_details("Missing 'code' parameter in OIDC callback")
                .with_duration(start_time.elapsed().as_millis() as u64)
                .with_auth_method("oidc");
            
            log_auth_event(event);
            
            return (
                StatusCode::BAD_REQUEST,
                Json(OidcErrorResponse { error: "Missing 'code' parameter".to_string() }),
            ).into_response();
        }
    };
    
    let state = match params.get("state") {
        Some(state) => state,
        None => {
            // Log missing state parameter
            let event = AuthEvent::new(AuthEventType::FailedLogin, None, false)
                .with_details("Missing 'state' parameter in OIDC callback")
                .with_duration(start_time.elapsed().as_millis() as u64)
                .with_auth_method("oidc");
            
            log_auth_event(event);
            
            return (
                StatusCode::BAD_REQUEST, 
                Json(OidcErrorResponse { error: "Missing 'state' parameter".to_string() }),
            ).into_response();
        }
    };
    
    // Handle the callback
    match client.handle_callback(code, state).await {
        Ok(user_info) => {
            // Generate tokens
            let access_token = match token::generate_token(
                &user_info.user_id, 
                token::TokenType::Access, 
                Some(user_info.roles.clone())
            ) {
                Ok(token) => token,
                Err(e) => {
                    error!("Failed to generate access token: {}", e);
                    
                    // Log token generation failure
                    let duration = start_time.elapsed().as_millis() as u64;
                    let event = AuthEvent::new(AuthEventType::FailedLogin, Some(&user_info.user_id), false)
                        .with_details(format!("Failed to generate token: {}", e))
                        .with_duration(duration)
                        .with_auth_method("oidc");
                    
                    log_auth_event(event);
                    
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(OidcErrorResponse { error: "Failed to generate access token".to_string() }),
                    ).into_response();
                }
            };
            
            let refresh_token = match token::generate_token(
                &user_info.user_id, 
                token::TokenType::Refresh, 
                Some(user_info.roles.clone())
            ) {
                Ok(token) => token,
                Err(e) => {
                    error!("Failed to generate refresh token: {}", e);
                    
                    // Log refresh token generation failure
                    let duration = start_time.elapsed().as_millis() as u64;
                    let event = AuthEvent::new(AuthEventType::FailedLogin, Some(&user_info.user_id), false)
                        .with_details(format!("Failed to generate refresh token: {}", e))
                        .with_duration(duration)
                        .with_auth_method("oidc");
                    
                    log_auth_event(event);
                    
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(OidcErrorResponse { error: "Failed to generate refresh token".to_string() }),
                    ).into_response();
                }
            };
            
            // Create login response with tokens and user info
            let response = LoginResponse {
                access_token,
                refresh_token,
                token_type: "Bearer".to_string(),
                user: user_info.clone(),
            };
            
            // Log successful login
            let duration = start_time.elapsed().as_millis() as u64;
            let event = AuthEvent::new(AuthEventType::Login, Some(&user_info.user_id), true)
                .with_details("User successfully authenticated via OIDC".to_string())
                .with_duration(duration)
                .with_auth_method("oidc");
            
            log_auth_event(event);
            
            debug!("Generated tokens for OIDC user: {}", user_info.user_id);
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("OIDC callback error: {:?}", e);
            
            // Log failed login
            let duration = start_time.elapsed().as_millis() as u64;
            let event = AuthEvent::new(AuthEventType::FailedLogin, None, false)
                .with_details(format!("OIDC callback error: {}", e))
                .with_duration(duration)
                .with_auth_method("oidc");
            
            log_auth_event(event);
            
            (
                StatusCode::BAD_REQUEST,
                Json(OidcErrorResponse { error: format!("Authentication failed: {}", e) }),
            ).into_response()
        }
    }
}

/// Test handler for OIDC flow in test environments
#[cfg(any(test, feature = "mock"))]
async fn test_handler() -> impl IntoResponse {
    // Return test user info
    let user_info = UserInfo {
        user_id: "test-user-123".to_string(),
        roles: vec!["user".to_string()],
        email: Some("test@example.com".to_string()),
        name: Some("Test User".to_string()),
        picture: Some("https://example.com/avatar.png".to_string()),
        auth_source: "oidc".to_string(),
    };
    
    (StatusCode::OK, Json(user_info))
}

/// Test handler for OIDC flow in production
#[cfg(not(any(test, feature = "mock")))]
async fn test_handler() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(OidcErrorResponse { error: "Test endpoint only available in test environment".to_string() }),
    )
} 