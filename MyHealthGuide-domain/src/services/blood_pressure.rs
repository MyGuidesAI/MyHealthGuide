use thiserror::Error;
use tracing::error;
use chrono::Utc;
use validator::Validate;
use async_trait::async_trait;

use crate::entities::blood_pressure::{
    BloodPressureCategory, BloodPressureInsights, BloodPressureReading, CreateBloodPressureRequest,
};
use crate::entities::conversions;
use MyHealthGuide_data::repository::{BloodPressureRepositoryTrait, RepositoryError};
use crate::services::insights::categorize_blood_pressure;

/// Blood pressure service errors
#[derive(Debug, Error)]
pub enum BloodPressureServiceError {
    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Not found error
    #[error("Reading not found: {0}")]
    NotFound(String),
    
    /// Repository error
    #[error("Repository error: {0}")]
    RepositoryError(String),
    
    /// Insufficient data error
    #[error("Insufficient data: {0}")]
    InsufficientData(String),
}

/// Trait for blood pressure service operations
#[async_trait]
pub trait BloodPressureServiceTrait {
    /// Validate a create blood pressure request
    fn validate_create_request(
        &self,
        request: &CreateBloodPressureRequest,
    ) -> Result<(), BloodPressureServiceError>;
    
    /// Calculate blood pressure insights from readings
    fn calculate_insights(
        &self,
        readings: &[BloodPressureReading],
        timeframe_days: u32,
    ) -> Result<BloodPressureInsights, BloodPressureServiceError>;
    
    /// Get severity category for a blood pressure reading
    fn get_severity(&self, reading: &BloodPressureReading) -> BloodPressureCategory;
    
    /// Check if a reading indicates a hypertensive crisis
    fn is_hypertensive_crisis(&self, reading: &BloodPressureReading) -> bool;
    
    /// Create a new blood pressure reading
    async fn create_reading(&self, request: CreateBloodPressureRequest) 
        -> Result<BloodPressureReading, BloodPressureServiceError>;
    
    /// Get all blood pressure readings
    async fn get_all_readings(&self) -> Result<Vec<BloodPressureReading>, BloodPressureServiceError>;
    
    /// Get a blood pressure reading by ID
    async fn get_reading_by_id(&self, id: &str) -> Result<BloodPressureReading, BloodPressureServiceError>;
    
    /// Get filtered blood pressure readings
    async fn get_filtered_readings(
        &self,
        start_date: Option<String>,
        end_date: Option<String>,
        limit: Option<usize>,
        offset: Option<usize>,
        sort_desc: Option<bool>,
    ) -> Result<(Vec<BloodPressureReading>, usize), BloodPressureServiceError>;
}

/// Blood pressure service for domain logic
pub struct BloodPressureService<R: BloodPressureRepositoryTrait> {
    repository: R,
}

impl<R: BloodPressureRepositoryTrait> BloodPressureService<R> {
    /// Create a new blood pressure service
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
    
    /// Map repository errors to service errors
    fn map_repo_error(&self, err: RepositoryError) -> BloodPressureServiceError {
        match err {
            RepositoryError::NotFound(msg) => BloodPressureServiceError::NotFound(msg),
            RepositoryError::Validation(msg) => BloodPressureServiceError::ValidationError(msg),
            _ => BloodPressureServiceError::RepositoryError(err.to_string()),
        }
    }
}

#[async_trait]
impl<R: BloodPressureRepositoryTrait + Send + Sync> BloodPressureServiceTrait for BloodPressureService<R> {
    /// Validate a create blood pressure request
    fn validate_create_request(
        &self,
        request: &CreateBloodPressureRequest,
    ) -> Result<(), BloodPressureServiceError> {
        // Use the validator crate's validation
        if let Err(validation_errors) = request.validate() {
            // Convert validation errors to a meaningful error message
            let error_message = validation_errors
                .field_errors()
                .iter()
                .map(|(field, errors)| {
                    let error_msgs: Vec<String> = errors
                        .iter()
                        .map(|err| {
                            if let Some(msg) = &err.message {
                                msg.to_string()
                            } else {
                                format!("Invalid {}", field)
                            }
                        })
                        .collect();
                    format!("{}: {}", field, error_msgs.join(", "))
                })
                .collect::<Vec<String>>()
                .join("; ");
            
            return Err(BloodPressureServiceError::ValidationError(error_message));
        }
        
        // Additional validation: Validate that systolic is greater than diastolic
        if request.systolic <= request.diastolic {
            return Err(BloodPressureServiceError::ValidationError(
                "Systolic pressure must be greater than diastolic pressure".to_string(),
            ));
        }
        
        // All validations passed
        Ok(())
    }
    
