use std::net::SocketAddr;
use std::path::PathBuf;
use dotenv::dotenv;
use tokio::signal;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter,
};
use MyHealthGuide_api::api::create_application;

/// Application error type for the main function
/// 
/// This custom error type handles the specific errors that can occur
/// during server initialization and running.
#[derive(Debug)]
enum AppError {
    /// Error that occurs during server operations
    Server(std::io::Error),
    /// Error that occurs when parsing the port number
    PortParse(std::num::ParseIntError),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Server(e) => write!(f, "Server error: {}", e),
            AppError::PortParse(e) => write!(f, "Port parsing error: {}", e),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Server(err)
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(err: std::num::ParseIntError) -> Self {
        AppError::PortParse(err)
    }
}

/// The main entry point for the MyHealthGuide API server
///
/// This function:
/// 1. Initializes environment variables from .env file
/// 2. Sets up tracing for logging
/// 3. Ensures the data directory exists
/// 4. Initializes the database connection pool
/// 5. Creates and starts the Axum web application
/// 6. Handles graceful shutdown
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    if dotenv().is_err() {
        eprintln!("Warning: .env file not found or couldn't be read. Using environment variables.");
    }

    // Initialize tracing for structured logging
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(fmt::layer()
            .with_span_events(FmtSpan::CLOSE)
            .with_target(false)
            .with_ansi(true)
            .with_timer(fmt::time::uptime())
            .with_writer(std::io::stdout))
        .with(env_filter)
        .init();

    info!("🚀 Starting MyHealthGuide API server");

    // Define the database path - this is handled by the domain layer now
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "data".to_string());
    let db_path = PathBuf::from(&data_dir).join("health_guide.db");
    
    // Create the data directory if it doesn't exist
    if !PathBuf::from(&data_dir).exists() {
        info!("Creating data directory: {}", data_dir);
        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            error!("Failed to create data directory: {}", e);
            std::process::exit(1);
        }
    }

    // Set DB_SQLITE_PATH environment variable if not already set
    if std::env::var("DB_SQLITE_PATH").is_err() {
        std::env::set_var("DB_SQLITE_PATH", db_path.to_string_lossy().to_string());
        info!("Set DB_SQLITE_PATH to {}", db_path.display());
    }

    // Explicitly initialize the database connection pool
    // Use MyHealthGuide_domain to access MyHealthGuide_data functions
    match MyHealthGuide_domain::database::initialize_database_pool() {
        Ok(_) => info!("Database pool initialized successfully"),
        Err(e) => {
            error!("Failed to initialize database pool: {}", e);
            // Continue running even if the database initialization fails
            // The application will fall back to in-memory storage
        }
    }

    // Database initialization is now handled by the domain layer factory functions
    // Let's just log what database we're using
    let db_type = std::env::var("DB_TYPE")
        .unwrap_or_else(|_| "sqlite".to_string())
        .to_lowercase();

    match db_type.as_str() {
        "sqlite" => {
            info!("Using SQLite database at {}", db_path.display());
        }
        "postgres" => {
            info!("Using PostgreSQL database (connection details managed by domain layer)");
        }
        _ => {
            error!("Unsupported database type: {}", db_type);
            std::process::exit(1);
        }
    }

    // Initialize server start time for uptime reporting in health checks
    MyHealthGuide_api::api::handlers::health::initialize_server_start_time();

    // Create the Axum application with all routes and middleware
    let app = create_application().await;

    // Get the port from environment or use default 3000
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");
    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Listening on {}", addr);

    // Create a TCP listener and bind to the address
    let listener = TcpListener::bind(addr).await?;
    
    // Serve the application with graceful shutdown support
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    info!("Server shutdown complete");
    Ok(())
}

/// Sets up a signal handler for graceful shutdown
///
/// This function creates an async task that waits for either:
/// - CTRL+C signal
/// - SIGTERM (on Unix systems)
/// 
/// When either signal is received, the function returns and triggers
/// the graceful shutdown process.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutting down server...");
} 