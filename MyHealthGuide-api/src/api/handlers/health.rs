use axum::{http::StatusCode, response::IntoResponse, Json, Extension};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use utoipa::ToSchema;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Once, Arc};
use once_cell::sync::OnceCell;
use std::collections::HashMap;
// Use the trait from domain layer
use MyHealthGuide_domain::health::{HealthServiceTrait, SystemStatus, ComponentStatus as DomainComponentStatus, HealthComponent as DomainHealthComponent, SystemHealth};
use MyHealthGuide_domain::health;
use async_trait::async_trait;

/// Enhanced health check response model with more system information
#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    /// Current service status ("ok", "degraded", or "error")
    pub status: String,
    /// Current application version from Cargo manifest
    pub version: String,
    /// Timestamp of when the response was generated
    pub timestamp: u64,
    /// Uptime of the service in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime: Option<u64>,
    /// Details about various components of the system
    pub components: ComponentStatus,
    /// Environment information
    pub environment: String,
}

/// Status of individual system components
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ComponentStatus {
    /// Database connection status
    pub database: ComponentHealthStatus,
    /// API status
    pub api: ComponentHealthStatus,
    /// Additional components (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional: Option<serde_json::Value>,
}

/// Health status for an individual component
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ComponentHealthStatus {
    /// Status of the component ("ok", "degraded", or "error")
    pub status: String,
    /// Optional message with more details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

// Track the time when the server started using a thread-safe OnceCell
static SERVER_START_TIME: OnceCell<u64> = OnceCell::new();
static INIT: Once = Once::new();

// Initialize the server start time
pub fn initialize_server_start_time() {
    INIT.call_once(|| {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let _ = SERVER_START_TIME.set(start_time);
    });
}

/// Health check endpoint to verify the API is running
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "API is healthy", body = HealthResponse),
        (status = 500, description = "API is not healthy", body = HealthResponse),
        (status = 503, description = "API is degraded", body = HealthResponse)
    ),
    tag = "health"
)]
#[instrument]
pub async fn health_check(
    Extension(health_service): Extension<Arc<dyn HealthServiceTrait + Send + Sync>>,
) -> Result<impl IntoResponse, axum::response::Response> {
    info!("Health check requested");
    
    // Get the current timestamp
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    
    // Calculate uptime if server start time is available
    let uptime = SERVER_START_TIME.get().map(|&start_time| now.saturating_sub(start_time));
    
    // Get system health from the service
    let system_health = health_service.get_system_health().await;
    
    // Map domain status to API status
    let overall_status = match system_health.status {
        SystemStatus::Healthy => "ok",
        SystemStatus::Degraded => "degraded",
        SystemStatus::Unhealthy => "error",
    };
    
    // Map domain components to API component status
    let mut component_statuses = ComponentStatus {
        database: ComponentHealthStatus {
            status: map_component_status(&system_health.components.get("database")
                .map(|c| c.status.clone())
                .unwrap_or(DomainComponentStatus::Healthy)),
            message: system_health.components.get("database")
                .and_then(|c| c.details.clone()),
        },
        api: ComponentHealthStatus {
            status: map_component_status(&system_health.components.get("api")
                .map(|c| c.status.clone())
                .unwrap_or(DomainComponentStatus::Healthy)),
            message: system_health.components.get("api")
                .and_then(|c| c.details.clone()),
        },
        additional: None,
    };
    
    // Add any additional components as a JSON object
    if system_health.components.len() > 2 {
        let additional_components: serde_json::Value = system_health.components.iter()
            .filter(|(name, _)| name != &"database" && name != &"api")
            .map(|(name, component)| {
                (name.clone(), serde_json::json!({
                    "status": map_component_status(&component.status),
                    "message": component.details,
                }))
            })
            .collect::<serde_json::Map<String, serde_json::Value>>()
            .into();
            
        component_statuses.additional = Some(additional_components);
    }
    
    // Build the response
    let response = HealthResponse {
        status: overall_status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: now,
        uptime,
        components: component_statuses,
        environment: std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
    };
    
    // Return appropriate status code based on overall status
    match overall_status {
        "ok" => Ok((StatusCode::OK, Json(response))),
        "degraded" => Ok((StatusCode::SERVICE_UNAVAILABLE, Json(response))),
        _ => Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(response))),
    }
}

/// Map domain component status to API status string
fn map_component_status(status: &DomainComponentStatus) -> String {
    match status {
        DomainComponentStatus::Healthy => "ok",
        DomainComponentStatus::Degraded => "degraded",
        DomainComponentStatus::Unhealthy => "error",
    }.to_string()
}

/// Implementation of the health service
#[derive(Debug)]
pub struct HealthService {
    // State can be added here if needed
}

impl Default for HealthService {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthService {
    /// Create a new health service
    pub fn new() -> Self {
        HealthService {}
    }
}

#[async_trait]
impl HealthServiceTrait for HealthService {
    async fn get_system_health(&self) -> SystemHealth {
        let mut components = HashMap::new();
        
        // Check database status
        let db_status = match self.check_database_status().await {
            Ok(true) => DomainComponentStatus::Healthy,
            Ok(false) => DomainComponentStatus::Degraded,
            Err(_) => DomainComponentStatus::Unhealthy,
        };
        
        // Add database component
        components.insert(
            "database".to_string(),
            DomainHealthComponent {
                status: db_status.clone(),
                details: match db_status {
                    DomainComponentStatus::Healthy => None,
                    DomainComponentStatus::Degraded => Some("Database is experiencing high latency".to_string()),
                    DomainComponentStatus::Unhealthy => Some("Database connection failed".to_string()),
                },
            },
        );
        
        // Add API component (always healthy in this implementation)
        components.insert(
            "api".to_string(),
            DomainHealthComponent {
                status: DomainComponentStatus::Healthy,
                details: None,
            },
        );
        
        // Determine overall system status based on component statuses
        let system_status = if components.values().any(|c| c.status == DomainComponentStatus::Unhealthy) {
            SystemStatus::Unhealthy
        } else if components.values().any(|c| c.status == DomainComponentStatus::Degraded) {
            SystemStatus::Degraded
        } else {
            SystemStatus::Healthy
        };
        
        SystemHealth {
            status: system_status,
            components,
        }
    }
    
    async fn check_database_status(&self) -> Result<bool, String> {
        // Replace database::check_database_health with health::check_database_status
        health::check_database_status().await
    }
}

/// Factory function to create a health service
pub fn create_health_service() -> Arc<dyn HealthServiceTrait + Send + Sync> {
    Arc::new(HealthService::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    
    #[tokio::test]
    async fn test_health_check_response() {
        // Initialize start time
        initialize_server_start_time();
        
        // Create a mock health service
        let health_service = Arc::new(create_mock_health_service()) as Arc<dyn HealthServiceTrait + Send + Sync>;
        
        // Call health check with the mock service
        let response = health_check(Extension(health_service)).await.unwrap();
        
        // Convert to response
        let response = response.into_response();
        
        // Extract status code
        let status = response.status();
        
        // Should be OK since we're using a mock service configured to be healthy
        assert_eq!(status, StatusCode::OK);
    }
} 