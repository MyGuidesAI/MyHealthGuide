use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use validator::Validate;

#[cfg(feature = "with-api")]
use utoipa::ToSchema;

/// Domain model for a blood pressure reading
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-api", derive(ToSchema))]
pub struct BloodPressureReading {
    /// Unique identifier for the reading
    pub id: String,
    
    /// Systolic blood pressure (the higher number)
    pub systolic: u16,
    
    /// Diastolic blood pressure (the lower number)
    pub diastolic: u16,
    
    /// Optional pulse rate in beats per minute
    pub pulse: Option<u16>,
    
    /// Optional notes about the reading
    pub notes: Option<String>,
    
    /// When the reading was taken
    pub timestamp: String,
    
    /// Optional position (e.g., sitting, standing)
    pub position: Option<String>,
    
    /// Optional arm used (left or right)
    pub arm: Option<String>,
    
    /// Optional device ID used for measurement
    pub device_id: Option<String>,
}

/// Request payload for creating a new blood pressure reading
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "with-api", derive(ToSchema))]
pub struct CreateBloodPressureRequest {
    /// Systolic blood pressure (the higher number)
    #[validate(range(min = 40, max = 300, message = "Systolic must be between 40 and 300"))]
    pub systolic: u16,
    
    /// Diastolic blood pressure (the lower number)
    #[validate(range(min = 20, max = 200, message = "Diastolic must be between 20 and 200"))]
    pub diastolic: u16,
    
    /// Optional pulse rate in beats per minute
    #[validate(range(min = 20, max = 250, message = "Pulse must be between 20 and 250"))]
    pub pulse: Option<u16>,
    
    /// Optional notes about the reading
    #[validate(length(max = 1000, message = "Notes cannot exceed 1000 characters"))]
    pub notes: Option<String>,
    
    /// When the reading was taken. Defaults to current time if not provided.
    pub timestamp: String,
    
    /// Optional position during measurement (e.g., sitting, standing)
    pub position: Option<String>,
    
    /// Optional arm used for measurement (left or right)
    pub arm: Option<String>,
    
    /// Optional device ID used for measurement
    pub device_id: Option<String>,
}

/// Blood pressure category based on measurements
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "with-api", derive(ToSchema))]
pub enum BloodPressureCategory {
    /// Normal blood pressure (systolic < 120 and diastolic < 80)
    Normal,
    
    /// Elevated blood pressure (systolic 120-129 and diastolic < 80)
    Elevated,
    
    /// Stage 1 Hypertension (systolic 130-139 or diastolic 80-89)
    Hypertension1,
    
    /// Stage 2 Hypertension (systolic ≥ 140 or diastolic ≥ 90)
    Hypertension2,
    
    /// Hypertensive crisis (systolic > 180 and/or diastolic > 120)
    HypertensiveCrisis,
}

impl ToString for BloodPressureCategory {
    fn to_string(&self) -> String {
        match self {
            BloodPressureCategory::Normal => "Normal".to_string(),
            BloodPressureCategory::Elevated => "Elevated".to_string(),
            BloodPressureCategory::Hypertension1 => "Hypertension Stage 1".to_string(),
            BloodPressureCategory::Hypertension2 => "Hypertension Stage 2".to_string(),
            BloodPressureCategory::HypertensiveCrisis => "Hypertensive Crisis".to_string(),
        }
    }
}

/// Blood pressure reading insights and analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "with-api", derive(ToSchema))]
pub struct BloodPressureInsights {
    /// Average systolic reading over the analysis period
    pub avg_systolic: f64,
    
    /// Average diastolic reading over the analysis period
    pub avg_diastolic: f64,
    
    /// Average pulse rate over the analysis period (if available)
    pub avg_pulse: Option<f64>,
    
    /// Highest recorded systolic reading during the period
    pub max_systolic: i32,
    
    /// Highest recorded diastolic reading during the period
    pub max_diastolic: i32,
    
    /// Lowest recorded systolic reading during the period
    pub min_systolic: i32,
    
    /// Lowest recorded diastolic reading during the period
    pub min_diastolic: i32,
    
    /// Blood pressure category based on average readings
    pub category: BloodPressureCategory,
    
    /// Number of readings analyzed
    pub reading_count: usize,
    
    /// Analysis period in days
    pub period_days: u32,
    
    /// Timestamp of the analysis
    pub generated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    // Current tests...

    /// Test timestamp validation in CreateBloodPressureRequest
    #[test]
    fn test_timestamp_validation() {
        // Create a base valid request
        let base_request = CreateBloodPressureRequest {
            systolic: 120,
            diastolic: 80,
            pulse: Some(72),
            timestamp: Utc::now().to_rfc3339(),
            notes: None,
            position: None,
            arm: None,
            device_id: None,
        };
        
        // Valid current timestamp should be accepted
        assert!(base_request.validate().is_ok());
        
        // Invalid timestamp format should be rejected
        let invalid_format = CreateBloodPressureRequest {
            timestamp: "2023-05-01 12:30:00".to_string(),
            ..base_request.clone()
        };
        let result = invalid_format.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid timestamp format"));
        
        // Future timestamp should be rejected
        let future_time = Utc::now() + Duration::days(1);
        let future_timestamp = CreateBloodPressureRequest {
            timestamp: future_time.to_rfc3339(),
            ..base_request.clone()
        };
        let result = future_timestamp.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("future"));
    }
} 