// Database migrations module
// This will be implemented properly in the future

// Import specific functions from each module instead of using glob imports
mod sqlite;
pub use sqlite::run_migrations as run_sqlite_migrations;

#[cfg(feature = "mysql_db")]
mod mysql;
#[cfg(feature = "mysql_db")]
pub use mysql::run_migrations as run_mysql_migrations;

#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "postgres")]
pub use postgres::run_migrations as run_postgres_migrations; 