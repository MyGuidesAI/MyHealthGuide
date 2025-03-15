use std::sync::PoisonError;
use thiserror::Error;
use crate::database::DatabaseError;

/// Error type for repository operations
#[derive(Error, Debug)]
pub enum RepositoryError {
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    /// SQLite error
    #[cfg(feature = "sqlite")]
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    
    /// Connection pool error
    #[error("Connection pool error: {0}")]
    Pool(#[from] r2d2::Error),
    
    /// Lock error
    #[error("Lock error: {0}")]
    Lock(String),
    
    /// MySQL error
    #[cfg(feature = "mysql_db")]
    #[error("MySQL error: {0}")]
    MySql(#[from] mysql::Error),
    
    /// PostgreSQL error
    #[cfg(feature = "postgres")]
    #[error("PostgreSQL error: {0}")]
    Postgres(#[from] tokio_postgres::Error),
    
    /// Not found error
    #[error("Reading not found: {0}")]
    NotFound(String),
    
    /// Pagination error
    #[error("Pagination error: {0}")]
    Pagination(String),
    
    /// Date parsing error
    #[error("Date parsing error: {0}")]
    DateParse(String),
    
    /// Mutex lock error
    #[error("Mutex lock error: {0}")]
    MutexLock(String),
}

impl<T> From<PoisonError<T>> for RepositoryError {
    fn from(error: PoisonError<T>) -> Self {
        RepositoryError::Lock(error.to_string())
    }
}

impl From<String> for RepositoryError {
    fn from(error: String) -> Self {
        // Determine if it's a validation error based on the error message
        if error.contains("validation") || error.contains("invalid") {
            RepositoryError::Validation(error)
        } else {
            RepositoryError::Database(DatabaseError::GenericError(error))
        }
    }
} 