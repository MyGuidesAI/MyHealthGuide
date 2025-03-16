use std::sync::Arc;
use axum::{
    extract::{Json, Query, State, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info, instrument, warn};
use chrono::Utc;
use uuid::Uuid;
use utoipa::{IntoParams, ToSchema};

// Import domain entities and services
use my_health_guide_domain::services::{BloodPressureServiceTrait, create_default_blood_pressure_service};
use my_health_guide_domain::entities::blood_pressure::BloodPressureReading as DomainBloodPressureReading;

// Import our entities
use crate::entities::blood_pressure::{BloodPressureReading, CreateBloodPressureRequest};

/// Query parameters for retrieving blood pressure history
#[derive(Debug, Deserialize, Clone, IntoParams, ToSchema)]
pub struct HistoryQueryParams {
    /// ISO 8601 start date (default: 30 days ago)
    pub start_date: Option<String>,

    /// ISO 8601 end date (default: current date)
    pub end_date: Option<String>,

    /// Maximum number of results (default: 100, max: 1000)
    pub limit: Option<usize>,

    /// Pagination offset (default: 0)
    pub offset: Option<usize>,

    /// Sort direction (asc/desc, default: desc)
    pub sort: Option<String>,
}

/// Query parameters for retrieving blood pressure insights
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct InsightsQueryParams {
    /// Analysis period in days (default: 30, max: 365)
    pub timeframe: Option<u32>,
}

/// Paginated response for blood pressure data
#[derive(Serialize, ToSchema)]
#[aliases(BloodPressurePaginatedResponse = PaginatedResponse<BloodPressureReading>)]
pub struct PaginatedResponse<T> {
    /// Total count of items available
    pub total_count: usize,

    /// Current offset
    pub offset: usize,

    /// Current limit
    pub limit: usize,

    /// URL for the next page (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,

    /// URL for the previous page (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous: Option<String>,

    /// Actual data items
    pub data: Vec<T>,
}

/// Error response format for API
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Error type/code - machine-readable identifier
    pub error: String,

    /// Human-readable error message
    pub message: String,

    /// Optional additional details about the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    /// Create a not found error response
    pub fn not_found(resource: &str) -> Self {
        Self {
            error: "not_found".to_string(),
            message: format!("The requested {} could not be found", resource),
            details: None,
        }
    }

    /// Create a validation error response
    pub fn validation_error(message: &str, details: Option<serde_json::Value>) -> Self {
        Self {
            error: "validation_error".to_string(),
            message: message.to_string(),
            details,
        }
    }

    /// Create a bad request error response
    pub fn bad_request(message: &str) -> Self {
        Self {
            error: "bad_request".to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    /// Create an internal error response
    pub fn internal_error() -> Self {
        Self {
            error: "internal_error".to_string(),
            message: "An unexpected error occurred".to_string(),
            details: None,
        }
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let status = match self.error.as_str() {
            "not_found" => StatusCode::NOT_FOUND,
            "validation_error" => StatusCode::BAD_REQUEST,
            "bad_request" => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(self)).into_response()
    }
}

/// Service type for dependency injection
pub type BloodPressureService = Arc<dyn BloodPressureServiceTrait + Send + Sync>;

/// Create a default service for the handlers to use
pub fn create_service() -> BloodPressureService {
    Arc::new(create_default_blood_pressure_service())
}

/// Get a single blood pressure reading by ID
#[utoipa::path(
    get,
    path = "/api/v1/bloodpressure/{id}",
    params(
        ("id" = String, Path, description = "Blood pressure reading ID")
    ),
    responses(
        (status = 200, description = "Blood pressure reading found", body = BloodPressureReading),
        (status = 404, description = "Blood pressure reading not found", body = PublicErrorResponse),
        (status = 500, description = "Internal server error", body = PublicErrorResponse),
    ),
    security(
        ("bearer" = [])
    ),
    tag = "blood_pressure"
)]
#[instrument(skip(service))]
pub async fn get_blood_pressure(
    State(service): State<BloodPressureService>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, Response> {
    info!("Fetching blood pressure reading with ID: {}", id);

    // Call domain service
    match service.get_reading_by_id(&id.to_string()).await {
        Ok(reading) => {
            // Convert domain entity to public entity
            let public_reading = convert_to_public_reading(reading);
            Ok((StatusCode::OK, Json(public_reading)))
        },
        Err(e) => {
            let error_message = e.to_string();
            if error_message.contains("not found") {
                info!("Blood pressure reading not found: {}", id);
                Err((StatusCode::NOT_FOUND, Json(ErrorResponse::not_found("blood pressure reading"))).into_response())
            } else {
                error!("Error retrieving blood pressure reading: {}", error_message);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::internal_error())).into_response())
            }
        }
    }
}

