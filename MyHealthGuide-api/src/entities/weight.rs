use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use validator::Validate;
use utoipa::ToSchema;

/// Public representation of a weight reading
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PublicWeightReading {
    /// Unique identifier for the reading
    pub id: Uuid,
    
    /// Weight in kilograms
    pub weight_kg: f32,
    
    /// Optional body fat percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_fat_percentage: Option<f32>,
    
    /// Optional muscle mass in kilograms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muscle_mass_kg: Option<f32>,
    
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

/// Request payload for creating a new weight reading
#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct PublicCreateWeightRequest {
    /// Weight in kilograms
    #[validate(range(min = 20.0, max = 500.0, message = "Weight must be between 20 and 500 kg"))]
    pub weight_kg: f32,
    
    /// Optional body fat percentage
    #[validate(range(min = 1.0, max = 70.0, message = "Body fat percentage must be between 1 and 70%"))]
    pub body_fat_percentage: Option<f32>,
    
    /// Optional muscle mass in kilograms
    #[validate(range(min = 10.0, max = 200.0, message = "Muscle mass must be between 10 and 200 kg"))]
    pub muscle_mass_kg: Option<f32>,
    
    /// Optional notes about the reading
    #[validate(length(max = 1000, message = "Notes cannot exceed 1000 characters"))]
    pub notes: Option<String>,
    
    /// When the reading was taken. Defaults to current time if not provided.
    pub timestamp: Option<DateTime<Utc>>,
}

/// Weight insights response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublicWeightInsights {
    /// Current weight in kilograms
    pub current_weight_kg: f32,
    
    /// Weight change over the last 30 days in kilograms
    pub change_30d_kg: f32,
    
    /// Weight change over the last 90 days in kilograms
    pub change_90d_kg: f32,
    
    /// The trend direction (gaining, losing, maintaining)
    pub trend: String,
    
    /// Body fat percentage if available
    pub body_fat_percentage: Option<f32>,
    
    /// Muscle mass in kilograms if available
    pub muscle_mass_kg: Option<f32>,
    
    /// BMI based on current weight (requires height to be stored in user profile)
    pub bmi: Option<f32>,
    
    /// BMI category (underweight, normal, overweight, obese)
    pub bmi_category: Option<String>,
    
    /// When the insights were generated
    pub generated_at: DateTime<Utc>,
} 