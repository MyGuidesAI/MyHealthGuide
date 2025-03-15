// MyHealth Domain
// This crate contains the business logic for the MyHealthGuide application

// Services that implement business logic
pub mod services;

// Authentication
pub mod auth;

// Domain entities
pub mod entities;

// Health checks and system status
pub mod health;

// Re-export the database module from myhealth-data for convenience
pub use MyHealthGuide_data::database;

// Testing utilities - only available with mock feature
#[cfg(feature = "mock")]
pub mod testing;

// Re-export domain models module for backward compatibility
#[deprecated(
    since = "0.2.0",
    note = "Use entities module instead. This will be removed in a future version."
)]
pub mod models; 