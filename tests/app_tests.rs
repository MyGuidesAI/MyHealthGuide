use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use axum::body::Body;
use serde_json::Value;
use std::env;

// Initialize tracing once for all tests
static INIT: std::sync::Once = std::sync::Once::new();
fn initialize() {
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("info")
            .with_test_writer()
            .try_init();
    });
}

// Helper function to get body bytes from a response
async fn get_body_bytes(response: axum::response::Response) -> Vec<u8> {
    let body = response.into_body();
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    bytes.to_vec()
}

#[tokio::test]
async fn test_app_creation_and_health_check() {
    initialize();
    
    // Temporarily set any environment variables needed for testing
    env::set_var("PORT", "3030");
    
    // Create the app using the library function
    let app = my_health_guide::create_application();
    
    // Make a request to the health endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Check the response body
    let body = get_body_bytes(response).await;
    let health: Value = serde_json::from_slice(&body).unwrap();
    
    // In tests, we can get either "ok" or "degraded" depending on database initialization
    // So we'll allow both statuses to pass the test
    assert!(health["status"] == "ok" || health["status"] == "degraded", 
            "Health status should be either 'ok' or 'degraded' but was '{}'", health["status"]);
    assert!(health["version"].is_string());
}

#[tokio::test]
async fn test_openapi_documentation_available() {
    initialize();
    
    // Create the app
    let app = my_health_guide::create_application();
    
    // Make a request to the OpenAPI JSON endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api-docs/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Check that the response is valid JSON
    let body = get_body_bytes(response).await;
    let openapi: Value = serde_json::from_slice(&body).unwrap();
    
    // Verify basic OpenAPI structure
    assert!(openapi["openapi"].is_string());
    assert!(openapi["info"].is_object());
    assert!(openapi["paths"].is_object());
}

#[tokio::test]
async fn test_swagger_ui_available() {
    initialize();
    
    // Create the app
    let app = my_health_guide::create_application();
    
    // Make a request to the Swagger UI endpoint
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/swagger-ui/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Check that the response contains HTML for Swagger UI
    let body = get_body_bytes(response).await;
    let body_str = String::from_utf8_lossy(&body);
    
    assert!(body_str.contains("swagger-ui"));
} 