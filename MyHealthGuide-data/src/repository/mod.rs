// Repository module structure
pub mod errors;
mod blood_pressure;
mod in_memory;
mod storage;

// Re-export commonly used types
pub use errors::RepositoryError;
pub use blood_pressure::{BloodPressureRepository, BloodPressureRepositoryTrait};

// Re-export test modules for both testing and when mock feature is enabled
#[cfg(any(test, feature = "mock"))]
pub use blood_pressure::tests; 