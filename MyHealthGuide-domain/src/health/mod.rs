//! Domain layer health check functionality
//! This module provides health check services for the application

use my_health_guide_data::database;
use std::collections::HashMap;
use async_trait::async_trait;

/// System health status
#[derive(Debug, Clone, PartialEq)]
pub enum SystemStatus {
    /// All components are healthy
    Healthy,
    /// Some components are degraded but the system is functional
    Degraded,
    /// System is not functioning properly
    Unhealthy,
}

/// Component health status
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentStatus {
    /// Component is functioning normally
    Healthy,
    /// Component is functioning but with reduced performance
    Degraded,
    /// Component is not functioning
    Unhealthy,
}

/// Represents a health component with status and optional details
#[derive(Debug, Clone)]
pub struct HealthComponent {
    /// Status of the component
    pub status: ComponentStatus,
    /// Optional details about the component status
    pub details: Option<String>,
}

/// Represents the overall health of the system
#[derive(Debug, Clone)]
pub struct SystemHealth {
    /// Overall system status
    pub status: SystemStatus,
    /// Map of component names to their health status
    pub components: HashMap<String, HealthComponent>,
}

/// Trait for health services
#[async_trait]
pub trait HealthServiceTrait: Send + Sync + std::fmt::Debug {
    /// Get the overall system health
    async fn get_system_health(&self) -> SystemHealth;

    /// Check the status of the database
    /// Returns true if the database is healthy, false if not
    /// Returns an error if the check could not be performed
    async fn check_database_status(&self) -> Result<bool, String>;
}

/// Check if the database is available and functioning properly
///
/// Returns:
/// - Ok(true) if the database is fully operational
/// - Ok(false) if the database has degraded functionality
/// - Err if the database is completely unavailable
pub async fn check_database_status() -> Result<bool, String> {
    // Since we're in the domain layer, we can access the data layer
    match database::get_connection_info() {
        Some(info) => {
            if info.contains("healthy") {
                Ok(true)
            } else {
                Ok(false)
            }
        },
        None => {
            // Try to get a connection from the pool
            match database::get_db_pool() {
                Ok(_) => Ok(true),
                Err(e) => Err(format!("Database connection error: {}", e)),
            }
        }
    }
}

/// Get overall system health
pub async fn get_system_health() -> SystemHealth {
    let db_status = check_database_status().await;

    let db_component = match db_status {
        Ok(true) => HealthComponent {
            status: ComponentStatus::Healthy,
            details: None,
        },
        Ok(false) => HealthComponent {
            status: ComponentStatus::Degraded,
            details: Some("Database is available but has performance issues".to_string()),
        },
        Err(e) => HealthComponent {
            status: ComponentStatus::Unhealthy,
            details: Some(e),
        },
    };

    let overall_status = if db_component.status == ComponentStatus::Unhealthy {
        SystemStatus::Unhealthy
    } else if db_component.status == ComponentStatus::Degraded {
        SystemStatus::Degraded
    } else {
        SystemStatus::Healthy
    };

    SystemHealth {
        status: overall_status,
        components: vec![
            ("database".to_string(), db_component),
        ].into_iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_system_health() {
        let health = get_system_health().await;
        // Don't assert specific status as it may depend on environment
        // Just check that components are present
        assert!(health.components.contains_key("database"));
    }
}
