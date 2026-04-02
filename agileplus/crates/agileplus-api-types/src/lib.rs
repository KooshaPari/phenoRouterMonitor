//! Shared API types for AgilePlus.
//!
//! Provides DTOs for API contracts, request/response types, and error handling.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Generic Response Wrapper
// ============================================================================

/// Generic API response envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Response payload (if successful).
    pub data: Option<T>,
    /// Error message (if failed).
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Create a successful response.
    pub fn success(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            data: None,
            error: Some(message.into()),
        }
    }

    /// Check if the response represents success.
    pub fn is_success(&self) -> bool {
        self.data.is_some() && self.error.is_none()
    }
}

// ============================================================================
// Pagination
// ============================================================================

/// Request parameters for paginated endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationRequest {
    /// Page number (1-indexed).
    pub page: u32,
    /// Items per page.
    pub per_page: u32,
}

impl Default for PaginationRequest {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
        }
    }
}

/// Paginated response metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    /// Current page number.
    pub page: u32,
    /// Items per page.
    pub per_page: u32,
    /// Total items available.
    pub total: u64,
    /// Total pages available.
    pub total_pages: u32,
}

impl PaginationMeta {
    /// Create pagination metadata.
    pub fn new(page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = if total == 0 {
            1
        } else {
            total.div_ceil(per_page as u64) as u32
        };

        Self {
            page,
            per_page,
            total,
            total_pages,
        }
    }
}

/// Generic paginated response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// Page data.
    pub items: Vec<T>,
    /// Pagination metadata.
    pub pagination: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    /// Create a paginated response.
    pub fn new(items: Vec<T>, page: u32, per_page: u32, total: u64) -> Self {
        Self {
            items,
            pagination: PaginationMeta::new(page, per_page, total),
        }
    }
}

// ============================================================================
// Project DTOs
// ============================================================================

/// Request to create a new project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: String,
}

/// Response containing a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Work Item DTOs
// ============================================================================

/// Request to create a work item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkItemRequest {
    pub title: String,
    pub description: String,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
}

/// Request to update a work item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkItemRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
}

/// Response containing a work item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItemResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub assignee: Option<String>,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Sprint DTOs
// ============================================================================

/// Request to create a sprint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSprintRequest {
    pub name: String,
    pub goal: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

/// Response containing a sprint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub goal: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

// ============================================================================
// Error Response
// ============================================================================

/// Error response with code and message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_response_success() {
        let resp: ApiResponse<String> = ApiResponse::success("test".into());
        assert!(resp.is_success());
        assert_eq!(resp.data, Some("test".into()));
    }

    #[test]
    fn api_response_error() {
        let resp: ApiResponse<String> = ApiResponse::error("failure");
        assert!(!resp.is_success());
        assert_eq!(resp.error, Some("failure".into()));
    }

    #[test]
    fn pagination_meta() {
        let meta = PaginationMeta::new(1, 20, 100);
        assert_eq!(meta.total_pages, 5);
        assert_eq!(meta.page, 1);
    }

    #[test]
    fn pagination_meta_single_page() {
        let meta = PaginationMeta::new(1, 50, 10);
        assert_eq!(meta.total_pages, 1);
    }

    #[test]
    fn pagination_meta_empty() {
        let meta = PaginationMeta::new(1, 20, 0);
        assert_eq!(meta.total_pages, 1);
    }

    #[test]
    fn paginated_response() {
        let items = vec![1, 2, 3];
        let resp = PaginatedResponse::new(items.clone(), 1, 10, 3);
        assert_eq!(resp.items, items);
        assert_eq!(resp.pagination.total, 3);
    }

    #[test]
    fn create_project_request() {
        let req = CreateProjectRequest {
            name: "Test".into(),
            description: "A test".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("Test"));
    }

    #[test]
    fn error_response() {
        let err = ErrorResponse::new("NOT_FOUND", "Project not found");
        assert_eq!(err.code, "NOT_FOUND");
        assert_eq!(err.message, "Project not found");
    }
}
