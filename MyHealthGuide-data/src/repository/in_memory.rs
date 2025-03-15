use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::blood_pressure::BloodPressureReading;
use super::errors::RepositoryError;

/// In-memory storage implementation for blood pressure readings
#[derive(Debug, Clone)]
pub struct InMemoryStorage {
    /// Storage for blood pressure readings
    readings: Arc<Mutex<HashMap<String, BloodPressureReading>>>,
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self {
            readings: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Store a reading in memory
    pub async fn store_reading(&self, reading: &BloodPressureReading) -> Result<BloodPressureReading, RepositoryError> {
        let mut store = self.readings.lock().map_err(|e| RepositoryError::MutexLock(e.to_string()))?;
        store.insert(reading.id.clone(), reading.clone());
        Ok(reading.clone())
    }

    /// Get all readings from memory
    pub async fn get_all(&self) -> Result<Vec<BloodPressureReading>, RepositoryError> {
        let store = self.readings.lock().map_err(|e| RepositoryError::MutexLock(e.to_string()))?;
        let readings: Vec<BloodPressureReading> = store.values().cloned().collect();
        Ok(readings)
    }

    /// Get the latest reading from memory
    pub async fn get_latest(&self) -> Result<Option<BloodPressureReading>, RepositoryError> {
        let store = self.readings.lock().map_err(|e| RepositoryError::MutexLock(e.to_string()))?;
        
        // Sort by timestamp and get the latest
        let mut readings: Vec<BloodPressureReading> = store.values().cloned().collect();
        readings.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(readings.first().cloned())
    }

    /// Get a reading by ID from memory
    pub async fn get_by_id(&self, id: &Uuid) -> Result<Option<BloodPressureReading>, RepositoryError> {
        let store = self.readings.lock().map_err(|e| RepositoryError::MutexLock(e.to_string()))?;
        Ok(store.get(&id.to_string()).cloned())
    }

    /// Get filtered readings from memory
    pub async fn get_filtered(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
        limit: Option<usize>,
        offset: Option<usize>,
        sort_desc: Option<bool>,
    ) -> Result<(Vec<BloodPressureReading>, usize), RepositoryError> {
        let store = self.readings.lock().map_err(|e| RepositoryError::MutexLock(e.to_string()))?;
        let sort_desc = sort_desc.unwrap_or(true);
        
        // First collect and filter all readings
        let mut readings: Vec<BloodPressureReading> = store.values().filter(|&reading| {
                // Filter by date range if specified
                if let Some(start_date) = start_date {
                    if reading.timestamp.as_str() < start_date {
                        return false;
                    }
                }
                
                if let Some(end_date) = end_date {
                    if reading.timestamp.as_str() > end_date {
                        return false;
                    }
                }
                
                true
            }).cloned()
            .collect();
        
        // Sort by timestamp
        readings.sort_by(|a, b| {
            let cmp = a.timestamp.cmp(&b.timestamp);
            if sort_desc {
                cmp.reverse()
            } else {
                cmp
            }
        });
        
        // Apply pagination
        let total = readings.len();
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(total);
        
        let page = readings
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();
            
        Ok((page, total))
    }
} 