//! API key authentication middleware.
//!
//! Protected endpoints require the `X-API-Key` header.
//! Health and info endpoints are always accessible without authentication.
//!
//! Keys are validated via the `CredentialStore`. Constant-time comparison
//! is performed inside `CredentialStore::validate_api_key` to prevent
//! timing attacks. The raw key value is never logged.
//!
//! Traceability: FR-030 / WP15-T087

use axum::extract::Request;
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use tracing::warn;

use agileplus_domain::credentials::CredentialStore;

use crate::error::ApiError;

/// Paths that do not require authentication.
const PUBLIC_PATHS: &[&str] = &["/health", "/info"];

/// axum middleware that validates the `X-API-Key` header for all non-public endpoints.
///
/// Returns `401 Unauthorized` if the header is missing or the key is invalid.
pub async fn validate_api_key(
    axum::extract::State(creds): axum::extract::State<std::sync::Arc<dyn CredentialStore>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let path = request.uri().path().to_string();

    // Always allow public endpoints.
    if PUBLIC_PATHS.iter().any(|p| path.starts_with(p)) {
        return Ok(next.run(request).await);
    }

    let api_key = headers
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("Missing X-API-Key header".to_string()))?;

    let valid = creds
        .validate_api_key(api_key)
        .await
        .map_err(|e| ApiError::Internal(format!("credential store error: {e}")))?;

    if !valid {
        // Log only a truncated hash for identification — never the raw key.
        let key_hint: String = api_key
            .chars()
            .take(4)
            .chain(['*'; 8])
            .collect();
        warn!(key_hint, "API authentication failed for key hint");
        return Err(ApiError::Unauthorized("Invalid API key".to_string()));
    }

    Ok(next.run(request).await)
}
