use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Standardized error response format
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PublicErrorResponse {
    /// Error message
    pub message: String,
    
    /// Optional error code for client-side handling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    
    /// Optional details about the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<String>>,
}

/// Query parameters for paginated requests
#[derive(Debug, Deserialize, ToSchema)]
pub struct PublicPaginationParams {
    /// Number of results to return (default: 20)
    #[schema(default = 20, minimum = 1, maximum = 100)]
    pub limit: Option<usize>,
    
    /// Number of results to skip (default: 0)
    #[schema(default = 0, minimum = 0)]
    pub offset: Option<usize>,
    
    /// Field to sort by
    pub sort_by: Option<String>,
    
    /// Sort direction (asc or desc)
    pub sort_dir: Option<String>,
}

/// Paginated response format
#[derive(Debug, Serialize, ToSchema)]
pub struct PublicPaginatedResponse<T> {
    /// The data items for this page
    pub data: Vec<T>,
    
    /// Total number of items
    pub total: usize,
    
    /// Number of items returned
    pub count: usize,
    
    /// Number of items to skip
    pub offset: usize,
    
    /// Number of items per page
    pub limit: usize,
    
    /// URL for the next page, if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    
    /// URL for the previous page, if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous: Option<String>,
} 