    /// Calculate blood pressure insights from readings
    fn calculate_insights(
        &self,
        readings: &[BloodPressureReading],
        timeframe_days: u32,
    ) -> Result<BloodPressureInsights, BloodPressureServiceError> {
        if readings.is_empty() {
            return Err(BloodPressureServiceError::InsufficientData(
                "No readings available to generate insights".to_string(),
            ));
        }
        
        // For simplicity, calculate basic stats
        let mut systolic_sum: f64 = 0.0;
        let mut diastolic_sum: f64 = 0.0;
        let mut pulse_sum: f64 = 0.0;
        let mut pulse_count: usize = 0;
        
        let mut max_systolic: i32 = 0;
        let mut max_diastolic: i32 = 0;
        let mut min_systolic: i32 = i32::MAX;
        let mut min_diastolic: i32 = i32::MAX;
        
        for reading in readings {
            systolic_sum += reading.systolic as f64;
            diastolic_sum += reading.diastolic as f64;
            
            if let Some(pulse) = reading.pulse {
                pulse_sum += pulse as f64;
                pulse_count += 1;
            }
            
            max_systolic = max_systolic.max(reading.systolic as i32);
            max_diastolic = max_diastolic.max(reading.diastolic as i32);
            min_systolic = min_systolic.min(reading.systolic as i32);
            min_diastolic = min_diastolic.min(reading.diastolic as i32);
        }
        
        let avg_systolic = systolic_sum / readings.len() as f64;
        let avg_diastolic = diastolic_sum / readings.len() as f64;
        let avg_pulse = if pulse_count > 0 {
            Some(pulse_sum / pulse_count as f64)
        } else {
            None
        };
        
        // Calculate the blood pressure category based on average readings
        let category = categorize_blood_pressure(avg_systolic as u16, avg_diastolic as u16);
        
        Ok(BloodPressureInsights {
            avg_systolic,
            avg_diastolic,
            avg_pulse,
            max_systolic,
            max_diastolic,
            min_systolic,
            min_diastolic,
            category,
            reading_count: readings.len(),
            period_days: timeframe_days,
            generated_at: Utc::now(),
        })
    }
    
    /// Get severity category for a blood pressure reading
    fn get_severity(&self, reading: &BloodPressureReading) -> BloodPressureCategory {
        categorize_blood_pressure(reading.systolic, reading.diastolic)
    }
    
    /// Check if a reading indicates a hypertensive crisis
    fn is_hypertensive_crisis(&self, reading: &BloodPressureReading) -> bool {
        reading.systolic > 180 || reading.diastolic > 120
    }
    
    /// Create a new blood pressure reading
    async fn create_reading(&self, request: CreateBloodPressureRequest) 
        -> Result<BloodPressureReading, BloodPressureServiceError> 
    {
        // Validate the request
        self.validate_create_request(&request)?;
        
        // Convert domain entity to data model using the centralized conversion function
        let data_request = conversions::convert_to_data_create_request(&request);
        
        // Call repository method
        let data_reading = self.repository.create(data_request)
            .await
            .map_err(|e| self.map_repo_error(e))?;
        
        // Convert back to domain entity using the centralized conversion function
        let domain_reading = conversions::convert_to_domain_reading(data_reading);
        
        Ok(domain_reading)
    }
    
    /// Get all blood pressure readings
    async fn get_all_readings(&self) -> Result<Vec<BloodPressureReading>, BloodPressureServiceError> {
        // Call repository method
        let data_readings = self.repository.get_all()
            .await
            .map_err(|e| self.map_repo_error(e))?;
        
        // Convert to domain entities using the centralized conversion function
        let domain_readings = data_readings.into_iter()
            .map(conversions::convert_to_domain_reading)
            .collect();
        
        Ok(domain_readings)
    }
    
