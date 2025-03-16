use crate::entities::blood_pressure::{
    BloodPressureReading, CreateBloodPressureRequest, BloodPressureInsights, BloodPressureCategory
};
use uuid::Uuid;

/// Conversion functions between domain entities and data models
/// These functions follow the pattern convert_to_[target_layer]_[model_name]
/// as specified in the architectural rules

/// Helper function to safely parse a string ID to UUID
///
/// This centralizes UUID parsing logic to ensure consistent handling across the application.
/// When an invalid UUID is provided, it returns a descriptive error message.
///
/// # Arguments
/// * `id` - The string ID to parse into a UUID
///
/// # Returns
/// * `Result<Uuid, String>` - The parsed UUID or an error message
pub fn parse_string_to_uuid(id: &str) -> Result<Uuid, String> {
    Uuid::parse_str(id).map_err(|_| format!("Invalid UUID format: {}", id))
}

/// Convert from data model to domain entity for blood pressure reading
pub fn convert_to_domain_reading(data_reading: my_health_guide_data::models::blood_pressure::BloodPressureReading)
    -> BloodPressureReading
{
    BloodPressureReading {
        id: data_reading.id,
        systolic: data_reading.systolic,
        diastolic: data_reading.diastolic,
        pulse: data_reading.pulse,
        notes: data_reading.notes,
        timestamp: data_reading.timestamp,
        position: data_reading.position,
        arm: data_reading.arm,
        device_id: data_reading.device_id,
    }
}

/// Convert from domain entity to data model for create request
pub fn convert_to_data_create_request(domain_request: &CreateBloodPressureRequest)
    -> my_health_guide_data::models::blood_pressure::CreateBloodPressureRequest
{
    my_health_guide_data::models::blood_pressure::CreateBloodPressureRequest {
        systolic: domain_request.systolic,
        diastolic: domain_request.diastolic,
        pulse: domain_request.pulse,
        notes: domain_request.notes.clone(),
        timestamp: domain_request.timestamp.clone(),
        position: domain_request.position.clone(),
        arm: domain_request.arm.clone(),
        device_id: domain_request.device_id.clone(),
    }
}

/// Convert from domain entity to data model for blood pressure insights
pub fn convert_to_data_insights(domain_insights: &BloodPressureInsights)
    -> my_health_guide_data::models::blood_pressure::BloodPressureInsights
{
    my_health_guide_data::models::blood_pressure::BloodPressureInsights {
        avg_systolic: domain_insights.avg_systolic,
        avg_diastolic: domain_insights.avg_diastolic,
        avg_pulse: domain_insights.avg_pulse,
        max_systolic: domain_insights.max_systolic,
        max_diastolic: domain_insights.max_diastolic,
        min_systolic: domain_insights.min_systolic,
        min_diastolic: domain_insights.min_diastolic,
        category: domain_insights.category.to_string(),
        reading_count: domain_insights.reading_count,
        period_days: domain_insights.period_days,
        generated_at: domain_insights.generated_at,
    }
}

/// Convert from data model to domain entity for blood pressure insights
pub fn convert_to_domain_insights(data_insights: my_health_guide_data::models::blood_pressure::BloodPressureInsights)
    -> Result<BloodPressureInsights, &'static str>
{
    // Parse the category string to get the domain category enum
    let category = match data_insights.category.as_str() {
        "Normal" => BloodPressureCategory::Normal,
        "Elevated" => BloodPressureCategory::Elevated,
        "Hypertension Stage 1" => BloodPressureCategory::Hypertension1,
        "Hypertension Stage 2" => BloodPressureCategory::Hypertension2,
        "Hypertensive Crisis" => BloodPressureCategory::HypertensiveCrisis,
        _ => return Err("Invalid blood pressure category string"),
    };

    Ok(BloodPressureInsights {
        avg_systolic: data_insights.avg_systolic,
        avg_diastolic: data_insights.avg_diastolic,
        avg_pulse: data_insights.avg_pulse,
        max_systolic: data_insights.max_systolic,
        max_diastolic: data_insights.max_diastolic,
        min_systolic: data_insights.min_systolic,
        min_diastolic: data_insights.min_diastolic,
        category,
        reading_count: data_insights.reading_count,
        period_days: data_insights.period_days,
        generated_at: data_insights.generated_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_convert_to_domain_reading() {
        // Create a data model
        let data_reading = my_health_guide_data::models::blood_pressure::BloodPressureReading {
            id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            systolic: 120,
            diastolic: 80,
            pulse: Some(72),
            notes: Some("Test reading".to_string()),
            timestamp: Utc::now().to_rfc3339(),
            position: Some("Sitting".to_string()),
            arm: Some("Left".to_string()),
            device_id: Some("Device123".to_string()),
        };

        // Convert to domain entity
        let domain_reading = convert_to_domain_reading(data_reading.clone());

        // Verify conversion
        assert_eq!(domain_reading.id, data_reading.id);
        assert_eq!(domain_reading.systolic, data_reading.systolic);
        assert_eq!(domain_reading.diastolic, data_reading.diastolic);
        assert_eq!(domain_reading.pulse, data_reading.pulse);
        assert_eq!(domain_reading.notes, data_reading.notes);
        assert_eq!(domain_reading.timestamp, data_reading.timestamp);
        assert_eq!(domain_reading.position, data_reading.position);
        assert_eq!(domain_reading.arm, data_reading.arm);
        assert_eq!(domain_reading.device_id, data_reading.device_id);
    }

    #[test]
    fn test_convert_to_data_create_request() {
        // Create a domain entity
        let domain_request = CreateBloodPressureRequest {
            systolic: 120,
            diastolic: 80,
            pulse: Some(72),
            notes: Some("Test reading".to_string()),
            timestamp: Utc::now().to_rfc3339(),
            position: Some("Sitting".to_string()),
            arm: Some("Left".to_string()),
            device_id: Some("Device123".to_string()),
        };

        // Convert to data model
        let data_request = convert_to_data_create_request(&domain_request);

        // Verify conversion
        assert_eq!(data_request.systolic, domain_request.systolic);
        assert_eq!(data_request.diastolic, domain_request.diastolic);
        assert_eq!(data_request.pulse, domain_request.pulse);
        assert_eq!(data_request.notes, domain_request.notes);
        assert_eq!(data_request.timestamp, domain_request.timestamp);
        assert_eq!(data_request.position, domain_request.position);
        assert_eq!(data_request.arm, domain_request.arm);
        assert_eq!(data_request.device_id, domain_request.device_id);
    }
}
