pub mod insights;
pub mod blood_pressure;

// Domain services
// This module contains business logic implementations.

// Re-export service traits and factory functions
pub use blood_pressure::{BloodPressureServiceTrait, create_default_blood_pressure_service};

// Re-export mock service factory functions when the mock feature is enabled
#[cfg(feature = "mock")]
pub use blood_pressure::create_mock_blood_pressure_service;