/// Create a new blood pressure reading
#[utoipa::path(
    post,
    path = "/api/v1/bloodpressure",
    request_body = CreateBloodPressureRequest,
    responses(
        (status = 201, description = "Blood pressure reading created", body = BloodPressureReading),
        (status = 400, description = "Invalid request", body = PublicErrorResponse),
        (status = 500, description = "Internal server error", body = PublicErrorResponse),
    ),
    security(
        ("bearer" = [])
    ),
    tag = "blood_pressure"
)]
#[instrument(skip(service, request))]
pub async fn create_blood_pressure(
    State(service): State<BloodPressureService>,
    Json(request): Json<CreateBloodPressureRequest>,
) -> Result<impl IntoResponse, Response> {
    info!("Creating new blood pressure reading");

    // Convert public request to domain request
    let domain_request = convert_to_domain_request(request);

    // Call domain service
    match service.create_reading(domain_request).await {
        Ok(reading) => {
            info!("Blood pressure reading created with ID: {}", reading.id);
            // Convert domain entity to public entity for API response
            let public_reading = convert_to_public_reading(reading);
            Ok((StatusCode::CREATED, Json(public_reading)))
        },
        Err(e) => {
            let error_message = e.to_string();
            if error_message.contains("Validation") {
                warn!("Invalid blood pressure reading data: {}", error_message);
                Err((StatusCode::BAD_REQUEST, Json(ErrorResponse::validation_error(&error_message, None))).into_response())
            } else {
                error!("Error creating blood pressure reading: {}", error_message);
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::internal_error())).into_response())
            }
        }
    }
}

/// Generate pagination links from the current request
fn generate_pagination_links(
    total_count: usize,
    limit: usize,
    offset: usize,
    base_url: &str,
    query_params: &HistoryQueryParams,
) -> (Option<String>, Option<String>) {
    let has_next = offset + limit < total_count;
    let has_prev = offset > 0;

    // Build query string
    let mut next_params = query_params.clone();
    let mut prev_params = query_params.clone();

    // For next link
    let next = if has_next {
        next_params.offset = Some(offset + limit);
        next_params.limit = Some(limit);

        let mut query_parts = Vec::new();

        if let Some(start) = &next_params.start_date {
            query_parts.push(format!("start_date={}", start));
        }

        if let Some(end) = &next_params.end_date {
            query_parts.push(format!("end_date={}", end));
        }

        if let Some(limit) = next_params.limit {
            query_parts.push(format!("limit={}", limit));
        }

        if let Some(offset) = next_params.offset {
            query_parts.push(format!("offset={}", offset));
        }

        if let Some(sort) = &next_params.sort {
            query_parts.push(format!("sort={}", sort));
        }

        let query_string = if query_parts.is_empty() {
            String::new()
        } else {
            format!("?{}", query_parts.join("&"))
        };

        Some(format!("{}{}", base_url, query_string))
    } else {
        None
    };

    // For previous link
    let previous = if has_prev {
        let new_offset = offset.saturating_sub(limit);

        prev_params.offset = Some(new_offset);
        prev_params.limit = Some(limit);

        let mut query_parts = Vec::new();

        if let Some(start) = &prev_params.start_date {
            query_parts.push(format!("start_date={}", start));
        }

        if let Some(end) = &prev_params.end_date {
            query_parts.push(format!("end_date={}", end));
        }

        if let Some(limit) = prev_params.limit {
            query_parts.push(format!("limit={}", limit));
        }

        if let Some(offset) = prev_params.offset {
            query_parts.push(format!("offset={}", offset));
        }

        if let Some(sort) = &prev_params.sort {
            query_parts.push(format!("sort={}", sort));
        }

        let query_string = if query_parts.is_empty() {
            String::new()
        } else {
            format!("?{}", query_parts.join("&"))
        };

        Some(format!("{}{}", base_url, query_string))
    } else {
        None
    };

    (next, previous)
}

