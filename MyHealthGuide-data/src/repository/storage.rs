use tracing::debug;
use uuid::Uuid;

use crate::models::blood_pressure::BloodPressureReading;
use crate::database::DatabasePool;
use super::errors::RepositoryError;

/// Database storage operations for blood pressure readings
pub struct DatabaseStorage;

impl DatabaseStorage {
    /// Store a reading in the database
    #[cfg(feature = "sqlite")]
    pub async fn store_reading(pool: &DatabasePool, reading: &BloodPressureReading) -> Result<(), RepositoryError> {
        debug!("Storing blood pressure reading in database: id={}", reading.id);
        
        match pool {
            DatabasePool::SQLite(pool) => {
                let conn = pool.get().map_err(RepositoryError::Pool)?;
                
                conn.execute(
                    "INSERT INTO blood_pressure_readings 
                     (id, systolic, diastolic, pulse, notes, timestamp, position, arm, device_id) 
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    (
                        &reading.id,
                        reading.systolic,
                        reading.diastolic,
                        reading.pulse,
                        &reading.notes,
                        &reading.timestamp,
                        &reading.position,
                        &reading.arm,
                        &reading.device_id,
                    ),
                ).map_err(RepositoryError::Sqlite)?;
                
                Ok(())
            },
            
            #[cfg(feature = "mysql_db")]
            DatabasePool::MySQL(pool) => {
                use mysql::prelude::*;
                
                let mut conn = pool.get()
                    .map_err(|e| RepositoryError::Pool(e))?;
                
                conn.exec_drop(
                    "INSERT INTO blood_pressure_readings 
                     (id, systolic, diastolic, pulse, timestamp, notes, position, arm, device_id) 
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    (
                        &reading.id,
                        reading.systolic,
                        reading.diastolic,
                        reading.pulse,
                        &reading.timestamp,
                        &reading.notes,
                        &reading.position,
                        &reading.arm,
                        &reading.device_id,
                    ),
                ).map_err(|e| RepositoryError::Database(e.to_string().into()))?;
                
                Ok(())
            },
            
            #[cfg(feature = "postgres")]
            DatabasePool::PostgreSQL(pool) => {
                // Get a client from the pool with async/await
                let client = pool.get().await
                    .map_err(|e| RepositoryError::Database(e.to_string().into()))?;
                
                // Execute the query with async/await
                client.execute(
                    "INSERT INTO blood_pressure_readings 
                     (id, systolic, diastolic, pulse, timestamp, notes, position, arm, device_id) 
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                    &[
                        &reading.id,
                        &(reading.systolic as i32),
                        &(reading.diastolic as i32),
                        &reading.pulse.map(|p| p as i32),
                        &reading.timestamp,
                        &reading.notes,
                        &reading.position,
                        &reading.arm,
                        &reading.device_id,
                    ],
                ).await.map_err(|e| RepositoryError::Database(e.to_string().into()))?;
                
                Ok(())
            },
            
            #[allow(unreachable_patterns)]
            _ => Err(RepositoryError::Database("Unsupported database type or not implemented".to_string().into())),
        }
    }
    
