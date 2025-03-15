#[cfg(test)]
mod routes_tests {
    use crate::auth::oidc::OidcClient;
    use crate::auth::routes::oidc_routes;
    
    use std::sync::Arc;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::util::ServiceExt;
    use serde_json;
    use axum::body::to_bytes;
    

    // Define a constant for the body size limit
    const BODY_SIZE_LIMIT: usize = 1024 * 1024; // 1MB

    #[tokio::test]
    async fn test_oidc_test_endpoint() {
        // Create a router with the OIDC routes
        let client = Arc::new(OidcClient::stub());
        let app = oidc_routes().with_state(client);

        // Create a request to the test endpoint
        let request = Request::builder()
            .uri("/test")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        // Send the request to the router
        let response = app.oneshot(request).await.unwrap();

        // Check the response
        assert_eq!(response.status(), StatusCode::OK);

        // Get the response body with size limit
        let body = to_bytes(response.into_body(), BODY_SIZE_LIMIT).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        
        println!("Response body: {}", body_str);
        
        // Verify the response contains expected user info
        let user_info: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(user_info["user_id"].as_str().unwrap(), "test-user-123");
        assert_eq!(user_info["auth_source"].as_str().unwrap(), "oidc");
        assert!(user_info["email"].is_string());
        assert!(user_info["name"].is_string());
        assert!(user_info["picture"].is_string());
    }
    
    #[tokio::test]
    async fn test_oidc_login_endpoint() {
        // Create a router with the OIDC routes
        let client = Arc::new(OidcClient::stub());
        let app = oidc_routes().with_state(client);

        // Create a request to the login endpoint
        let request = Request::builder()
            .uri("/login")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        // Send the request to the router
        let response = app.oneshot(request).await.unwrap();

        // Check the response
        assert_eq!(response.status(), StatusCode::OK);

        // Get the response body with size limit
        let body = to_bytes(response.into_body(), BODY_SIZE_LIMIT).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        
        println!("Response body: {}", body_str);
        
        // Verify the response contains an auth_url
        let login_response: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert!(login_response["auth_url"].is_string());
        let auth_url = login_response["auth_url"].as_str().unwrap();
        assert!(!auth_url.is_empty());
    }
    
    #[tokio::test]
    async fn test_oidc_callback_success() {
        // Create a router with the OIDC routes
        let client = Arc::new(OidcClient::stub());
        let app = oidc_routes().with_state(client);

        // Create a request to the callback endpoint with valid code and state
        let request = Request::builder()
            .uri("/callback?code=test_code&state=test_state")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        // Send the request to the router
        let response = app.oneshot(request).await.unwrap();

        // Check the response
        assert_eq!(response.status(), StatusCode::OK);

        // Get the response body with size limit
        let body = to_bytes(response.into_body(), BODY_SIZE_LIMIT).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        
        println!("Response body: {}", body_str);
        
        // Verify the response contains user info
        let user_info: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(user_info["user_id"].as_str().unwrap(), "test-user-123");
        assert_eq!(user_info["auth_source"].as_str().unwrap(), "oidc");
    }
    
    #[tokio::test]
    async fn test_oidc_callback_error() {
        // Create a router with the OIDC routes
        let client = Arc::new(OidcClient::stub());
        let app = oidc_routes().with_state(client);

        // Create a request to the callback endpoint with error code
        let request = Request::builder()
            .uri("/callback?code=test_error_code&state=test_state")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        // Send the request to the router
        let response = app.oneshot(request).await.unwrap();

        // Check the response
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Get the response body with size limit
        let body = to_bytes(response.into_body(), BODY_SIZE_LIMIT).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        
        println!("Response body: {}", body_str);
        
        // Verify the response contains an error message
        let error_response: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert!(error_response["error"].is_string());
        let error_msg = error_response["error"].as_str().unwrap();
        assert!(error_msg.contains("Authentication failed"));
    }
} 