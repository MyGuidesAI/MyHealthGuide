use chrono::Utc;
use tracing::{debug, error};
use uuid::Uuid;
use async_trait::async_trait;

use crate::models::blood_pressure::{BloodPressureReading, CreateBloodPressureRequest, BloodPressureInsights};
use crate::database::get_db_pool;
use super::errors::RepositoryError;
use super::in_memory::InMemoryStorage;
use super::storage::DatabaseStorage;

/// Repository trait for blood pressure readings
#[async_trait]
pub trait BloodPressureRepositoryTrait {
    /// Create a new blood pressure reading from a request
    async fn create(&self, request: CreateBloodPressureRequest) -> Result<BloodPressureReading, RepositoryError>;
    
    /// Get all blood pressure readings
    async fn get_all(&self) -> Result<Vec<BloodPressureReading>, RepositoryError>;
    
    /// Get the latest blood pressure reading
    async fn get_latest(&self) -> Result<Option<BloodPressureReading>, RepositoryError>;
    
    /// Get a blood pressure reading by ID
    async fn get_by_id(&self, id: Uuid) -> Result<Option<BloodPressureReading>, RepositoryError>;
    
    /// Get filtered blood pressure readings
    async fn get_filtered(
        &self,
        start_date: Option<String>,
        end_date: Option<String>,
        limit: Option<usize>,
        offset: Option<usize>,
        sort_desc: Option<bool>,
    ) -> Result<(Vec<BloodPressureReading>, usize), RepositoryError>;
    
    /// Generate insights from blood pressure readings
    async fn generate_insights(&self, timeframe_days: u32) -> Result<Option<BloodPressureInsights>, RepositoryError>;
}

/// Repository for blood pressure readings.
/// This implementation can use different database backends with SQLite as the default.
#[derive(Debug, Clone, Default)]
pub struct BloodPressureRepository {
    /// In-memory storage for when database is not available
    storage: InMemoryStorage,
}

impl BloodPressureRepository {
    /// Create a new repository
    pub fn new() -> Self {
        Self {
            storage: InMemoryStorage::new(),
        }
    }
}

#[async_trait]
impl BloodPressureRepositoryTrait for BloodPressureRepository {
    /// Create a new blood pressure reading from a request
    async fn create(&self, request: CreateBloodPressureRequest) -> Result<BloodPressureReading, RepositoryError> {
        // Generate a unique ID
        let id = Uuid::new_v4();
        
        // Create the reading object
        let reading = BloodPressureReading {
            id: id.to_string(),
            systolic: request.systolic,
            diastolic: request.diastolic,
            pulse: request.pulse,
            notes: request.notes,
            timestamp: request.timestamp,
            position: request.position,
            arm: request.arm,
            device_id: request.device_id,
        };
        
        // Try to store in database first
        match get_db_pool() {
            Ok(pool) => {
                debug!("Storing blood pressure reading in database: {}", reading.id);
                match DatabaseStorage::store_reading(&pool, &reading).await {
                    Ok(_) => Ok(reading),
                    Err(e) => {
                        error!("Failed to store reading in database: {}", e);
                        // Fall back to in-memory storage
                        self.storage.store_reading(&reading).await
                    }
                }
            },
            Err(e) => {
                // Database not available, use in-memory storage
                debug!("Database not available ({}), using in-memory storage", e);
                self.storage.store_reading(&reading).await
            }
        }
    }

    /// Get all blood pressure readings
    async fn get_all(&self) -> Result<Vec<BloodPressureReading>, RepositoryError> {
        // Try to get from database first
        match get_db_pool() {
            Ok(pool) => {
                debug!("Getting all blood pressure readings from database");
                match DatabaseStorage::get_all(&pool).await {
                    Ok(readings) => Ok(readings),
                    Err(e) => {
                        error!("Failed to get readings from database: {}", e);
                        // Fall back to in-memory storage
                        self.storage.get_all().await
                    }
                }
            },
            Err(e) => {
                // Database not available or error occurred, use in-memory storage
                debug!("Database not available ({}), using in-memory storage for get_all", e);
                self.storage.get_all().await
            }
        }
    }
    
    /// Get the latest blood pressure reading
    async fn get_latest(&self) -> Result<Option<BloodPressureReading>, RepositoryError> {
        // Try to get from database first
        match get_db_pool() {
            Ok(pool) => {
                debug!("Getting latest blood pressure reading from database");
                match DatabaseStorage::get_latest(&pool).await {
                    Ok(reading) => Ok(reading),
                    Err(e) => {
                        error!("Failed to get latest reading from database: {}", e);
                        // Fall back to in-memory storage
                        self.storage.get_latest().await
                    }
                }
            },
            Err(e) => {
                // Database not available or error occurred, use in-memory storage
                debug!("Database not available ({}), using in-memory storage for get_latest", e);
                self.storage.get_latest().await
            }
        }
    }

