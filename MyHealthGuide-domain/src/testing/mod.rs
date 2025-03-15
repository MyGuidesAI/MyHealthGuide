// Testing utilities and mock implementations for the domain layer
// This module is only available when the "mock" feature is enabled

// Re-export useful test mocks from the data layer
pub use MyHealthGuide_data::repository::tests::MockBloodPressureRepository;

use crate::entities::blood_pressure::{BloodPressureReading, CreateBloodPressureRequest, BloodPressureInsights, BloodPressureCategory};
use crate::services::blood_pressure::{BloodPressureServiceTrait, BloodPressureServiceError};
use std::sync::RwLock;
use std::collections::HashMap;
use crate::health::{SystemHealth, SystemStatus, ComponentStatus, HealthComponent, HealthServiceTrait};
use async_trait::async_trait;

/// Mock implementation of the BloodPressureServiceTrait for testing
pub struct MockBloodPressureService {
    readings: RwLock<HashMap<String, BloodPressureReading>>,
    should_fail_validation: bool,
    should_fail_creation: bool,
}

impl Default for MockBloodPressureService {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBloodPressureService {
    /// Create a new mock blood pressure service
    pub fn new() -> Self {
        Self {
            readings: RwLock::new(HashMap::new()),
            should_fail_validation: false,
            should_fail_creation: false,
        }
    }
    
    /// Configure the mock to fail validation
    pub fn with_validation_failure(mut self) -> Self {
        self.should_fail_validation = true;
        self
    }
    
    /// Configure the mock to fail creation
    pub fn with_creation_failure(mut self) -> Self {
        self.should_fail_creation = true;
        self
    }
    
    /// Add a pre-defined reading to the mock
    pub fn with_reading(self, reading: BloodPressureReading) -> Self {
        {
            let mut readings = self.readings.write().unwrap();
            readings.insert(reading.id.clone(), reading);
        }
        self
    }
    
    /// Add multiple pre-defined readings to the mock
    pub fn with_readings(self, readings: Vec<BloodPressureReading>) -> Self {
        {
            let mut readings_map = self.readings.write().unwrap();
            for reading in readings {
                readings_map.insert(reading.id.clone(), reading);
            }
        }
        self
    }
}

#[async_trait]
impl BloodPressureServiceTrait for MockBloodPressureService {
    fn validate_create_request(
        &self,
        _request: &CreateBloodPressureRequest,
    ) -> Result<(), BloodPressureServiceError> {
        if self.should_fail_validation {
            Err(BloodPressureServiceError::ValidationError(
                "Validation failed - mock is configured to fail validation".to_string(),
            ))
        } else {
            Ok(())
        }
    }
    
    fn calculate_insights(
        &self,
        readings: &[BloodPressureReading],
        timeframe_days: u32,
    ) -> Result<crate::entities::blood_pressure::BloodPressureInsights, BloodPressureServiceError> {
        if readings.is_empty() {
            return Err(BloodPressureServiceError::InsufficientData(
                "No readings available to calculate insights".to_string(),
            ));
        }
        
        // Generate mock insights
        Ok(BloodPressureInsights {
            avg_systolic: 120.0,
            avg_diastolic: 80.0,
            avg_pulse: Some(70.0),
            max_systolic: 140,
            max_diastolic: 90,
            min_systolic: 110,
            min_diastolic: 70,
            category: BloodPressureCategory::Normal,
            reading_count: readings.len(),
            period_days: timeframe_days,
            generated_at: chrono::Utc::now(),
        })
    }
    
    fn get_severity(&self, reading: &BloodPressureReading) -> crate::entities::blood_pressure::BloodPressureCategory {
        if reading.systolic >= 180 || reading.diastolic >= 120 {
            BloodPressureCategory::HypertensiveCrisis
        } else if reading.systolic >= 140 || reading.diastolic >= 90 {
            BloodPressureCategory::Hypertension2
        } else if reading.systolic >= 130 || reading.diastolic >= 80 {
            BloodPressureCategory::Hypertension1
        } else if reading.systolic >= 120 && reading.diastolic < 80 {
            BloodPressureCategory::Elevated
        } else {
            BloodPressureCategory::Normal
        }
    }
    
    fn is_hypertensive_crisis(&self, reading: &BloodPressureReading) -> bool {
        reading.systolic >= 180 || reading.diastolic >= 120
    }
    
    async fn create_reading(&self, request: CreateBloodPressureRequest) 
        -> Result<BloodPressureReading, BloodPressureServiceError> 
    {
        // First validate the request
        self.validate_create_request(&request)?;
        
        if self.should_fail_creation {
            return Err(BloodPressureServiceError::RepositoryError(
                "Repository error - mock is configured to fail creation".to_string(),
            ));
        }
        
        // Generate a new reading
        let id = uuid::Uuid::new_v4().to_string();
        let reading = BloodPressureReading {
            id,
            systolic: request.systolic,
            diastolic: request.diastolic,
            pulse: request.pulse,
            timestamp: request.timestamp,
            notes: request.notes,
            position: request.position,
            arm: request.arm,
            device_id: request.device_id,
        };
        
        // Store the reading
        let mut readings = self.readings.write().unwrap();
        let id = reading.id.clone();
        readings.insert(id, reading.clone());
        
        Ok(reading)
    }
    
