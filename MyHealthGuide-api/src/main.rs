use std::net::SocketAddr;
use MyHealthGuide_api::api::routes::create_app;
use axum::serve;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging with environment settings
    tracing_subscriber::fmt::init();
    
    // Load environment variables from .env file if it exists
    dotenv::dotenv().ok();
    
    // Create application router
    let app = create_app().await;
    
    // Get port from environment or use default
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);
    
    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting server on {}", addr);
    
    let listener = TcpListener::bind(addr).await?;
    serve(listener, app).await?;
    
    Ok(())
} 