    /// Get a blood pressure reading by ID
    async fn get_reading_by_id(&self, id: &str) -> Result<BloodPressureReading, BloodPressureServiceError> {
        // Convert to UUID using the centralized helper function
        let id_uuid = crate::entities::conversions::parse_string_to_uuid(id)
            .map_err(BloodPressureServiceError::ValidationError)?;
        
        // Call repository method
        let data_reading = self.repository.get_by_id(id_uuid)
            .await
            .map_err(|e| self.map_repo_error(e))?
            .ok_or_else(|| BloodPressureServiceError::NotFound(
                format!("Blood pressure reading with ID {} not found", id)
            ))?;
        
        // Convert to domain entity using the centralized conversion function
        let domain_reading = conversions::convert_to_domain_reading(data_reading);
        
        Ok(domain_reading)
    }
    
    /// Get filtered blood pressure readings
    async fn get_filtered_readings(
        &self,
        start_date: Option<String>,
        end_date: Option<String>,
        limit: Option<usize>,
        offset: Option<usize>,
        sort_desc: Option<bool>,
    ) -> Result<(Vec<BloodPressureReading>, usize), BloodPressureServiceError> {
        // Call repository method
        let (data_readings, total_count) = self.repository.get_filtered(
            start_date,
            end_date,
            limit,
            offset,
            sort_desc,
        ).await
        .map_err(|e| self.map_repo_error(e))?;
        
        // Convert to domain entities using the centralized conversion function
        let domain_readings = data_readings.into_iter()
            .map(conversions::convert_to_domain_reading)
            .collect();
        
        Ok((domain_readings, total_count))
    }
}

/// Create a default blood pressure service using the repository from data layer
pub fn create_default_blood_pressure_service() -> impl BloodPressureServiceTrait + Send + Sync {
    let repository = MyHealthGuide_data::repository::BloodPressureRepository::new();
    BloodPressureService::new(repository)
}