/// Get paginated blood pressure history
#[utoipa::path(
    get,
    path = "/api/v1/bloodpressure",
    params(
        HistoryQueryParams
    ),
    responses(
        (status = 200, description = "Blood pressure history retrieved", body = BloodPressurePaginatedResponse),
        (status = 500, description = "Internal server error", body = PublicErrorResponse),
    ),
    security(
        ("bearer" = [])
    ),
    tag = "blood_pressure"
)]
#[instrument(skip(service))]
pub async fn get_blood_pressure_history(
    State(service): State<BloodPressureService>,
    Query(params): Query<HistoryQueryParams>,
) -> Result<impl IntoResponse, Response> {
    // Process query parameters
    let limit = params.limit.unwrap_or(100).min(1000); // Cap at 1000
    let offset = params.offset.unwrap_or(0);

    // Default to sorting by most recent if not specified
    let sort_desc = match params.sort.as_deref() {
        Some("asc") => false,
        _ => true, // Default to descending (newest first)
    };

    // Parse date range
    let now = Utc::now();
    let thirty_days_ago = now - chrono::Duration::days(30);

    let start_date = if let Some(ref date_str) = params.start_date {
        match chrono::DateTime::parse_from_rfc3339(date_str) {
            Ok(date) => date.with_timezone(&Utc),
            Err(_) => {
                let error = ErrorResponse::bad_request("Invalid start_date format. Use ISO 8601 (e.g. 2023-03-15T08:30:00Z)");
                return Err((StatusCode::BAD_REQUEST, Json(error)).into_response());
            }
        }
    } else {
        thirty_days_ago
    };

    let end_date = if let Some(ref date_str) = params.end_date {
        match chrono::DateTime::parse_from_rfc3339(date_str) {
            Ok(date) => date.with_timezone(&Utc),
            Err(_) => {
                let error = ErrorResponse::bad_request("Invalid end_date format. Use ISO 8601 (e.g. 2023-03-15T08:30:00Z)");
                return Err((StatusCode::BAD_REQUEST, Json(error)).into_response());
            }
        }
    } else {
        now
    };

    // Convert dates to strings for filtering
    let start_date_str = Some(start_date.to_rfc3339());
    let end_date_str = Some(end_date.to_rfc3339());

    // Call domain service
    match service.get_filtered_readings(start_date_str, end_date_str, Some(limit), Some(offset), Some(sort_desc)).await {
        Ok((domain_readings, total_count)) => {
            // Base URL for pagination links
            let base_url = "/api/v1/bloodpressure";

            // Generate pagination links
            let (next, previous) = generate_pagination_links(
                total_count,
                limit,
                offset,
                base_url,
                &params,
            );

            // Convert the domain readings to public readings
            let public_readings = domain_readings.into_iter()
                .map(convert_to_public_reading)
                .collect();

            // Create paginated response
            let response = PaginatedResponse {
                total_count,
                offset,
                limit,
                next,
                previous,
                data: public_readings,
            };

            Ok((StatusCode::OK, Json(response)))
        },
        Err(e) => {
            error!("Failed to get blood pressure history: {}", e);
            let error = ErrorResponse::internal_error();
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response())
        }
    }
}

