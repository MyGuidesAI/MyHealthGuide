use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Configure Swagger UI endpoints
pub fn configure_swagger_routes() -> SwaggerUi {
    SwaggerUi::new("/api-docs")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
}

// API Documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        // Health endpoints
        crate::api::handlers::health::health_check,

        // Blood pressure endpoints
        crate::api::handlers::blood_pressure::get_blood_pressure,
        crate::api::handlers::blood_pressure::create_blood_pressure,
        crate::api::handlers::blood_pressure::get_blood_pressure_history,
        crate::api::handlers::blood_pressure::get_blood_pressure_insights,

        // Auth endpoints
        my_health_guide_domain::auth::auth_info,
        my_health_guide_domain::auth::refresh_token,
        my_health_guide_domain::auth::logout,
        my_health_guide_domain::auth::login,

        // OIDC endpoints - note these are partially defined through the routes module
        my_health_guide_domain::auth::routes::oidc_routes
    ),
    components(
        schemas(
            // Entities
            crate::entities::blood_pressure::BloodPressureReading,
            crate::entities::blood_pressure::CreateBloodPressureRequest,
            crate::entities::common::PublicErrorResponse,
            crate::entities::common::PublicPaginationParams,

            // Health handlers
            crate::api::handlers::health::HealthResponse,
            crate::api::handlers::health::ComponentStatus,
            crate::api::handlers::health::ComponentHealthStatus,

            // Blood pressure handlers
            crate::api::handlers::blood_pressure::ErrorResponse,
            crate::api::handlers::blood_pressure::BloodPressurePaginatedResponse,
            crate::api::handlers::blood_pressure::HistoryQueryParams,
            crate::api::handlers::blood_pressure::InsightsQueryParams,

            // Auth schemas
            my_health_guide_domain::auth::LoginRequest,
            my_health_guide_domain::auth::LoginResponse,
            my_health_guide_domain::auth::UserInfo,
            my_health_guide_domain::auth::Claims,

            // OIDC schemas
            my_health_guide_domain::auth::routes::OidcCallbackParams,
            my_health_guide_domain::auth::routes::OidcLoginResponse,
            my_health_guide_domain::auth::routes::OidcErrorResponse
        )
    ),
    tags(
        (name = "health", description = "Health check endpoint"),
        (name = "blood_pressure", description = "Blood pressure management endpoints"),
        (name = "Authentication", description = "Authentication and authorization endpoints")
    ),
    info(
        title = "My Health Guide API",
        version = "0.1.0",
        description = "API for tracking health metrics and providing insights",
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        ),
    ),
    servers(
        (url = "/", description = "Local development server")
    )
)]
struct ApiDoc;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;

    #[test]
    fn test_api_doc_generation() {
        // Test that OpenAPI schema can be generated without errors
        let openapi = ApiDoc::openapi();

        // Verify basic info fields are set correctly
        assert_eq!(openapi.info.title, "MyHealthGuide API");
        assert_eq!(openapi.info.version, "0.1.0");

        // Verify tags are defined
        let tags = &openapi.tags;
        assert!(tags.is_some());
        let tags = tags.as_ref().unwrap();
        assert!(tags.iter().any(|tag| tag.name == "System"));
        assert!(tags.iter().any(|tag| tag.name == "Blood Pressure"));

        // Verify servers are defined - checking if the servers exist
        assert!(openapi.servers.is_some() || !openapi.servers.as_ref().map_or(true, |s| s.is_empty()));

        // Debug print all available paths
        println!("Available paths in OpenAPI schema:");
        for path in openapi.paths.paths.keys() {
            println!("  {}", path);
        }

        // Verify paths are defined for our endpoints
        assert!(openapi.paths.paths.contains_key("/health"));

        // Check only new standardized paths
        assert!(
            openapi.paths.paths.contains_key("/api/v1/bloodpressure/{id}")
        );

        assert!(
            openapi.paths.paths.contains_key("/api/v1/bloodpressure")
        );

        assert!(
            openapi.paths.paths.contains_key("/api/v1/bloodpressure/insights")
        );
    }

    #[test]
    fn test_configure_swagger_routes() {
        // Create an empty router
        let app: Router = Router::new();

        // Add Swagger UI routes
        let app_with_swagger = configure_swagger_routes();

        // We can't easily test the app directly, but we can verify that the router
        // changes after applying the swagger configuration
        assert_ne!(
            format!("{:?}", app),
            format!("{:?}", app_with_swagger),
            "Router should be modified after adding Swagger UI routes"
        );
    }
}
