#[cfg(test)]
mod api_routes_tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use hyper::body::to_bytes;
    use tower::ServiceExt;
    use serde_json;
    use crate::api::create_app;

    #[tokio::test]
    async fn test_api_includes_oidc_routes() {
        // Create a router with all API routes
        let app = create_app().into_router();

        // Create a request to the OIDC login endpoint
        let request = Request::builder()
            .uri("/api/v1/auth/oidc/login")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        // Send the request to the router
        let response = app.oneshot(request).await.unwrap();
        
        println!("Response status: {:?}", response.status());

        // Check the response status - it should be 200 OK
        assert_eq!(response.status(), StatusCode::OK);

        // Get the response body
        let body = to_bytes(response.into_body()).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        
        println!("Response body: {}", body_str);
        
        // Verify the response contains an auth_url
        match serde_json::from_str::<serde_json::Value>(&body_str) {
            Ok(login_response) => {
                assert!(login_response.get("auth_url").is_some(), "Response does not contain auth_url field");
                assert!(login_response["auth_url"].is_string(), "auth_url is not a string");
                let auth_url = login_response["auth_url"].as_str().unwrap();
                assert!(!auth_url.is_empty(), "auth_url is empty");
            },
            Err(e) => {
                panic!("Failed to parse response as JSON: {}\nResponse body: {}", e, body_str);
            }
        }
    }
} 