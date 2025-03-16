use axum::{
    middleware,
    routing::get,
    routing::post,
    Router,
    Extension,
};
use tracing::debug;
use std::sync::Arc;

use my_health_guide_domain::auth::{auth_middleware, configure_auth, oidc::OidcClient, routes::oidc_routes, authorize};
use crate::api::handlers::{health, blood_pressure};
use crate::openapi::configure_swagger_routes;

type AppState = blood_pressure::BloodPressureService;

/// Create the application router
pub async fn create_app() -> Router {
    debug!("Creating application router");

    // Create blood pressure service using factory function
    let blood_pressure_service = blood_pressure::create_service();

    // Create health service using factory function
    let health_service = health::create_health_service();

    // Initialize OIDC client
    #[cfg(not(test))]
    let oidc_client = {
        // Check if OIDC is enabled via environment variable (default to true for backward compatibility)
        let enable_oidc = std::env::var("ENABLE_OIDC")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(true);

        if enable_oidc {
            tracing::info!("OIDC authentication is enabled");
            let oidc_config = my_health_guide_domain::auth::oidc::OidcConfig::default();

            match OidcClient::new(oidc_config).await {
                Ok(client) => Arc::new(client),
                Err(err) => {
                    // Log error but don't crash the application
                    tracing::error!("Failed to initialize OIDC client: {:?}. OIDC auth will not be available.", err);
                    // Return a stub client that will return appropriate errors
                    Arc::new(OidcClient::stub())
                }
            }
        } else {
            tracing::info!("OIDC authentication is disabled via ENABLE_OIDC environment variable");
            Arc::new(OidcClient::stub())
        }
    };

    // In test mode, just use a stub client
    #[cfg(test)]
    let oidc_client = Arc::new(OidcClient::stub());

    // Set up API routes that require authentication
    let api_routes = Router::new()
        // Define specific routes before parametrized routes to avoid conflicts
        .route("/bloodpressure/insights", get(blood_pressure::get_blood_pressure_insights))
        .route("/bloodpressure", get(blood_pressure::get_blood_pressure_history)
                               .post(blood_pressure::create_blood_pressure))
        .route("/bloodpressure/:id", get(blood_pressure::get_blood_pressure))
        .layer(middleware::from_fn_with_state(
            blood_pressure_service.clone(),
            auth_middleware::<AppState>
        ));

    debug!("API routes configured");

    // Simple test handler
    async fn test_handler() -> axum::Json<serde_json::Value> {
        debug!("Test endpoint called");
        use serde_json::json;
        axum::Json(json!({ "status": "ok", "message": "Test route is working!" }))
    }

    // Admin handler that requires admin role
    async fn admin_handler() -> axum::Json<serde_json::Value> {
        debug!("Admin endpoint called");
        use serde_json::json;
        axum::Json(json!({
            "status": "ok",
            "message": "Admin access confirmed",
            "admin_data": {
                "total_users": 1250,
                "active_users": 830,
                "new_users_today": 15
            }
        }))
    }

    // Set up admin routes that require admin role
    let admin_routes = Router::new()
        .route("/admin", get(admin_handler))
        .route("/admin/users", get(|| async { "Admin users list" }))
        .route("/admin/settings", get(|| async { "Admin settings" }))
        .layer(middleware::from_fn_with_state(
            blood_pressure_service.clone(),
            authorize::require_role::<AppState>("admin")
        ))
        .layer(middleware::from_fn_with_state(
            blood_pressure_service.clone(),
            auth_middleware::<AppState>  // Authentication must happen before authorization
        ));

    debug!("Admin routes configured");

    // Set up public routes that don't require authentication
    let public_routes = Router::new()
        .route("/health", get(health::health_check))
        .route("/test", get(test_handler))
        .route("/auth/login", post(my_health_guide_domain::auth::login))
        .route("/auth/refresh", post(my_health_guide_domain::auth::refresh_token))
        .layer(Extension(health_service));

    debug!("Public routes configured");

    // Set up authentication routes
    let auth_routes = Router::new()
        .route("/auth/info", get(my_health_guide_domain::auth::auth_info))
        .route("/auth/logout", post(my_health_guide_domain::auth::logout))
        .layer(middleware::from_fn_with_state(
            blood_pressure_service.clone(),
            auth_middleware::<AppState>
        ))
        .nest("/auth/oidc", oidc_routes().with_state(oidc_client));

    debug!("Auth routes configured");

    // Combine all routes
    let app = Router::new()
        .merge(public_routes)
        .merge(auth_routes)
        .merge(admin_routes); // Add admin routes to the main router

    debug!("Base routes merged");

    let app = app.nest("/api/v1", api_routes)
        .with_state(blood_pressure_service);

    debug!("API routes nested");

    // Configure the Swagger UI using the helper function
    let app = add_swagger_ui(app);

    debug!("Swagger UI merged");

    // Apply security configuration
    let app = configure_auth(app);
    debug!("Security configuration applied");

    // Initialize health check service startup time
    health::initialize_server_start_time();
    debug!("Health check service initialized");

    app
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Create a test application
    pub async fn create_test_app() -> Router {
        super::create_app().await
    }
}

/// Add Swagger UI to the router
pub fn add_swagger_ui(app: Router) -> Router {
    // Get Swagger UI routes
    let swagger = configure_swagger_routes();

    // Merge Swagger UI with the app router
    app.merge(swagger)
}
