#[cfg(test)]
mod health_tests {
    use MyHealthGuide_domain::health::{SystemStatus, ComponentStatus, HealthServiceTrait, SystemHealth, HealthComponent};
    use std::sync::Arc;
    use std::collections::HashMap;
    use async_trait::async_trait;
    
    // Direct implementation of a mock health service for testing
    #[derive(Debug)]
    struct TestMockHealthService {
        system_status: SystemStatus,
        database_status: ComponentStatus,
        components: HashMap<String, HealthComponent>,
    }
    
    impl TestMockHealthService {
        fn new() -> Self {
            let mut components = HashMap::new();
            components.insert(
                "database".to_string(),
                HealthComponent {
                    status: ComponentStatus::Healthy,
                    details: None,
                },
            );
            components.insert(
                "api".to_string(),
                HealthComponent {
                    status: ComponentStatus::Healthy,
                    details: None,
                },
            );
            
            Self {
                system_status: SystemStatus::Healthy,
                database_status: ComponentStatus::Healthy,
                components,
            }
        }
        
        fn with_degraded_database(mut self) -> Self {
            self.database_status = ComponentStatus::Degraded;
            self.components.insert(
                "database".to_string(),
                HealthComponent {
                    status: ComponentStatus::Degraded,
                    details: Some("Database is experiencing high latency".to_string()),
                },
            );
            self
        }
        
        fn with_unhealthy_database(mut self) -> Self {
            self.database_status = ComponentStatus::Unhealthy;
            self.components.insert(
                "database".to_string(),
                HealthComponent {
                    status: ComponentStatus::Unhealthy,
                    details: Some("Database connection failed".to_string()),
                },
            );
            self
        }
        
        fn with_system_status(mut self, status: SystemStatus) -> Self {
            self.system_status = status;
            self
        }
        
        fn with_component(mut self, name: &str, status: ComponentStatus, details: Option<String>) -> Self {
            self.components.insert(
                name.to_string(),
                HealthComponent {
                    status,
                    details,
                },
            );
            self
        }
    }
    
    #[async_trait]
    impl HealthServiceTrait for TestMockHealthService {
        async fn get_system_health(&self) -> SystemHealth {
            SystemHealth {
                status: self.system_status.clone(),
                components: self.components.clone(),
            }
        }
        
        async fn check_database_status(&self) -> Result<bool, String> {
            match self.database_status {
                ComponentStatus::Healthy => Ok(true),
                ComponentStatus::Degraded => Ok(true), // Degraded but still functional
                ComponentStatus::Unhealthy => Err("Database connection failed".to_string()),
            }
        }
    }
    
    #[tokio::test]
    async fn test_mock_health_service_healthy() {
        // Create a mock health service with default settings (everything healthy)
        let mock_service = Arc::new(TestMockHealthService::new());
        
        // Verify we can use it as a trait object
        let service: Arc<dyn HealthServiceTrait + Send + Sync> = mock_service.clone();
        
        // Get the system health
        let health = service.get_system_health().await;
        
        // Verify the system status is healthy
        assert_eq!(health.status, SystemStatus::Healthy);
        
        // Verify the database component is healthy
        let db_component = health.components.get("database").expect("Database component should exist");
        assert_eq!(db_component.status, ComponentStatus::Healthy);
        assert!(db_component.details.is_none());
        
        // Check database status
        let db_status = service.check_database_status().await;
        assert!(db_status.is_ok());
        assert!(db_status.unwrap());
    }
    
    #[tokio::test]
    async fn test_mock_health_service_degraded() {
        // Create a mock health service with a degraded database
        let mock_service = Arc::new(
            TestMockHealthService::new()
                .with_degraded_database()
                .with_system_status(SystemStatus::Degraded)
        );
        
        // Verify we can use it as a trait object
        let service: Arc<dyn HealthServiceTrait + Send + Sync> = mock_service.clone();
        
        // Get the system health
        let health = service.get_system_health().await;
        
        // Verify the system status is degraded
        assert_eq!(health.status, SystemStatus::Degraded);
        
        // Verify the database component is degraded
        let db_component = health.components.get("database").expect("Database component should exist");
        assert_eq!(db_component.status, ComponentStatus::Degraded);
        assert!(db_component.details.is_some());
        assert_eq!(db_component.details.as_ref().unwrap(), "Database is experiencing high latency");
        
        // Check database status - should still return Ok(true) since degraded is still functional
        let db_status = service.check_database_status().await;
        assert!(db_status.is_ok());
        assert!(db_status.unwrap());
    }
    
    #[tokio::test]
    async fn test_mock_health_service_unhealthy() {
        // Create a mock health service with an unhealthy database
        let mock_service = Arc::new(
            TestMockHealthService::new()
                .with_unhealthy_database()
                .with_system_status(SystemStatus::Unhealthy)
        );
        
        // Verify we can use it as a trait object
        let service: Arc<dyn HealthServiceTrait + Send + Sync> = mock_service.clone();
        
        // Get the system health
        let health = service.get_system_health().await;
        
        // Verify the system status is unhealthy
        assert_eq!(health.status, SystemStatus::Unhealthy);
        
        // Verify the database component is unhealthy
        let db_component = health.components.get("database").expect("Database component should exist");
        assert_eq!(db_component.status, ComponentStatus::Unhealthy);
        assert!(db_component.details.is_some());
        assert_eq!(db_component.details.as_ref().unwrap(), "Database connection failed");
        
        // Check database status - should return Err
        let db_status = service.check_database_status().await;
        assert!(db_status.is_err());
        assert_eq!(db_status.unwrap_err(), "Database connection failed");
    }
    
    #[tokio::test]
    async fn test_mock_health_service_custom_components() {
        // Create a mock health service with a custom component
        let mock_service = Arc::new(
            TestMockHealthService::new()
                .with_component("cache", ComponentStatus::Degraded, Some("Cache hit rate is low".to_string()))
                .with_component("notification", ComponentStatus::Healthy, None)
        );
        
        // Verify we can use it as a trait object
        let service: Arc<dyn HealthServiceTrait + Send + Sync> = mock_service.clone();
        
        // Get the system health
        let health = service.get_system_health().await;
        
        // Verify the custom components exist
        let cache_component = health.components.get("cache").expect("Cache component should exist");
        assert_eq!(cache_component.status, ComponentStatus::Degraded);
        assert_eq!(cache_component.details.as_ref().unwrap(), "Cache hit rate is low");
        
        let notification_component = health.components.get("notification").expect("Notification component should exist");
        assert_eq!(notification_component.status, ComponentStatus::Healthy);
        assert!(notification_component.details.is_none());
        
        // Verify the standard components still exist
        assert!(health.components.contains_key("database"));
        assert!(health.components.contains_key("api"));
    }
} 