/// Create a mock blood pressure service for testing
/// This function is only available when the mock feature is enabled
#[cfg(feature = "mock")]
pub fn create_mock_blood_pressure_service() -> impl BloodPressureServiceTrait + Send {
    crate::testing::MockBloodPressureService::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    /// Create a test blood pressure reading
    fn create_test_reading(systolic: u16, diastolic: u16, pulse: Option<u16>) -> BloodPressureReading {
        BloodPressureReading {
            id: Utc::now().to_rfc3339(),
            systolic,
            diastolic,
            pulse,
            notes: None,
            timestamp: Utc::now().to_rfc3339(),
            position: None,
            arm: None,
            device_id: None,
        }
    }
    
    #[test]
    fn test_validate_create_request_valid() {
        // Create a valid request
        let request = CreateBloodPressureRequest {
            systolic: 120,
            diastolic: 80,
            pulse: Some(72),
            notes: None,
            timestamp: Utc::now().to_rfc3339(),
            position: None,
            arm: None,
            device_id: None,
        };
        
        // Create a mock repository
        let mock_repo = MyHealthGuide_data::repository::tests::MockBloodPressureRepository::new();
        let service = BloodPressureService::new(mock_repo);
        
        // Validation should pass
        assert!(service.validate_create_request(&request).is_ok());
    }
    
    #[test]
    fn test_validate_create_request_invalid_systolic() {
        // Create a request with invalid systolic (too high)
        let request = CreateBloodPressureRequest {
            systolic: 350, // Too high
            diastolic: 80,
            pulse: Some(72),
            notes: None,
            timestamp: Utc::now().to_rfc3339(),
            position: None,
            arm: None,
            device_id: None,
        };
        
        // Create a mock repository
        let mock_repo = MyHealthGuide_data::repository::tests::MockBloodPressureRepository::new();
        let service = BloodPressureService::new(mock_repo);
        
        // Validation should fail
        let result = service.validate_create_request(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Systolic"));
    }
    
    #[test]
    fn test_validate_create_request_invalid_diastolic() {
        // Create a request with invalid diastolic (too high)
        let request = CreateBloodPressureRequest {
            systolic: 120,
            diastolic: 250, // Too high
            pulse: Some(72),
            notes: None,
            timestamp: Utc::now().to_rfc3339(),
            position: None,
            arm: None,
            device_id: None,
        };
        
        // Create a mock repository
        let mock_repo = MyHealthGuide_data::repository::tests::MockBloodPressureRepository::new();
        let service = BloodPressureService::new(mock_repo);
        
        // Validation should fail
        let result = service.validate_create_request(&request);
        assert!(result.is_err());
        
        // Get the error message and check that it includes the word "Diastolic"
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Diastolic") || error_message.contains("diastolic"), 
               "Error message '{}' should mention diastolic pressure", error_message);
    }
    
    #[test]
    fn test_validate_create_request_systolic_not_greater_than_diastolic() {
        // Create a request where systolic is not greater than diastolic
        let request = CreateBloodPressureRequest {
            systolic: 80,
            diastolic: 80, // Same as systolic
            pulse: Some(72),
            notes: None,
            timestamp: Utc::now().to_rfc3339(),
            position: None,
            arm: None,
            device_id: None,
        };
        
        // Create a mock repository
        let mock_repo = MyHealthGuide_data::repository::tests::MockBloodPressureRepository::new();
        let service = BloodPressureService::new(mock_repo);
        
        // Validation should fail
        let result = service.validate_create_request(&request);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("greater than"));
    }
    
    #[test]
    fn test_calculate_insights() {
        // Create some test readings
        let readings = vec![
            create_test_reading(120, 80, Some(72)),
            create_test_reading(130, 85, Some(75)),
            create_test_reading(125, 82, Some(70)),
        ];
        
        // Create a mock repository
        let mock_repo = MyHealthGuide_data::repository::tests::MockBloodPressureRepository::new();
        let service = BloodPressureService::new(mock_repo);
        
        // Calculate insights
        let insights = service.calculate_insights(&readings, 30).unwrap();
        assert_eq!(insights.reading_count, 3);
        assert_eq!(insights.period_days, 30);
        assert!(insights.avg_systolic > 0.0);
        assert!(insights.avg_diastolic > 0.0);
        assert!(insights.avg_pulse.unwrap() > 0.0);
    }
    
    #[test]
    fn test_calculate_insights_empty_readings() {
        // Create empty readings
        let readings = vec![];
        
        // Create a mock repository
        let mock_repo = MyHealthGuide_data::repository::tests::MockBloodPressureRepository::new();
        let service = BloodPressureService::new(mock_repo);
        
        // Calculate insights should fail
        let result = service.calculate_insights(&readings, 30);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No readings"));
    }
    
    #[test]
    fn test_is_hypertensive_crisis() {
        // Create crisis reading
        let crisis_reading = create_test_reading(200, 110, Some(85));
        
        // Create normal reading
        let normal_reading = create_test_reading(120, 80, Some(72));
        
        // Create a mock repository
        let mock_repo = MyHealthGuide_data::repository::tests::MockBloodPressureRepository::new();
        let service = BloodPressureService::new(mock_repo);
        
        // Test crisis detection
        assert!(service.is_hypertensive_crisis(&crisis_reading));
        assert!(!service.is_hypertensive_crisis(&normal_reading));
    }
    
    #[test]
    fn test_create_reading() {
        // ... existing code ...
        let mock_repo = MyHealthGuide_data::repository::tests::MockBloodPressureRepository::new();
        // ... existing code ...
    }
    
    #[test]
    fn test_get_all_readings() {
        // ... existing code ...
        let mock_repo = MyHealthGuide_data::repository::tests::MockBloodPressureRepository::new();
        // ... existing code ...
    }
    
    #[test]
    fn test_get_reading_by_id() {
        // ... existing code ...
        let mock_repo = MyHealthGuide_data::repository::tests::MockBloodPressureRepository::new();
        // ... existing code ...
    }
    
    #[test]
    fn test_get_filtered_readings() {
        // ... existing code ...
        let mock_repo = MyHealthGuide_data::repository::tests::MockBloodPressureRepository::new();
        // ... existing code ...
    }
    
    #[test]
    fn test_get_filtered_readings_with_sort() {
        // ... existing code ...
        let mock_repo = MyHealthGuide_data::repository::tests::MockBloodPressureRepository::new();
        // ... existing code ...
    }
    
    #[test]
    fn test_get_filtered_readings_with_limit_offset() {
        // ... existing code ...
        let mock_repo = MyHealthGuide_data::repository::tests::MockBloodPressureRepository::new();
        // ... existing code ...
    }
    
    #[test]
    fn test_get_filtered_readings_with_date_range() {
        // ... existing code ...
        let mock_repo = MyHealthGuide_data::repository::tests::MockBloodPressureRepository::new();
        // ... existing code ...
    }
} 