    /// Get all readings from the database
    pub async fn get_all(pool: &DatabasePool) -> Result<Vec<BloodPressureReading>, RepositoryError> {
        debug!("Getting all blood pressure readings from database");
        
        match pool {
            DatabasePool::SQLite(pool) => {
                let conn = pool.get()?;
                
                let mut stmt = conn.prepare(
                    "SELECT id, systolic, diastolic, pulse, timestamp, notes, position, arm, device_id
                     FROM blood_pressure_readings ORDER BY timestamp DESC"
                )?;
                
                let readings = stmt.query_map([], |row| {
                    Ok(BloodPressureReading {
                        id: row.get(0)?,
                        systolic: row.get::<_, i32>(1)? as u16,
                        diastolic: row.get::<_, i32>(2)? as u16,
                        pulse: row.get::<_, Option<i32>>(3)?.map(|p| p as u16),
                        timestamp: row.get(4)?,
                        notes: row.get(5)?,
                        position: row.get(6)?,
                        arm: row.get(7)?,
                        device_id: row.get(8)?,
                    })
                })?;
                
                let mut result = Vec::new();
                for reading in readings {
                    result.push(reading?);
                }
                
                Ok(result)
            },
            
            #[cfg(feature = "postgres")]
            DatabasePool::PostgreSQL(pool) => {
                // Get a client from the pool
                let client = pool.get().await
                    .map_err(|e| RepositoryError::Database(e.to_string().into()))?;
                
                // Execute the query
                let rows = client.query(
                    "SELECT id, systolic, diastolic, pulse, timestamp, notes, position, arm, device_id
                     FROM blood_pressure_readings ORDER BY timestamp DESC",
                    &[],
                ).await.map_err(|e| RepositoryError::Database(e.to_string().into()))?;
                
                // Convert the rows to BloodPressureReading objects
                let mut result = Vec::new();
                for row in rows {
                    let reading = BloodPressureReading {
                        id: row.get(0),
                        systolic: row.get::<_, i32>(1) as u16,
                        diastolic: row.get::<_, i32>(2) as u16,
                        pulse: row.get::<_, Option<i32>>(3).map(|p| p as u16),
                        timestamp: row.get(4),
                        notes: row.get(5),
                        position: row.get(6),
                        arm: row.get(7),
                        device_id: row.get(8),
                    };
                    result.push(reading);
                }
                
                Ok(result)
            },
            
            #[allow(unreachable_patterns)]
            _ => Err(RepositoryError::Database("Unsupported database type or not implemented".to_string().into())),
        }
    }
    