    /// Get a blood pressure reading by ID
    async fn get_by_id(&self, id: Uuid) -> Result<Option<BloodPressureReading>, RepositoryError> {
        // Try to get from database first
        match get_db_pool() {
            Ok(pool) => {
                debug!("Getting blood pressure reading by ID from database: {}", id);
                match DatabaseStorage::get_by_id(&pool, &id).await {
                    Ok(reading) => Ok(reading),
                    Err(e) => {
                        error!("Failed to get reading by ID from database: {}", e);
                        // Fall back to in-memory storage
                        self.storage.get_by_id(&id).await
                    }
                }
            },
            Err(e) => {
                // Database not available or error occurred, use in-memory storage
                debug!("Database not available ({}), using in-memory storage for get_by_id", e);
                self.storage.get_by_id(&id).await
            }
        }
    }
    
    /// Get filtered blood pressure readings
    async fn get_filtered(
        &self,
        start_date: Option<String>,
        end_date: Option<String>,
        limit: Option<usize>,
        offset: Option<usize>,
        sort_desc: Option<bool>,
    ) -> Result<(Vec<BloodPressureReading>, usize), RepositoryError> {
        // Try to get from database first
        match get_db_pool() {
            Ok(pool) => {
                debug!("Getting filtered blood pressure readings from database");
                match DatabaseStorage::get_filtered(
                    &pool,
                    start_date.as_deref(),
                    end_date.as_deref(),
                    limit,
                    offset,
                    sort_desc,
                ).await {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        error!("Failed to get filtered readings from database: {}", e);
                        // Fall back to in-memory storage
                        self.storage.get_filtered(
                            start_date.as_deref(),
                            end_date.as_deref(),
                            limit,
                            offset,
                            sort_desc,
                        ).await
                    }
                }
            },
            Err(e) => {
                // Database not available or error occurred, use in-memory storage
                debug!("Database not available ({}), using in-memory storage for get_filtered", e);
                // Convert String to str for in-memory storage
                self.storage.get_filtered(
                    start_date.as_deref(),
                    end_date.as_deref(),
                    limit,
                    offset,
                    sort_desc,
                ).await
            }
        }
    }
    
    /// Generate insights from blood pressure readings
    async fn generate_insights(&self, timeframe_days: u32) -> Result<Option<BloodPressureInsights>, RepositoryError> {
        // Get readings within the timeframe
        let start_date = chrono::Utc::now()
            .checked_sub_signed(chrono::Duration::days(timeframe_days as i64))
            .map(|dt| dt.to_rfc3339());
            
        let (readings, _) = self.get_filtered(
            start_date,
            None,
            None,
            None,
            Some(false) // oldest first
        ).await?;
        
        if readings.is_empty() {
            return Ok(None);
        }
        
        // Calculate averages
        let count = readings.len();
        let mut sum_systolic = 0.0;
        let mut sum_diastolic = 0.0;
        let mut sum_pulse = 0.0;
        let mut pulse_count = 0;
        
        let mut max_systolic = i32::MIN;
        let mut max_diastolic = i32::MIN;
        let mut min_systolic = i32::MAX;
        let mut min_diastolic = i32::MAX;
        
        for reading in &readings {
            sum_systolic += reading.systolic as f64;
            sum_diastolic += reading.diastolic as f64;
            
            if let Some(pulse) = reading.pulse {
                sum_pulse += pulse as f64;
                pulse_count += 1;
            }
            
            max_systolic = max_systolic.max(reading.systolic as i32);
            max_diastolic = max_diastolic.max(reading.diastolic as i32);
            min_systolic = min_systolic.min(reading.systolic as i32);
            min_diastolic = min_diastolic.min(reading.diastolic as i32);
        }
        
        let avg_systolic = sum_systolic / count as f64;
        let avg_diastolic = sum_diastolic / count as f64;
        let avg_pulse = if pulse_count > 0 {
            Some(sum_pulse / pulse_count as f64)
        } else {
            None
        };
        
        // Determine category based on average readings
        let category_str = if avg_systolic >= 180.0 || avg_diastolic >= 120.0 {
            "HypertensiveCrisis"
        } else if avg_systolic >= 140.0 || avg_diastolic >= 90.0 {
            "Hypertension2"
        } else if avg_systolic >= 130.0 || avg_diastolic >= 80.0 {
            "Hypertension1"
        } else if avg_systolic >= 120.0 && avg_diastolic < 80.0 {
            "Elevated"
        } else {
            "Normal"
        };
        
        let insights = BloodPressureInsights {
            avg_systolic,
            avg_diastolic,
            avg_pulse,
            max_systolic,
            max_diastolic,
            min_systolic,
            min_diastolic,
            category: category_str.to_string(),
            reading_count: count,
            period_days: timeframe_days,
            generated_at: Utc::now(),
        };
        
        Ok(Some(insights))
    }
}

