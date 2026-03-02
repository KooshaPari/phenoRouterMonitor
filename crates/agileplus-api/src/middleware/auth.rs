//! API key authentication middleware.
//!
//! Protected endpoints require the `X-API-Key` header or `?api_key=` query param.
//! Health, info, and webhook endpoints are always accessible without API key auth.
//!
//! Keys are validated via the `CredentialStore`. Constant-time comparison
//! is performed inside `CredentialStore::validate_api_key` to prevent
//! timing attacks. The raw key value is never logged.
//!
//! Traceability: FR-030 / WP11-T065

use axum::extract::Request;
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use tracing::warn;

use agileplus_domain::credentials::CredentialStore;

use crate::error::ApiError;

/// Paths that do not require authentication.
const PUBLIC_PATHS: &[&str] = &["/health", "/info", "/webhooks"];

/// axum middleware that validates the `X-API-Key` header (or `?api_key=` query
/// param) for all non-public endpoints.
///
/// Returns `401 Unauthorized` if the header/param is missing or the key is invalid.
pub async fn validate_api_key(
    axum::extract::State(creds): axum::extract::State<std::sync::Arc<dyn CredentialStore>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let path = request.uri().path().to_string();

    // Always allow public endpoints (health, info, webhooks).
    if PUBLIC_PATHS.iter().any(|p| path.starts_with(p)) {
        return Ok(next.run(request).await);
    }

    // Try X-API-Key header first, then fall back to ?api_key= query param.
    let api_key = if let Some(header_val) = headers.get("X-API-Key").and_then(|v| v.to_str().ok()) {
        header_val.to_string()
    } else if let Some(query) = request.uri().query() {
        // Parse api_key= from the query string.
        query
            .split('&')
            .find_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                let k = parts.next()?;
                let v = parts.next()?;
                if k == "api_key" { Some(v.to_string()) } else { None }
            })
            .ok_or_else(|| ApiError::Unauthorized("Missing API key (X-API-Key header or ?api_key= param required)".to_string()))?
    } else {
        return Err(ApiError::Unauthorized(
            "Missing API key (X-API-Key header or ?api_key= param required)".to_string(),
        ));
    };

    let valid = creds
        .validate_api_key(&api_key)
        .await
        .map_err(|e| ApiError::Internal(format!("credential store error: {e}")))?;

    if !valid {
        // Log only a truncated hint for identification — never the raw key.
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
