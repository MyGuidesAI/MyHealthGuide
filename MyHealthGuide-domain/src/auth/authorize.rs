use axum::{
    extract::State,
    middleware::Next,
    response::{Response, IntoResponse},
    body::Body,
    http::{Request, StatusCode},
    Json,
};
use tracing::{debug, warn};
use serde_json::json;
use futures::future::BoxFuture;

use crate::auth::UserInfo;
use crate::auth::logging::{log_auth_event, AuthEvent, AuthEventType, log_access_denied};

/// Middleware for role-based access control
/// 
/// This middleware checks if the authenticated user has any of the required roles.
/// If the user lacks all required roles, they are denied access with a 403 Forbidden response.
pub async fn require_roles<S, I>(
    _state: State<S>,
    req: Request<Body>,
    next: Next,
    required_roles: I,
) -> Response 
where
    I: IntoIterator<Item = String>,
{
    // Convert required_roles into a Vec for easier processing and logging
    let required_roles: Vec<String> = required_roles.into_iter().collect();
    
    // Get the request path for logging
    let request_path = req.uri().path().to_string();
    
    // Extract user info from request extensions
    let user_info = req.extensions().get::<UserInfo>();
    
    match user_info {
        Some(user) => {
            // Check if user has any of the required roles
            let has_required_role = required_roles.iter()
                .any(|role| user.roles.contains(role));
                
            if has_required_role {
                debug!("User {} has required role for resource access: {}", user.user_id, request_path);
                
                // Log successful authorization
                let event = AuthEvent::new(AuthEventType::TokenValidation, Some(&user.user_id), true)
                    .with_details(format!("User authorized to access: {}", request_path))
                    .with_resource(request_path)
                    .with_auth_method("rbac");
                
                log_auth_event(event);
                
                // User has permission, continue with the request
                next.run(req).await
            } else {
                warn!("User {} lacks required roles: {:?} for resource: {}", 
                      user.user_id, required_roles, request_path);
                
                // Log the access denied event
                log_access_denied(&user.user_id, &request_path, &required_roles);
                
                // User does not have required role
                (
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "error": "forbidden",
                        "message": "You don't have the required permissions to access this resource",
                        "required_roles": required_roles
                    }))
                ).into_response()
            }
        },
        None => {
            // No user info in request extensions, this should never happen
            // as the auth_middleware should run before this middleware
            warn!("No user info found in request extensions for path: {}", request_path);
            
            // Log the error
            let event = AuthEvent::new(AuthEventType::AccessDenied, None, false)
                .with_details("Authentication context missing in request extensions")
                .with_resource(request_path.clone())
                .with_auth_method("rbac");
            
            log_auth_event(event);
            
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "internal_error",
                    "message": "Authentication context missing"
                }))
            ).into_response()
        }
    }
}

/// Middleware factory that requires a specific role for access
/// 
/// This is a convenience function that creates a middleware requiring a specific role.
/// 
/// # Example
/// ```
/// let admin_routes = Router::new()
///    .route("/admin", get(admin_handler))
///    .layer(middleware::from_fn_with_state(
///        app_state.clone(),
///        require_role("admin")
///    ));
/// ```
pub fn require_role<S: Clone + Send + Sync + 'static>(role: &str) -> impl Fn(State<S>, Request<Body>, Next) -> BoxFuture<'static, Response> + Clone + Send + 'static {
    let role = role.to_string();
    move |state, req, next| {
        let role_vec = vec![role.clone()];
        let fut = async move {
            require_roles(state, req, next, role_vec).await
        };
        Box::pin(fut)
    }
}

/// Middleware factory that requires any of the specified roles for access
/// 
/// This is a convenience function that creates a middleware requiring any of several roles.
/// 
/// # Example
/// ```
/// let privileged_routes = Router::new()
///   .route("/reports", get(reports_handler))
///   .layer(middleware::from_fn_with_state(
///       app_state.clone(),
///       require_any_role(&["admin", "manager", "analyst"])
///   ));
/// ```
pub fn require_any_role<S: Clone + Send + Sync + 'static>(roles: &[&str]) -> impl Fn(State<S>, Request<Body>, Next) -> BoxFuture<'static, Response> + Clone + Send + 'static {
    let roles: Vec<String> = roles.iter().map(|r| r.to_string()).collect();
    move |state, req, next| {
        let roles = roles.clone();
        let fut = async move {
            require_roles(state, req, next, roles).await
        };
        Box::pin(fut)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::body::to_bytes;
    use http_body_util::BodyExt;
    
    #[tokio::test]
    async fn test_require_roles_with_matching_role() {
        // Create a test request with user having "admin" role
        let user_info = UserInfo {
            user_id: "test-user".to_string(),
            roles: vec!["admin".to_string(), "user".to_string()],
            email: None,
            name: None,
            picture: None,
            auth_source: "test".to_string(),
        };
        
        let mut req = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
            
        req.extensions_mut().insert(user_info);
        
        // Create a next handler that returns a 200 OK response
        let next = Next::new(|req| async move {
            Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap()
        });
        
        // Call the middleware with "admin" role requirement
        let response = require_roles(
            State(()),
            req,
            next,
            vec!["admin".to_string()],
        ).await;
        
        // Check that the middleware allowed the request
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    #[tokio::test]
    async fn test_require_roles_with_no_matching_role() {
        // Create a test request with user having only "user" role
        let user_info = UserInfo {
            user_id: "test-user".to_string(),
            roles: vec!["user".to_string()],
            email: None,
            name: None,
            picture: None,
            auth_source: "test".to_string(),
        };
        
        let mut req = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
            
        req.extensions_mut().insert(user_info);
        
        // Create a next handler (which should not be called)
        let next = Next::new(|req| async move {
            Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap()
        });
        
        // Call the middleware with "admin" role requirement
        let response = require_roles(
            State(()),
            req,
            next,
            vec!["admin".to_string()],
        ).await;
        
        // Check that the middleware blocked the request with 403 Forbidden
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
} 