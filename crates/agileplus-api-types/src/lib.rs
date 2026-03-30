//! Shared API types for AgilePlus.
//!
//! This crate provides shared JSON DTOs and response envelopes for AgilePlus HTTP APIs.
//! It includes response wrappers, error handling, pagination utilities, and type-safe serialization.

use agileplus_error_core::ApiError;
use serde::{Deserialize, Serialize};

/// Generic API response wrapper for all HTTP endpoints.
///
/// Provides a consistent envelope for successful data responses and error cases.
/// Traces to: FR-PHENO-API-001
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiResponse<T> {
    /// The successful response payload (mutually exclusive with `error`).
    pub data: Option<T>,
    /// The error message if request failed (mutually exclusive with `data`).
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Wrap a successful value in an `ApiResponse`.
    ///
    /// Traces to: FR-PHENO-API-001
    pub fn success(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
        }
    }

    /// Wrap an error message in an `ApiResponse`.
    ///
    /// Traces to: FR-PHENO-API-001
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            data: None,
            error: Some(msg.into()),
        }
    }

    /// Build an error envelope from the shared [`ApiError`] type (stable message text).
    ///
    /// Traces to: FR-PHENO-API-002
    pub fn from_api_error(err: ApiError) -> Self {
        Self {
            data: None,
            error: Some(err.to_string()),
        }
    }

    /// Map any value that converts to the canonical API error type.
    ///
    /// Traces to: FR-PHENO-API-002
    pub fn from_api_error_ref(err: &ApiError) -> Self {
        Self::from_api_error(err.clone())
    }

    /// Check if the response represents success (has data).
    ///
    /// Traces to: FR-PHENO-API-001
    pub fn is_success(&self) -> bool {
        self.data.is_some() && self.error.is_none()
    }

    /// Check if the response represents an error.
    ///
    /// Traces to: FR-PHENO-API-002
    pub fn is_error(&self) -> bool {
        self.error.is_some() && self.data.is_none()
    }
}

/// Pagination metadata for paginated API responses.
///
/// Traces to: FR-PHENO-API-003
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pagination {
    /// Zero-based offset of the first item in the current page.
    pub offset: u64,
    /// Maximum number of items returned per page.
    pub limit: u64,
    /// Total number of items available.
    pub total: u64,
    /// Whether more items are available after the current page.
    pub has_more: bool,
}

impl Pagination {
    /// Create a new pagination descriptor.
    ///
    /// Traces to: FR-PHENO-API-003
    pub fn new(offset: u64, limit: u64, total: u64) -> Self {
        let has_more = offset + limit < total;
        Self {
            offset,
            limit,
            total,
            has_more,
        }
    }

    /// Get the current page number (1-indexed).
    ///
    /// Traces to: FR-PHENO-API-003
    pub fn page_number(&self) -> u64 {
        (self.offset / self.limit) + 1
    }

    /// Get the total number of pages.
    ///
    /// Traces to: FR-PHENO-API-003
    pub fn total_pages(&self) -> u64 {
        (self.total + self.limit - 1) / self.limit
    }

    /// Calculate the offset for the next page.
    ///
    /// Returns `None` if already on the last page.
    ///
    /// Traces to: FR-PHENO-API-003
    pub fn next_offset(&self) -> Option<u64> {
        if self.has_more {
            Some(self.offset + self.limit)
        } else {
            None
        }
    }
}

