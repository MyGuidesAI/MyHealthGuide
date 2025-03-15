use tracing::error;
use thiserror::Error;

// Database modules
pub mod connection;
pub mod migrations;

// Re-export database connection functions
pub use connection::*;

// Empty tests module for compatibility
#[cfg(test)]
pub mod tests {
    // Test-specific code will go here
}

/// Database error enum
#[derive(Debug, Clone, Error)]
pub enum DatabaseError {
    /// Generic database error
    #[error("Database error: {0}")]
    GenericError(String),
    
    /// Connection error
    #[error("Failed to connect to database: {0}")]
    ConnectionError(String),
    
    /// Configuration error
    #[error("Database configuration error: {0}")]
    ConfigError(String),
    
    /// Migration error
    #[error("Database migration error: {0}")]
    MigrationError(String),
    
    /// Query error
    #[error("Database query error: {0}")]
    QueryError(String),
    
    /// Transaction error
    #[error("Database transaction error: {0}")]
    TransactionError(String),
}

impl From<String> for DatabaseError {
    fn from(error: String) -> Self {
        DatabaseError::GenericError(error)
    }
} 