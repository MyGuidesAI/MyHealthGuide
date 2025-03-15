# Mock Implementations for Testing

This directory contains mock implementations of domain services for use in testing. These mocks are only available when the `mock` feature is enabled.

## Usage

To use these mock implementations in your tests, you need to:

1. Enable the `mock` feature in your `Cargo.toml`:

```toml
[features]
# ...other features
mock = ["myhealth-domain/mock"]
```

2. Import the mocks in your test files:

```rust
#[cfg(test)]
mod tests {
    use myhealth_domain::testing::MockBloodPressureService;
    use myhealth_domain::services::BloodPressureServiceTrait;
    use std::sync::Arc;
    
    #[test]
    fn test_with_mock() {
        // Create a new mock service
        let mock_service = Arc::new(MockBloodPressureService::new());
        
        // Use it as a trait object
        let service: Arc<dyn BloodPressureServiceTrait + Send + Sync> = mock_service;
        
        // Use the mock in your test
        // ...
    }
}
```

## Available Mocks

### MockBloodPressureService

A mock implementation of the `BloodPressureServiceTrait` that can be configured for different test scenarios:

- **Basic usage**:
  ```rust
  let mock_service = Arc::new(MockBloodPressureService::new());
  let service: Arc<dyn BloodPressureServiceTrait + Send + Sync> = mock_service;
  ```

- **With validation failure**:
  ```rust
  let mock_service = Arc::new(
      MockBloodPressureService::new()
          .with_validation_failure()
  );
  ```

- **With creation failure**:
  ```rust
  let mock_service = Arc::new(
      MockBloodPressureService::new()
          .with_creation_failure()
  );
  ```

- **With predefined readings**:
  ```rust
  let reading = BloodPressureReading {
      // ... reading details
  };
  let mock_service = Arc::new(
      MockBloodPressureService::new()
          .with_reading(reading)
  );
  ```

- **With multiple predefined readings**:
  ```rust
  let readings = vec![
      BloodPressureReading { /* ... details ... */ },
      BloodPressureReading { /* ... details ... */ },
  ];
  let mock_service = Arc::new(
      MockBloodPressureService::new()
          .with_readings(readings)
  );
  ```

### MockHealthService

A mock implementation of health services that can be used to test system health monitoring:

- **Basic usage (all healthy)**:
  ```rust
  let mock_service = Arc::new(MockHealthService::new());
  let service: Arc<dyn HealthServiceTrait + Send + Sync> = mock_service;
  ```

- **With degraded database**:
  ```rust
  let mock_service = Arc::new(
      MockHealthService::new()
          .with_degraded_database()
  );
  ```

- **With unhealthy database**:
  ```rust
  let mock_service = Arc::new(
      MockHealthService::new()
          .with_unhealthy_database()
  );
  ```

- **With custom system status**:
  ```rust
  let mock_service = Arc::new(
      MockHealthService::new()
          .with_system_status(SystemStatus::Degraded)
  );
  ```

- **With custom components**:
  ```rust
  let mock_service = Arc::new(
      MockHealthService::new()
          .with_component("cache", ComponentStatus::Degraded, Some("Cache hit rate is low".to_string()))
  );
  ```

## Factory Functions

The domain layer also provides factory functions to create mock services:

```rust
// For blood pressure service
use myhealth_domain::services::create_mock_blood_pressure_service;

#[test]
fn test_with_factory_mock() {
    let mock_service = create_mock_blood_pressure_service();
    // Use the mock service in your test
}

// For health service
use myhealth_domain::testing::create_mock_health_service;

#[test]
fn test_with_health_mock() {
    let mock_service = create_mock_health_service();
    // Use the mock health service in your test
}
```

## How it Works

The mock implementations store data in memory and provide configurable behavior for testing different scenarios. They implement the same traits as the real services, allowing them to be used interchangeably in tests.

## Best Practices

1. **Use trait objects**: Store your mock as a trait object to ensure your code works with any implementation:
   ```rust
   let service: Arc<dyn BloodPressureServiceTrait + Send + Sync> = Arc::new(MockBloodPressureService::new());
   ```

2. **Test error conditions**: Use methods like `with_validation_failure()` and `with_unhealthy_database()` to test how your code handles errors.

3. **Test with predefined data**: Use `with_reading()` and similar methods to set up specific test scenarios.

4. **Use in integration tests**: These mocks are particularly useful for testing API handlers without needing a database or other external dependencies.

5. **Test all states**: For health monitoring, test healthy, degraded, and unhealthy states to ensure your application responds correctly to each scenario.

6. **Follow component isolation pattern**: Create traits in the domain layer and implement them with mocks in test modules to maintain proper separation of concerns. 