/// Mock blood pressure repository for testing
#[cfg(any(test, feature = "mock"))]
pub mod tests {
    use super::*;
    
    /// Mock implementation of BloodPressureRepository for testing
    pub struct MockBloodPressureRepository {
        readings: Vec<BloodPressureReading>,
    }
    
    impl Default for MockBloodPressureRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockBloodPressureRepository {
        /// Create a new empty mock repository
        pub fn new() -> Self {
            Self { readings: Vec::new() }
        }
        
        /// Create a mock repository with predefined readings
        pub fn with_readings(readings: Vec<BloodPressureReading>) -> Self {
            Self { readings }
        }
    }
    
    #[async_trait]
    impl BloodPressureRepositoryTrait for MockBloodPressureRepository {
        async fn create(&self, request: CreateBloodPressureRequest) -> Result<BloodPressureReading, RepositoryError> {
            let reading = BloodPressureReading {
                id: Uuid::new_v4().to_string(),
                systolic: request.systolic,
                diastolic: request.diastolic,
                pulse: request.pulse,
                notes: request.notes,
                timestamp: request.timestamp,
                position: request.position,
                arm: request.arm,
                device_id: request.device_id,
            };
            
            Ok(reading)
        }
        
        async fn get_all(&self) -> Result<Vec<BloodPressureReading>, RepositoryError> {
            Ok(self.readings.clone())
        }
        
        async fn get_latest(&self) -> Result<Option<BloodPressureReading>, RepositoryError> {
            let latest = self.readings.iter()
                .max_by(|a, b| a.timestamp.cmp(&b.timestamp))
                .cloned();
                
            Ok(latest)
        }
        
        async fn get_by_id(&self, id: Uuid) -> Result<Option<BloodPressureReading>, RepositoryError> {
            let reading = self.readings.iter()
                .find(|r| r.id == id.to_string())
                .cloned();
                
            Ok(reading)
        }
        
        async fn get_filtered(
            &self,
            start_date: Option<String>,
            end_date: Option<String>,
            limit: Option<usize>,
            offset: Option<usize>,
            sort_desc: Option<bool>,
        ) -> Result<(Vec<BloodPressureReading>, usize), RepositoryError> {
            let offset = offset.unwrap_or(0);
            let limit = limit.unwrap_or(usize::MAX);
            let sort_desc = sort_desc.unwrap_or(true);
            
            let mut filtered: Vec<BloodPressureReading> = self.readings.iter()
                .filter(|reading| {
                    if let Some(start) = &start_date {
                        if reading.timestamp < *start {
                            return false;
                        }
                    }
                    
                    if let Some(end) = &end_date {
                        if reading.timestamp > *end {
                            return false;
                        }
                    }
                    
                    true
                })
                .cloned()
                .collect();
                
            // Sort
            filtered.sort_by(|a, b| {
                let cmp = a.timestamp.cmp(&b.timestamp);
                if sort_desc {
                    cmp.reverse()
                } else {
                    cmp
                }
            });
            
            let total = filtered.len();
            
            // Apply pagination
            let paged = filtered
                .into_iter()
                .skip(offset)
                .take(limit)
                .collect();
                
            Ok((paged, total))
        }
        
        async fn generate_insights(&self, timeframe_days: u32) -> Result<Option<BloodPressureInsights>, RepositoryError> {
            if self.readings.is_empty() {
                return Ok(None);
            }
            
            // Simple mock implementation
            Ok(Some(BloodPressureInsights {
                avg_systolic: 120.0,
                avg_diastolic: 80.0,
                avg_pulse: Some(72.0),
                max_systolic: 130,
                max_diastolic: 85,
                min_systolic: 110,
                min_diastolic: 75,
                category: "Normal".to_string(),
                reading_count: self.readings.len(),
                period_days: timeframe_days,
                generated_at: Utc::now(),
            }))
        }
    }
} 