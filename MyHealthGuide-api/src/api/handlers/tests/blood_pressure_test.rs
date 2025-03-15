#[cfg(test)]
mod blood_pressure_tests {
    use MyHealthGuide_domain::entities::blood_pressure::{BloodPressureReading, CreateBloodPressureRequest};
    use MyHealthGuide_domain::services::BloodPressureServiceTrait;
    use MyHealthGuide_domain::testing::MockBloodPressureService;
    use std::sync::Arc;
    
    use chrono::Utc;

    #[test]
    fn test_mock_service_creation() {
        // Verify we can create a mock service
        let mock_service = Arc::new(MockBloodPressureService::new());
        
        // Verify the service implements the BloodPressureServiceTrait
        let _: Arc<dyn BloodPressureServiceTrait + Send + Sync> = mock_service;
    }
    
    #[test]
    fn test_create_reading_with_mock() {
        // Create a mock service
        let mock_service = Arc::new(MockBloodPressureService::new());
        
        // Create a request
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
        
        // Use the mock service to create a reading
        let result = mock_service.create_reading(request);
        
        // Verify the result
        assert!(result.is_ok());
        let reading = result.unwrap();
        assert_eq!(reading.systolic, 120);
        assert_eq!(reading.diastolic, 80);
        assert_eq!(reading.pulse, Some(72));
    }
    
    #[test]
    fn test_mock_with_preconfigured_behavior() {
        // Create a mock service with validation failure
        let mock_service = Arc::new(
            MockBloodPressureService::new()
                .with_validation_failure()
        );
        
        // Create a request
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
        
        // Use the mock service to create a reading, which should fail validation
        let result = mock_service.create_reading(request);
        
        // Verify the result
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("validation"));
    }
    
    #[test]
    fn test_mock_with_preloaded_data() {
        // Create a test reading
        let test_id = "12345678-1234-1234-1234-123456789012".to_string();
        let preloaded_reading = BloodPressureReading {
            id: test_id.clone(),
            systolic: 135,
            diastolic: 85,
            pulse: Some(75),
            notes: Some("Test reading".to_string()),
            timestamp: Utc::now().to_rfc3339(),
            position: Some("Sitting".to_string()),
            arm: Some("Left".to_string()),
            device_id: None,
        };
        
        // Create a mock service with preloaded data
        let mock_service = Arc::new(
            MockBloodPressureService::new()
                .with_reading(preloaded_reading)
        );
        
        // Retrieve the reading by ID
        let result = mock_service.get_reading_by_id(&test_id);
        
        // Verify the result
        assert!(result.is_ok());
        let reading = result.unwrap();
        assert_eq!(reading.id, test_id);
        assert_eq!(reading.systolic, 135);
        assert_eq!(reading.diastolic, 85);
        assert_eq!(reading.pulse, Some(75));
        assert_eq!(reading.notes, Some("Test reading".to_string()));
        assert_eq!(reading.position, Some("Sitting".to_string()));
        assert_eq!(reading.arm, Some("Left".to_string()));
        
        // Verify we can get all readings
        let all_readings = mock_service.get_all_readings().unwrap();
        assert_eq!(all_readings.len(), 1);
        
        // Verify filtered readings work too
        let (filtered, count) = mock_service.get_filtered_readings(None, None, Some(10), Some(0), Some(true)).unwrap();
        assert_eq!(count, 1);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, test_id);
    }
    
    #[test]
    fn test_mock_with_multiple_readings() {
        // Create readings for testing
        let now = Utc::now().to_rfc3339();
        let yesterday = (Utc::now() - chrono::Duration::days(1)).to_rfc3339();
        let two_days_ago = (Utc::now() - chrono::Duration::days(2)).to_rfc3339();
        
        let reading1 = BloodPressureReading {
            id: "reading1".to_string(),
            systolic: 120,
            diastolic: 80,
            pulse: Some(72),
            notes: None,
            timestamp: now.clone(),
            position: None,
            arm: None,
            device_id: None,
        };
        
        let reading2 = BloodPressureReading {
            id: "reading2".to_string(),
            systolic: 130,
            diastolic: 85,
            pulse: Some(75),
            notes: Some("After exercise".to_string()),
            timestamp: yesterday.clone(),
            position: Some("Sitting".to_string()),
            arm: Some("Left".to_string()),
            device_id: None,
        };
        
        let reading3 = BloodPressureReading {
            id: "reading3".to_string(),
            systolic: 115,
            diastolic: 75,
            pulse: Some(68),
            notes: Some("Morning reading".to_string()),
            timestamp: two_days_ago.clone(),
            position: Some("Sitting".to_string()),
            arm: Some("Right".to_string()),
            device_id: None,
        };
        
        // Create a mock service with the pre-loaded readings
        let mock_service = Arc::new(
            MockBloodPressureService::new()
                .with_readings(vec![reading1.clone(), reading2.clone(), reading3.clone()])
        );
        
        // Test get_all_readings
        let all_readings = mock_service.get_all_readings().unwrap();
        assert_eq!(all_readings.len(), 3);
        
        // Test get_reading_by_id
        let reading = mock_service.get_reading_by_id("reading2").unwrap();
        assert_eq!(reading.systolic, 130);
        assert_eq!(reading.diastolic, 85);
        
        // Test get_filtered_readings with limit
        let (limited_readings, total) = mock_service.get_filtered_readings(
            None, None, Some(2), None, Some(true)
        ).unwrap();
        
        assert_eq!(total, 3);  // Total should be 3
        assert_eq!(limited_readings.len(), 2);  // But only 2 returned due to limit
        
        // Test get_filtered_readings with date range
        let start_date = two_days_ago.clone();
        let end_date = yesterday.clone();
        
        let (ranged_readings, _) = mock_service.get_filtered_readings(
            Some(start_date), Some(end_date), None, None, None
        ).unwrap();
        
        // Should only include reading2 and reading3, not reading1 (which is today)
        assert_eq!(ranged_readings.len(), 2);
        assert!(ranged_readings.iter().any(|r| r.id == "reading2"));
        assert!(ranged_readings.iter().any(|r| r.id == "reading3"));
        assert!(!ranged_readings.iter().any(|r| r.id == "reading1"));
        
        // Test sorting (ascending by default)
        let (sorted_asc, _) = mock_service.get_filtered_readings(
            None, None, None, None, Some(false)
        ).unwrap();
        
        assert_eq!(sorted_asc.len(), 3);
        assert_eq!(sorted_asc[0].id, "reading3");  // Oldest first
        assert_eq!(sorted_asc[2].id, "reading1");  // Newest last
        
        // Test sorting (descending)
        let (sorted_desc, _) = mock_service.get_filtered_readings(
            None, None, None, None, Some(true)
        ).unwrap();
        
        assert_eq!(sorted_desc.len(), 3);
        assert_eq!(sorted_desc[0].id, "reading1");  // Newest first
        assert_eq!(sorted_desc[2].id, "reading3");  // Oldest last
    }
} 