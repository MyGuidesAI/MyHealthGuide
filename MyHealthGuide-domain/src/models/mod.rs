// DEPRECATED: Use entities module instead
// This module is maintained for backwards compatibility

// We'll directly re-export from the entities module to ensure consistency
// and reduce duplicate code

#[deprecated(
    since = "0.2.0",
    note = "Use entities module instead. This module will be removed in a future version."
)]
pub mod blood_pressure {
    // Re-export all types from entities module
    #[deprecated(
        since = "0.2.0",
        note = "Import from entities module instead. This will be removed in a future version."
    )]
    pub use crate::entities::blood_pressure::*;
}

// Re-export common types for backwards compatibility
#[deprecated(
    since = "0.2.0", 
    note = "Import from entities module instead. These re-exports will be removed in a future version."
)]
pub use crate::entities::blood_pressure::{BloodPressureReading, CreateBloodPressureRequest, BloodPressureInsights, BloodPressureCategory}; 
