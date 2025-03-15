use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use validator::Validate;
use utoipa::ToSchema;

/// Public representation of a blood pressure reading
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BloodPressureReading {
    /// Unique identifier for the reading
    pub id: Uuid,
    
    /// Systolic blood pressure (the higher number)
    pub systolic: i32,
    
    /// Diastolic blood pressure (the lower number)
    pub diastolic: i32,
    
    /// Optional pulse rate in beats per minute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pulse: Option<i32>,
    
    /// Optional notes about the reading
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    
    /// When the reading was taken
    pub recorded_at: DateTime<Utc>,
    
    /// When the reading was created in the system
    pub created_at: DateTime<Utc>,
    
    /// When the reading was last updated
    pub updated_at: DateTime<Utc>,
}

/// Request payload for creating a new blood pressure reading
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateBloodPressureRequest {
    /// Systolic blood pressure (the higher number)
    #[validate(range(min = 40, max = 300, message = "Systolic must be between 40 and 300"))]
    pub systolic: i32,
    
    /// Diastolic blood pressure (the lower number)
    #[validate(range(min = 20, max = 200, message = "Diastolic must be between 20 and 200"))]
    pub diastolic: i32,
    
    /// Optional pulse rate in beats per minute
    #[validate(range(min = 20, max = 250, message = "Pulse must be between 20 and 250"))]
    pub pulse: Option<i32>,
    
    /// Optional notes about the reading
    #[validate(length(max = 1000, message = "Notes cannot exceed 1000 characters"))]
    pub notes: Option<String>,
    
    /// When the reading was taken. Defaults to current time if not provided.
    pub timestamp: Option<DateTime<Utc>>,
}

/// Request payload for updating an existing blood pressure reading
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateBloodPressureRequest {
    /// Systolic blood pressure (the higher number)
    #[validate(range(min = 40, max = 300, message = "Systolic must be between 40 and 300"))]
    pub systolic: Option<i32>,
    
    /// Diastolic blood pressure (the lower number)
    #[validate(range(min = 20, max = 200, message = "Diastolic must be between 20 and 200"))]
    pub diastolic: Option<i32>,
    
    /// Pulse rate in beats per minute
    #[validate(range(min = 20, max = 250, message = "Pulse must be between 20 and 250"))]
    pub pulse: Option<i32>,
    
    /// Notes about the reading
    #[validate(length(max = 1000, message = "Notes cannot exceed 1000 characters"))]
    pub notes: Option<String>,
    
    /// When the reading was taken
    pub timestamp: Option<DateTime<Utc>>,
} 