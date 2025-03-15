use rusqlite::Connection;
use tracing::{info};

/// Run SQLite migrations
pub fn run_migrations(conn: &Connection) -> Result<(), String> {
    info!("Running SQLite migrations");
    
    create_blood_pressure_table(conn)?;
    create_blood_pressure_index(conn)?;
    
    info!("SQLite migrations completed successfully");
    Ok(())
}

/// Create the blood pressure readings table
fn create_blood_pressure_table(conn: &Connection) -> Result<(), String> {
    info!("Creating blood_pressure_readings table if not exists");
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS blood_pressure_readings (
            id TEXT PRIMARY KEY,
            systolic INTEGER NOT NULL,
            diastolic INTEGER NOT NULL,
            pulse INTEGER,
            timestamp TEXT NOT NULL,
            notes TEXT,
            position TEXT,
            arm TEXT,
            device_id TEXT,
            category TEXT
        )",
        [],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Create index on timestamp for efficient filtering
fn create_blood_pressure_index(conn: &Connection) -> Result<(), String> {
    info!("Creating index on timestamp");
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_blood_pressure_readings_timestamp 
        ON blood_pressure_readings (timestamp DESC)",
        [],
    ).map_err(|e| format!("Failed to create index: {}", e))?;
    
    Ok(())
} 