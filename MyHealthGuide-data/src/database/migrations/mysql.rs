use mysql::prelude::*;
use mysql::Conn;
use tracing::info;

/// Run MySQL database migrations
pub fn run_migrations(conn: &mut Conn) -> Result<(), String> {
    info!("Running MySQL migrations");
    
    create_blood_pressure_table(conn)?;
    create_blood_pressure_index(conn)?;
    
    info!("MySQL migrations completed successfully");
    Ok(())
}

/// Create the blood pressure readings table
fn create_blood_pressure_table(conn: &mut Conn) -> Result<(), String> {
    info!("Creating blood_pressure_readings table if not exists");
    
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS blood_pressure_readings (
            id VARCHAR(36) PRIMARY KEY,
            systolic INT NOT NULL,
            diastolic INT NOT NULL,
            pulse INT,
            timestamp VARCHAR(30) NOT NULL,
            notes TEXT,
            position VARCHAR(20),
            arm VARCHAR(10),
            device_id VARCHAR(50),
            category VARCHAR(30)
        )"
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Create index on timestamp for efficient filtering
fn create_blood_pressure_index(conn: &mut Conn) -> Result<(), String> {
    info!("Creating index on timestamp");
    
    conn.query_drop(
        "CREATE INDEX IF NOT EXISTS idx_blood_pressure_readings_timestamp 
        ON blood_pressure_readings (timestamp DESC)"
    ).map_err(|e| format!("Failed to create index: {}", e))?;
    
    Ok(())
} 