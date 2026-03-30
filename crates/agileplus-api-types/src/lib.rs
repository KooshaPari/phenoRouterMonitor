//! Shared API types for AgilePlus.

use agileplus_error_core::ApiError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            data: None,
            error: Some(msg.into()),
        }
    }

    /// Build an error envelope from the shared [`ApiError`] type (stable message text).
    pub fn from_api_error(err: ApiError) -> Self {
        Self {
            data: None,
            error: Some(err.to_string()),
        }
    }

    /// Map any value that converts to the canonical API error type.
    pub fn from_api_error_ref(err: &ApiError) -> Self {
        Self::from_api_error(err.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-AGILE-001
    #[test]
    fn success_response_has_data_no_error() {
        let resp = ApiResponse::success(42);
        assert_eq!(resp.data, Some(42));
        assert!(resp.error.is_none());
    }

    // Traces to: FR-AGILE-001
    #[test]
    fn error_response_has_error_no_data() {
        let resp = ApiResponse::<i32>::error("not found");
        assert!(resp.data.is_none());
        assert_eq!(resp.error.as_deref(), Some("not found"));
    }

    // Traces to: FR-AGILE-001
    #[test]
    fn from_api_error_matches_display() {
        let e = ApiError::NotFound("feature/x".into());
        let resp = ApiResponse::<()>::from_api_error(e);
        assert!(resp.data.is_none());
        assert_eq!(
            resp.error.as_deref(),
            Some("not found: feature/x")
        );
    }

    // Traces to: FR-AGILE-001
    #[test]
    fn response_debug_impl() {
        let resp = ApiResponse::success("hello");
        let dbg = format!("{:?}", resp);
        assert!(dbg.contains("hello"));
    }

    // Traces to: FR-AGILE-001
    #[test]
    fn response_clone() {
        let resp = ApiResponse::success(vec![1, 2, 3]);
        let cloned = resp.clone();
        assert_eq!(cloned.data, Some(vec![1, 2, 3]));
    }

    // Traces to: FR-AGILE-001
    #[test]
    fn response_serialization_roundtrip() {
        let resp = ApiResponse::success("test".to_string());
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: ApiResponse<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.data, Some("test".to_string()));
        assert!(deserialized.error.is_none());
    }

    // Traces to: FR-AGILE-001
    #[test]
    fn error_response_serialization_roundtrip() {
        let resp = ApiResponse::<String>::error("fail");
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: ApiResponse<String> = serde_json::from_str(&json).unwrap();
        assert!(deserialized.data.is_none());
        assert_eq!(deserialized.error.as_deref(), Some("fail"));
    }
}
