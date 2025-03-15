use crate::entities::blood_pressure::BloodPressureCategory;

/// Categorize blood pressure based on measurements
pub fn categorize_blood_pressure(systolic: u16, diastolic: u16) -> BloodPressureCategory {
    if systolic >= 180 || diastolic >= 120 {
        BloodPressureCategory::HypertensiveCrisis
    } else if systolic >= 140 || diastolic >= 90 {
        BloodPressureCategory::Hypertension2
    } else if systolic >= 130 || diastolic >= 80 {
        BloodPressureCategory::Hypertension1
    } else if systolic >= 120 && diastolic < 80 {
        BloodPressureCategory::Elevated
    } else {
        BloodPressureCategory::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bp_category_normal() {
        let category = categorize_blood_pressure(110, 75);
        assert_eq!(category, BloodPressureCategory::Normal);
    }
    
    #[test]
    fn test_bp_category_elevated() {
        let category = categorize_blood_pressure(125, 75);
        assert_eq!(category, BloodPressureCategory::Elevated);
    }
    
    #[test]
    fn test_bp_category_hypertension1() {
        // Test systolic in range
        let category = categorize_blood_pressure(135, 75);
        assert_eq!(category, BloodPressureCategory::Hypertension1);
        
        // Test diastolic in range
        let category = categorize_blood_pressure(120, 85);
        assert_eq!(category, BloodPressureCategory::Hypertension1);
    }
    
    #[test]
    fn test_bp_category_hypertension2() {
        // Test systolic in range
        let category = categorize_blood_pressure(145, 75);
        assert_eq!(category, BloodPressureCategory::Hypertension2);
        
        // Test diastolic in range
        let category = categorize_blood_pressure(120, 95);
        assert_eq!(category, BloodPressureCategory::Hypertension2);
    }
    
    #[test]
    fn test_bp_category_crisis() {
        // Test systolic in range
        let category = categorize_blood_pressure(185, 75);
        assert_eq!(category, BloodPressureCategory::HypertensiveCrisis);
        
        // Test diastolic in range
        let category = categorize_blood_pressure(120, 125);
        assert_eq!(category, BloodPressureCategory::HypertensiveCrisis);
    }
} 