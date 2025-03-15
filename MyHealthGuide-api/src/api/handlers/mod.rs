pub mod health;
pub mod blood_pressure;

// Tests module
#[cfg(test)]
mod tests;

// Re-export handlers for easier imports
pub use blood_pressure::{
    create_blood_pressure, get_blood_pressure, get_blood_pressure_history, get_blood_pressure_insights,
};
pub use health::health_check; 