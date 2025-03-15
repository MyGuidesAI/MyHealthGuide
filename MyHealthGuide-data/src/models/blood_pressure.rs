use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Storage model for a blood pressure reading
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Input data for creating a new blood pressure reading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBloodPressureRequest {
    /// Systolic blood pressure (the higher number)
    pub systolic: u16,
    
    /// Diastolic blood pressure (the lower number)
    pub diastolic: u16,
    
    /// Optional pulse rate in beats per minute
    pub pulse: Option<u16>,
    
    /// Optional notes about the reading
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

/// Blood pressure reading insights and analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    
    /// Blood pressure category as a string
    pub category: String,
    
    /// Number of readings analyzed
    pub reading_count: usize,
    
    /// Analysis period in days
    pub period_days: u32,
    
    /// Timestamp of the analysis
    pub generated_at: DateTime<Utc>,
} 