/// Get blood pressure insights and analysis
#[utoipa::path(
    get,
    path = "/api/v1/bloodpressure/insights",
    responses(
        (status = 200, description = "Blood pressure insights generated", body = BloodPressureReading),
        (status = 500, description = "Internal server error", body = PublicErrorResponse),
    ),
    security(
        ("bearer" = [])
    ),
    tag = "blood_pressure"
)]
#[instrument(skip(service))]
pub async fn get_blood_pressure_insights(
    State(service): State<BloodPressureService>,
    Query(params): Query<InsightsQueryParams>,
) -> Result<impl IntoResponse, Response> {
    // Process query parameters
    let timeframe = params.timeframe.unwrap_or(30).min(365); // Default to 30 days, max 1 year

    info!("Generating blood pressure insights for {} days", timeframe);

    // Get all readings for the specified timeframe
    let now = Utc::now();
    let start_date = now - chrono::Duration::days(timeframe as i64);
    let start_date_str = Some(start_date.to_rfc3339());
    let end_date_str = Some(now.to_rfc3339());

    // Get readings within timeframe
    match service.get_filtered_readings(start_date_str, end_date_str, None, None, None).await {
        Ok((domain_readings, _)) => {
            // Calculate insights
            match service.calculate_insights(&domain_readings, timeframe) {
                Ok(insights) => {
                    info!("Blood pressure insights generated successfully");
                    Ok((StatusCode::OK, Json(insights)).into_response())
                },
                Err(e) => {
                    let error_message = e.to_string();
                    if error_message.contains("insufficient") {
                        info!("Insufficient data for insights");
                        Ok((
                            StatusCode::NOT_FOUND,
                            Json(ErrorResponse {
                                error: "insufficient_data".to_string(),
                                message: "Not enough data to generate insights".to_string(),
                                details: None,
                            }),
                        ).into_response())
                    } else {
                        error!("Error generating blood pressure insights: {}", e);
                        Ok((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: "internal_server_error".to_string(),
                                message: "Failed to generate blood pressure insights".to_string(),
                                details: None,
                            }),
                        ).into_response())
                    }
                }
            }
        },
        Err(e) => {
            error!("Failed to retrieve blood pressure readings: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::internal_error())).into_response())
        }
    }
}

// Convert public request to domain request
fn convert_to_domain_request(request: CreateBloodPressureRequest) -> my_health_guide_domain::entities::blood_pressure::CreateBloodPressureRequest {
    let timestamp = request.timestamp
        .map_or_else(|| Utc::now().to_rfc3339(), |dt| dt.to_rfc3339());

    my_health_guide_domain::entities::blood_pressure::CreateBloodPressureRequest {
        systolic: request.systolic as u16,
        diastolic: request.diastolic as u16,
        pulse: request.pulse.map(|p| p as u16),
        notes: request.notes,
        timestamp,
        position: None,
        arm: None,
        device_id: None,
    }
}

// Convert domain reading to public reading
fn convert_to_public_reading(reading: DomainBloodPressureReading) -> crate::entities::blood_pressure::BloodPressureReading {
    let timestamp = match chrono::DateTime::parse_from_rfc3339(&reading.timestamp) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(_) => chrono::Utc::now(), // Fallback to current time if parsing fails
    };

    crate::entities::blood_pressure::BloodPressureReading {
        id: uuid::Uuid::parse_str(&reading.id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
        systolic: reading.systolic as i32,
        diastolic: reading.diastolic as i32,
        pulse: reading.pulse.map(|p| p as i32),
        notes: reading.notes,
        recorded_at: timestamp,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_link_generation() {
        let query_params = HistoryQueryParams {
            start_date: Some("2023-01-01T00:00:00Z".to_string()),
            end_date: Some("2023-02-01T00:00:00Z".to_string()),
            limit: Some(10),
            offset: Some(20),
            sort: Some("desc".to_string()),
        };

        // Test with more results available
        let (next, prev) = generate_pagination_links(50, 10, 20, "/api/v1/bloodpressure", &query_params);

        assert!(next.is_some());
        assert!(prev.is_some());

        let next_url = next.unwrap();
        let prev_url = prev.unwrap();

        assert!(next_url.contains("offset=30"));
        assert!(prev_url.contains("offset=10"));

        // Test boundary conditions

        // First page
        let (next, prev) = generate_pagination_links(50, 10, 0, "/api/v1/bloodpressure", &query_params);
        assert!(next.is_some());
        assert!(prev.is_none()); // No previous page

        // Last page
        let (next, prev) = generate_pagination_links(50, 10, 40, "/api/v1/bloodpressure", &query_params);
        assert!(next.is_none()); // No next page
        assert!(prev.is_some());
    }
}