    async fn get_all_readings(&self) -> Result<Vec<BloodPressureReading>, BloodPressureServiceError> {
        let readings = self.readings.read().unwrap();
        let readings_vec: Vec<BloodPressureReading> = readings.values().cloned().collect();
        Ok(readings_vec)
    }
    
    async fn get_reading_by_id(&self, id: &str) -> Result<BloodPressureReading, BloodPressureServiceError> {
        let readings = self.readings.read().unwrap();
        
        match readings.get(id) {
            Some(reading) => Ok(reading.clone()),
            None => Err(BloodPressureServiceError::NotFound(
                format!("Reading with ID {} not found", id),
            )),
        }
    }
    
    async fn get_filtered_readings(
        &self,
        start_date: Option<String>,
        end_date: Option<String>,
        limit: Option<usize>,
        offset: Option<usize>,
        sort_desc: Option<bool>,
    ) -> Result<(Vec<BloodPressureReading>, usize), BloodPressureServiceError> {
        let readings = self.readings.read().unwrap();
        let mut readings_vec: Vec<BloodPressureReading> = readings.values().cloned().collect();
        
        // Filter by date range if provided
        if let Some(start) = &start_date {
            readings_vec.retain(|r| r.timestamp >= *start);
        }
        
        if let Some(end) = &end_date {
            readings_vec.retain(|r| r.timestamp <= *end);
        }
        
        // Sort by timestamp
        readings_vec.sort_by(|a, b| {
            if sort_desc.unwrap_or(false) {
                b.timestamp.cmp(&a.timestamp)
            } else {
                a.timestamp.cmp(&b.timestamp)
            }
        });
        
        // Get total count before pagination
        let total_count = readings_vec.len();
        
        // Apply pagination if provided
        if let Some(offset_val) = offset {
            if offset_val < readings_vec.len() {
                readings_vec = readings_vec.split_off(offset_val);
            } else {
                readings_vec = Vec::new();
            }
        }
        
        if let Some(limit_val) = limit {
            readings_vec.truncate(limit_val);
        }
        
        Ok((readings_vec, total_count))
    }
}

/// Mock implementation of health services for testing system health
#[derive(Debug)]
pub struct MockHealthService {
    /// Database component status
    database_status: ComponentStatus,
    /// System status
    system_status: SystemStatus,
    /// Additional components
    components: HashMap<String, HealthComponent>,
}

impl Default for MockHealthService {
    fn default() -> Self {
        Self::new()
    }
}

impl MockHealthService {
    /// Create a new mock health service with all components healthy
    pub fn new() -> Self {
        Self {
            database_status: ComponentStatus::Healthy,
            system_status: SystemStatus::Healthy,
            components: HashMap::new(),
        }
    }
    
    /// Configure the mock with a degraded database
    pub fn with_degraded_database(mut self) -> Self {
        self.database_status = ComponentStatus::Degraded;
        self
    }
    
    /// Configure the mock with an unhealthy database
    pub fn with_unhealthy_database(mut self) -> Self {
        self.database_status = ComponentStatus::Unhealthy;
        self
    }
    
    /// Set the overall system status
    pub fn with_system_status(mut self, status: SystemStatus) -> Self {
        self.system_status = status;
        self
    }
    
    /// Add a custom component with a specific status
    pub fn with_component(mut self, name: &str, status: ComponentStatus, details: Option<String>) -> Self {
        self.components.insert(
            name.to_string(), 
            HealthComponent { 
                status, 
                details 
            }
        );
        self
    }
}

#[async_trait]
impl HealthServiceTrait for MockHealthService {
    /// Get the system health
    async fn get_system_health(&self) -> SystemHealth {
        let mut components = HashMap::new();
        
        // Add database component
        components.insert(
            "database".to_string(),
            HealthComponent {
                status: self.database_status.clone(),
                details: match self.database_status {
                    ComponentStatus::Healthy => None,
                    ComponentStatus::Degraded => Some("Database is experiencing high load".to_string()),
                    ComponentStatus::Unhealthy => Some("Database connection failed".to_string()),
                },
            },
        );
        
        // Add API component
        components.insert(
            "api".to_string(),
            HealthComponent {
                status: ComponentStatus::Healthy,
                details: None,
            },
        );
        
        // Add any additional components
        for (name, component) in &self.components {
            components.insert(name.clone(), component.clone());
        }
        
        SystemHealth {
            status: self.system_status.clone(),
            components,
        }
    }
    
    /// Check database status
    async fn check_database_status(&self) -> Result<bool, String> {
        match self.database_status {
            ComponentStatus::Healthy => Ok(true),
            ComponentStatus::Degraded => Ok(true),
            ComponentStatus::Unhealthy => Err("Database connection failed".to_string()),
        }
    }
}

/// Factory function to create a mock blood pressure service
pub fn create_mock_blood_pressure_service() -> impl BloodPressureServiceTrait {
    MockBloodPressureService::new()
}

/// Factory function to create a mock health service
pub fn create_mock_health_service() -> impl HealthServiceTrait {
    MockHealthService::new()
} 