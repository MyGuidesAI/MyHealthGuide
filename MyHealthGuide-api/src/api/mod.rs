pub mod handlers;
pub mod routes;

use axum::Router;

/// Create the application router
pub async fn create_application() -> Router {
    routes::create_app().await
} 