use tokio_postgres::Client;
use tracing::info;

/// Run PostgreSQL database migrations
pub async fn run_migrations(client: &Client) -> Result<(), String> {
    info!("Running PostgreSQL migrations");
    
    create_blood_pressure_table(client).await?;
    create_blood_pressure_index(client).await?;
    
    info!("PostgreSQL migrations completed successfully");
    Ok(())
}

/// Create the blood pressure readings table
async fn create_blood_pressure_table(client: &Client) -> Result<(), String> {
    info!("Creating blood_pressure_readings table if not exists");
    
    client.execute(
        "CREATE TABLE IF NOT EXISTS blood_pressure_readings (
            id VARCHAR(36) PRIMARY KEY,
            systolic INTEGER NOT NULL,
            diastolic INTEGER NOT NULL,
            pulse INTEGER,
            timestamp VARCHAR(30) NOT NULL,
            notes TEXT,
            position VARCHAR(20),
            arm VARCHAR(10),
            device_id VARCHAR(50),
            category VARCHAR(30)
        )",
        &[],
    ).await.map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Create index on timestamp for efficient filtering
async fn create_blood_pressure_index(client: &Client) -> Result<(), String> {
    info!("Creating index on timestamp");
    
    client.execute(
        "CREATE INDEX IF NOT EXISTS idx_blood_pressure_readings_timestamp 
        ON blood_pressure_readings (timestamp DESC)",
        &[],
    ).await.map_err(|e| format!("Failed to create index: {}", e))?;
    
    Ok(())
} 