/// Paginated API response with metadata.
///
/// Traces to: FR-PHENO-API-004
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaginatedResponse<T> {
    /// The list of items in this page.
    pub items: Vec<T>,
    /// Pagination metadata.
    pub pagination: Pagination,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response.
    ///
    /// Traces to: FR-PHENO-API-004
    pub fn new(items: Vec<T>, offset: u64, limit: u64, total: u64) -> Self {
        Self {
            items,
            pagination: Pagination::new(offset, limit, total),
        }
    }

    /// Check if this page has items.
    ///
    /// Traces to: FR-PHENO-API-004
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the count of items in this page.
    ///
    /// Traces to: FR-PHENO-API-004
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // ApiResponse Tests
    // ============================================================================

    /// Traces to: FR-PHENO-API-001
    #[test]
    fn success_response_has_data_no_error() {
        let resp = ApiResponse::success(42);
        assert_eq!(resp.data, Some(42));
        assert!(resp.error.is_none());
    }

    /// Traces to: FR-PHENO-API-001
    #[test]
    fn error_response_has_error_no_data() {
        let resp = ApiResponse::<i32>::error("not found");
        assert!(resp.data.is_none());
        assert_eq!(resp.error.as_deref(), Some("not found"));
    }

    /// Traces to: FR-PHENO-API-001
    #[test]
    fn is_success_returns_true_for_data_responses() {
        let resp = ApiResponse::success(42);
        assert!(resp.is_success());
        assert!(!resp.is_error());
    }

    /// Traces to: FR-PHENO-API-001
    #[test]
    fn is_error_returns_true_for_error_responses() {
        let resp = ApiResponse::<i32>::error("failed");
        assert!(resp.is_error());
        assert!(!resp.is_success());
    }

    /// Traces to: FR-PHENO-API-001
    #[test]
    fn response_debug_impl() {
        let resp = ApiResponse::success("hello");
        let dbg = format!("{:?}", resp);
        assert!(dbg.contains("hello"));
    }

    /// Traces to: FR-PHENO-API-001
    #[test]
    fn response_clone() {
        let resp = ApiResponse::success(vec![1, 2, 3]);
        let cloned = resp.clone();
        assert_eq!(cloned.data, Some(vec![1, 2, 3]));
    }

    /// Traces to: FR-PHENO-API-001
    #[test]
    fn response_equality() {
        let resp1 = ApiResponse::success(42);
        let resp2 = ApiResponse::success(42);
        assert_eq!(resp1, resp2);
    }

    /// Traces to: FR-PHENO-API-001
    #[test]
    fn response_inequality() {
        let resp1 = ApiResponse::success(42);
        let resp2 = ApiResponse::success(43);
        assert_ne!(resp1, resp2);
    }

    // ============================================================================
    // ApiResponse Serialization Tests
    // ============================================================================

    /// Traces to: FR-PHENO-API-001
    #[test]
    fn response_serialization_roundtrip() {
        let resp = ApiResponse::success("test".to_string());
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: ApiResponse<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.data, Some("test".to_string()));
        assert!(deserialized.error.is_none());
    }

    /// Traces to: FR-PHENO-API-001
    #[test]
    fn error_response_serialization_roundtrip() {
        let resp = ApiResponse::<String>::error("fail");
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: ApiResponse<String> = serde_json::from_str(&json).unwrap();
        assert!(deserialized.data.is_none());
        assert_eq!(deserialized.error.as_deref(), Some("fail"));
    }

    /// Traces to: FR-PHENO-API-001
    #[test]
    fn response_json_with_complex_data() {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        struct TestData {
            id: u32,
            name: String,
        }

        let data = TestData {
            id: 123,
            name: "test".to_string(),
        };
        let resp = ApiResponse::success(data.clone());
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: ApiResponse<TestData> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.data, Some(data));
    }

    // ============================================================================
    // ApiError Integration Tests
    // ============================================================================

    /// Traces to: FR-PHENO-API-002
    #[test]
    fn from_api_error_matches_display() {
        let e = ApiError::NotFound("feature/x".into());
        let resp = ApiResponse::<()>::from_api_error(e);
        assert!(resp.data.is_none());
        assert_eq!(resp.error.as_deref(), Some("not found: feature/x"));
    }

    /// Traces to: FR-PHENO-API-002
    #[test]
    fn from_api_error_ref() {
        let e = ApiError::Conflict("duplicate".into());
        let resp = ApiResponse::<String>::from_api_error_ref(&e);
        assert!(resp.is_error());
        assert!(resp.error.as_ref().unwrap().contains("conflict"));
    }

    /// Traces to: FR-PHENO-API-002
    #[test]
    fn api_error_variant_coverage() {
        let variants = vec![
            (
                ApiError::NotFound("x".into()),
                "not found",
            ),
            (
                ApiError::Conflict("y".into()),
                "conflict",
            ),
            (
                ApiError::Unauthorized,
                "unauthorized",
            ),
        ];

        for (err, expected_keyword) in variants {
            let resp = ApiResponse::<()>::from_api_error(err);
            let error_msg = resp.error.unwrap();
            assert!(
                error_msg.to_lowercase().contains(expected_keyword),
                "Expected '{}' in error message: {}",
                expected_keyword,
                error_msg
            );
        }
    }

    // ============================================================================
    // Pagination Tests
    // ============================================================================

    /// Traces to: FR-PHENO-API-003
    #[test]
    fn pagination_new_calculates_has_more_correctly() {
        let pg = Pagination::new(0, 10, 25);
        assert!(pg.has_more);
        assert_eq!(pg.offset, 0);
        assert_eq!(pg.limit, 10);
        assert_eq!(pg.total, 25);
    }

    /// Traces to: FR-PHENO-API-003
    #[test]
    fn pagination_last_page_has_no_more() {
        let pg = Pagination::new(20, 10, 25);
        assert!(!pg.has_more);
    }

    /// Traces to: FR-PHENO-API-003
    #[test]
    fn pagination_page_number() {
        assert_eq!(Pagination::new(0, 10, 100).page_number(), 1);
        assert_eq!(Pagination::new(10, 10, 100).page_number(), 2);
        assert_eq!(Pagination::new(20, 10, 100).page_number(), 3);
    }

    /// Traces to: FR-PHENO-API-003
    #[test]
    fn pagination_total_pages() {
        assert_eq!(Pagination::new(0, 10, 100).total_pages(), 10);
        assert_eq!(Pagination::new(0, 10, 95).total_pages(), 10);
        assert_eq!(Pagination::new(0, 10, 101).total_pages(), 11);
    }

    /// Traces to: FR-PHENO-API-003
    #[test]
    fn pagination_next_offset() {
        let pg = Pagination::new(0, 10, 25);
        assert_eq!(pg.next_offset(), Some(10));

        let last_pg = Pagination::new(20, 10, 25);
        assert_eq!(last_pg.next_offset(), None);
    }

    /// Traces to: FR-PHENO-API-003
    #[test]
    fn pagination_equality() {
        let pg1 = Pagination::new(0, 10, 100);
        let pg2 = Pagination::new(0, 10, 100);
        assert_eq!(pg1, pg2);
    }

    /// Traces to: FR-PHENO-API-003
    #[test]
    fn pagination_serialization_roundtrip() {
        let pg = Pagination::new(10, 20, 100);
        let json = serde_json::to_string(&pg).unwrap();
        let deserialized: Pagination = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, pg);
    }

    // ============================================================================
    // PaginatedResponse Tests
    // ============================================================================

    /// Traces to: FR-PHENO-API-004
    #[test]
    fn paginated_response_new() {
        let items = vec![1, 2, 3];
        let resp = PaginatedResponse::new(items.clone(), 0, 10, 30);
        assert_eq!(resp.items, items);
        assert_eq!(resp.pagination.offset, 0);
        assert_eq!(resp.pagination.limit, 10);
        assert_eq!(resp.pagination.total, 30);
    }

    /// Traces to: FR-PHENO-API-004
    #[test]
    fn paginated_response_is_empty() {
        let empty_resp: PaginatedResponse<i32> = PaginatedResponse::new(vec![], 0, 10, 0);
        assert!(empty_resp.is_empty());

        let non_empty = PaginatedResponse::new(vec![1, 2], 0, 10, 30);
        assert!(!non_empty.is_empty());
    }

    /// Traces to: FR-PHENO-API-004
    #[test]
    fn paginated_response_len() {
        let resp = PaginatedResponse::new(vec![1, 2, 3, 4, 5], 0, 10, 100);
        assert_eq!(resp.len(), 5);
    }

    /// Traces to: FR-PHENO-API-004
    #[test]
    fn paginated_response_clone() {
        let resp = PaginatedResponse::new(vec!["a", "b"], 5, 10, 50);
        let cloned = resp.clone();
        assert_eq!(cloned.items, vec!["a", "b"]);
        assert_eq!(cloned.pagination.offset, 5);
    }

    /// Traces to: FR-PHENO-API-004
    #[test]
    fn paginated_response_equality() {
        let resp1 = PaginatedResponse::new(vec![1, 2], 0, 10, 100);
        let resp2 = PaginatedResponse::new(vec![1, 2], 0, 10, 100);
        assert_eq!(resp1, resp2);
    }

    /// Traces to: FR-PHENO-API-004
    #[test]
    fn paginated_response_serialization_roundtrip() {
        let resp = PaginatedResponse::new(vec!["x", "y", "z"], 10, 20, 100);
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: PaginatedResponse<&str> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.items, vec!["x", "y", "z"]);
        assert_eq!(deserialized.pagination.offset, 10);
    }

    /// Traces to: FR-PHENO-API-004
    #[test]
    fn paginated_response_with_complex_items() {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        struct Item {
            id: u64,
            name: String,
        }

        let items = vec![
            Item { id: 1, name: "first".into() },
            Item { id: 2, name: "second".into() },
        ];
        let resp = PaginatedResponse::new(items.clone(), 0, 10, 100);

        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: PaginatedResponse<Item> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.items, items);
    }
}