    /// Get a reading by ID from the database
    pub async fn get_by_id(pool: &DatabasePool, id: &Uuid) -> Result<Option<BloodPressureReading>, RepositoryError> {
        debug!("Getting blood pressure reading by ID from database: id={}", id);
        
        match pool {
            DatabasePool::SQLite(pool) => {
                let conn = pool.get()?;
                
                let mut stmt = conn.prepare(
                    "SELECT id, systolic, diastolic, pulse, timestamp, notes, position, arm, device_id
                     FROM blood_pressure_readings WHERE id = ?"
                )?;
                
                let reading = stmt.query_row([&id.to_string()], |row| {
                    Ok(BloodPressureReading {
                        id: row.get(0)?,
                        systolic: row.get::<_, i32>(1)? as u16,
                        diastolic: row.get::<_, i32>(2)? as u16,
                        pulse: row.get::<_, Option<i32>>(3)?.map(|p| p as u16),
                        timestamp: row.get(4)?,
                        notes: row.get(5)?,
                        position: row.get(6)?,
                        arm: row.get(7)?,
                        device_id: row.get(8)?,
                    })
                });
                
                match reading {
                    Ok(reading) => Ok(Some(reading)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(RepositoryError::Sqlite(e)),
                }
            },
            
            #[cfg(feature = "postgres")]
            DatabasePool::PostgreSQL(pool) => {
                let client = pool.get().await
                    .map_err(|e| RepositoryError::Database(e.to_string().into()))?;
                
                let rows = client.query(
                    "SELECT id, systolic, diastolic, pulse, timestamp, notes, position, arm, device_id
                     FROM blood_pressure_readings WHERE id = $1",
                    &[&id.to_string()],
                ).await.map_err(|e| RepositoryError::Database(e.to_string().into()))?;
                
                if rows.is_empty() {
                    return Ok(None);
                }
                
                let row = &rows[0];
                let reading = BloodPressureReading {
                    id: row.get(0),
                    systolic: row.get::<_, i32>(1) as u16,
                    diastolic: row.get::<_, i32>(2) as u16,
                    pulse: row.get::<_, Option<i32>>(3).map(|p| p as u16),
                    timestamp: row.get(4),
                    notes: row.get(5),
                    position: row.get(6),
                    arm: row.get(7),
                    device_id: row.get(8),
                };
                
                Ok(Some(reading))
            },
            
            #[allow(unreachable_patterns)]
            _ => Err(RepositoryError::Database("Unsupported database type or not implemented".to_string().into())),
        }
    }
    
    /// Get the latest blood pressure reading from the database
    pub async fn get_latest(pool: &DatabasePool) -> Result<Option<BloodPressureReading>, RepositoryError> {
        debug!("Getting latest blood pressure reading from database");
        
        match pool {
            DatabasePool::SQLite(pool) => {
                let conn = pool.get()?;
                
                let mut stmt = conn.prepare(
                    "SELECT id, systolic, diastolic, pulse, timestamp, notes, position, arm, device_id
                     FROM blood_pressure_readings ORDER BY timestamp DESC LIMIT 1"
                )?;
                
                let reading = stmt.query_row([], |row| {
                    Ok(BloodPressureReading {
                        id: row.get(0)?,
                        systolic: row.get::<_, i32>(1)? as u16,
                        diastolic: row.get::<_, i32>(2)? as u16,
                        pulse: row.get::<_, Option<i32>>(3)?.map(|p| p as u16),
                        timestamp: row.get(4)?,
                        notes: row.get(5)?,
                        position: row.get(6)?,
                        arm: row.get(7)?,
                        device_id: row.get(8)?,
                    })
                });
                
                match reading {
                    Ok(reading) => Ok(Some(reading)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(RepositoryError::Sqlite(e)),
                }
            },
            
            #[cfg(feature = "postgres")]
            DatabasePool::PostgreSQL(pool) => {
                let client = pool.get().await
                    .map_err(|e| RepositoryError::Database(e.to_string().into()))?;
                
                let rows = client.query(
                    "SELECT id, systolic, diastolic, pulse, timestamp, notes, position, arm, device_id
                     FROM blood_pressure_readings ORDER BY timestamp DESC LIMIT 1",
                    &[],
                ).await.map_err(|e| RepositoryError::Database(e.to_string().into()))?;
                
                if rows.is_empty() {
                    return Ok(None);
                }
                
                let row = &rows[0];
                let reading = BloodPressureReading {
                    id: row.get(0),
                    systolic: row.get::<_, i32>(1) as u16,
                    diastolic: row.get::<_, i32>(2) as u16,
                    pulse: row.get::<_, Option<i32>>(3).map(|p| p as u16),
                    timestamp: row.get(4),
                    notes: row.get(5),
                    position: row.get(6),
                    arm: row.get(7),
                    device_id: row.get(8),
                };
                
                Ok(Some(reading))
            },
            
            #[allow(unreachable_patterns)]
            _ => Err(RepositoryError::Database("Unsupported database type or not implemented".to_string().into())),
        }
    }
    
    /// Get filtered readings from the database
    pub async fn get_filtered(
        pool: &DatabasePool,
        start_date: Option<&str>,
        end_date: Option<&str>,
        limit: Option<usize>,
        offset: Option<usize>,
        sort_desc: Option<bool>,
    ) -> Result<(Vec<BloodPressureReading>, usize), RepositoryError> {
        debug!("Getting filtered blood pressure readings from database");
        
        let sort_direction = if sort_desc.unwrap_or(true) { "DESC" } else { "ASC" };
        let limit_val = limit.unwrap_or(100);
        let offset_val = offset.unwrap_or(0);
        
        match pool {
            DatabasePool::SQLite(pool) => {
                let conn = pool.get()?;
                
                // Build query with date filters if provided
                let mut query = String::from(
                    "SELECT id, systolic, diastolic, pulse, timestamp, notes, position, arm, device_id
                     FROM blood_pressure_readings"
                );
                
                let mut where_clauses = Vec::new();
                let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
                
                // Create owned copies of the date strings so they live long enough
                let start_string: Option<String> = start_date.map(|s| s.to_string());
                let end_string: Option<String> = end_date.map(|s| s.to_string());
                
                if let Some(ref start) = start_string {
                    where_clauses.push("timestamp >= ?");
                    params.push(start as &dyn rusqlite::ToSql);
                }
                
                if let Some(ref end) = end_string {
                    where_clauses.push("timestamp <= ?");
                    params.push(end as &dyn rusqlite::ToSql);
                }
                
                if !where_clauses.is_empty() {
                    query.push_str(" WHERE ");
                    query.push_str(&where_clauses.join(" AND "));
                }
                
                // Add sorting
                query.push_str(&format!(" ORDER BY timestamp {}", sort_direction));
                
                // Add pagination
                query.push_str(&format!(" LIMIT {} OFFSET {}", limit_val, offset_val));
                
                // Execute query
                let mut stmt = conn.prepare(&query)?;
                
                let readings = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
                    Ok(BloodPressureReading {
                        id: row.get(0)?,
                        systolic: row.get::<_, i32>(1)? as u16,
                        diastolic: row.get::<_, i32>(2)? as u16,
                        pulse: row.get::<_, Option<i32>>(3)?.map(|p| p as u16),
                        timestamp: row.get(4)?,
                        notes: row.get(5)?,
                        position: row.get(6)?,
                        arm: row.get(7)?,
                        device_id: row.get(8)?,
                    })
                })?;
                
                let mut result = Vec::new();
                for reading in readings {
                    result.push(reading?);
                }
                
                // Get total count for pagination
                let mut count_query = String::from("SELECT COUNT(*) FROM blood_pressure_readings");
                
                if !where_clauses.is_empty() {
                    count_query.push_str(" WHERE ");
                    count_query.push_str(&where_clauses.join(" AND "));
                }
                
                let mut count_stmt = conn.prepare(&count_query)?;
                let total: i64 = count_stmt.query_row(
                    rusqlite::params_from_iter(params.iter()),
                    |row| row.get(0)
                )?;
                
                Ok((result, total as usize))
            },
            
            #[cfg(feature = "postgres")]
            DatabasePool::PostgreSQL(pool) => {
                let client = pool.get().await
                    .map_err(|e| RepositoryError::Database(e.to_string().into()))?;
                
                // Build query with date filters
                let mut query = String::from(
                    "SELECT id, systolic, diastolic, pulse, timestamp, notes, position, arm, device_id
                     FROM blood_pressure_readings"
                );
                
                let mut where_clauses = Vec::new();
                let mut params = Vec::new();
                let mut param_index = 1;
                
                if let Some(start) = start_date {
                    where_clauses.push(format!("timestamp >= ${}", param_index));
                    params.push(start);
                    param_index += 1;
                }
                
                if let Some(end) = end_date {
                    where_clauses.push(format!("timestamp <= ${}", param_index));
                    params.push(end);
                    param_index += 1;
                }
                
                if !where_clauses.is_empty() {
                    query.push_str(" WHERE ");
                    query.push_str(&where_clauses.join(" AND "));
                }
                
                // Add sorting
                query.push_str(&format!(" ORDER BY timestamp {}", sort_direction));
                
                // Add pagination
                query.push_str(&format!(" LIMIT {} OFFSET {}", limit_val, offset_val));
                
                // Execute query
                let param_values: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = 
                    params.iter().map(|p| p as &(dyn tokio_postgres::types::ToSql + Sync)).collect();
                
                let rows = client.query(&query, &param_values[..])
                    .await.map_err(|e| RepositoryError::Database(e.to_string().into()))?;
                
                // Convert rows to BloodPressureReading objects
                let mut result = Vec::new();
                for row in rows {
                    let reading = BloodPressureReading {
                        id: row.get(0),
                        systolic: row.get::<_, i32>(1) as u16,
                        diastolic: row.get::<_, i32>(2) as u16,
                        pulse: row.get::<_, Option<i32>>(3).map(|p| p as u16),
                        timestamp: row.get(4),
                        notes: row.get(5),
                        position: row.get(6),
                        arm: row.get(7),
                        device_id: row.get(8),
                    };
                    result.push(reading);
                }
                
                // Get total count for pagination
                let mut count_query = String::from("SELECT COUNT(*) FROM blood_pressure_readings");
                
                if !where_clauses.is_empty() {
                    count_query.push_str(" WHERE ");
                    count_query.push_str(&where_clauses.join(" AND "));
                }
                
                let count_row = client.query_one(&count_query, &param_values[..])
                    .await.map_err(|e| RepositoryError::Database(e.to_string().into()))?;
                
                let total: i64 = count_row.get(0);
                
                Ok((result, total as usize))
            },
            
            #[allow(unreachable_patterns)]
            _ => Err(RepositoryError::Database("Unsupported database type or not implemented".to_string().into())),
        }
    }
} 