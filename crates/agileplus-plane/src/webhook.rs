//! T047: Webhook endpoint — axum handler for Plane.so webhook events.
//!
//! Traceability: WP08-T047

use axum::{
    body::Bytes,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

/// Supported Plane.so webhook event types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaneEventType {
    #[serde(rename = "issue_activity")]
    IssueActivity,
    /// issues:create
    #[serde(other)]
    Unknown,
}

/// Top-level action within a webhook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaneWebhookAction {
    Create,
    Update,
    Delete,
    #[serde(other)]
    Unknown,
}

/// Issue data embedded in a webhook payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaneWebhookIssue {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description_html: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub project: Option<String>,
}

/// Plane.so webhook payload envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaneWebhookPayload {
    pub event: String,
    pub action: PlaneWebhookAction,
    #[serde(default)]
    pub data: Option<PlaneWebhookIssue>,
}

/// Parsed webhook event ready for inbound processing.
#[derive(Debug, Clone)]
pub enum PlaneInboundEvent {
    IssueCreated(PlaneWebhookIssue),
    IssueUpdated(PlaneWebhookIssue),
    IssueDeleted { issue_id: String },
}

/// Verify the HMAC-SHA256 signature from Plane.so webhook headers.
///
/// Plane sends `X-Plane-Signature: sha256=<hex>`.
/// Returns `true` if the signature is valid.
pub fn verify_hmac_signature(secret: &[u8], body: &[u8], header_value: &str) -> bool {
    let hex_sig = match header_value.strip_prefix("sha256=") {
        Some(h) => h,
        None => return false,
    };
    let sig_bytes = match hex::decode(hex_sig) {
        Ok(b) => b,
        Err(_) => {
            // Try without prefix (some implementations send raw hex).
            match hex::decode(header_value) {
                Ok(b) => b,
                Err(_) => return false,
            }
        }
    };

    let mut mac: Hmac<Sha256> =
        Hmac::new_from_slice(secret).expect("HMAC can accept any key length");
    mac.update(body);
    mac.verify_slice(&sig_bytes).is_ok()
}

/// Parse and validate a Plane.so webhook request.
///
/// Returns the structured inbound event or an HTTP error response.
pub fn parse_webhook(
    secret: &[u8],
    headers: &HeaderMap,
    body: &Bytes,
) -> Result<PlaneInboundEvent, (StatusCode, String)> {
    // Signature verification.
    let sig_header = headers
        .get("x-plane-signature")
        .or_else(|| headers.get("X-Plane-Signature"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !secret.is_empty() && !verify_hmac_signature(secret, body, sig_header) {
        return Err((StatusCode::UNAUTHORIZED, "invalid signature".to_string()));
    }

    // Parse JSON body.
    let payload: PlaneWebhookPayload =
        serde_json::from_slice(body).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    // Map to typed event.
    let event = match payload.action {
        PlaneWebhookAction::Create => {
            let issue = payload
                .data
                .ok_or_else(|| (StatusCode::BAD_REQUEST, "missing data field".to_string()))?;
            PlaneInboundEvent::IssueCreated(issue)
        }
        PlaneWebhookAction::Update => {
            let issue = payload
                .data
                .ok_or_else(|| (StatusCode::BAD_REQUEST, "missing data field".to_string()))?;
            PlaneInboundEvent::IssueUpdated(issue)
        }
        PlaneWebhookAction::Delete => {
            let issue = payload
                .data
                .ok_or_else(|| (StatusCode::BAD_REQUEST, "missing data field".to_string()))?;
            PlaneInboundEvent::IssueDeleted {
                issue_id: issue.id,
            }
        }
        PlaneWebhookAction::Unknown => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("unknown action for event {}", payload.event),
            ))
        }
    };

    Ok(event)
}

/// axum handler: `POST /webhooks/plane`
///
/// Verifies signature, parses payload, and returns 200/401/400.
/// Callers should extract the webhook secret from app state.
pub async fn handle_plane_webhook(
    axum::extract::State(secret): axum::extract::State<Vec<u8>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    match parse_webhook(&secret, &headers, &body) {
        Ok(event) => {
            tracing::info!("Received Plane.so webhook event: {:?}", event);
            (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
        }
        Err((code, msg)) => {
            tracing::warn!("Plane.so webhook error {}: {}", code, msg);
            (code, Json(serde_json::json!({"error": msg}))).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Bytes;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    fn make_sig(secret: &[u8], body: &[u8]) -> String {
        let mut mac: Hmac<Sha256> = Hmac::new_from_slice(secret).unwrap();
        mac.update(body);
        let result = mac.finalize().into_bytes();
        format!("sha256={}", hex::encode(result))
    }

    #[test]
    fn valid_signature_accepted() {
        let secret = b"mysecret";
        let body = b"hello";
        let sig = make_sig(secret, body);
        assert!(verify_hmac_signature(secret, body, &sig));
    }

    #[test]
    fn invalid_signature_rejected() {
        assert!(!verify_hmac_signature(b"secret", b"body", "sha256=badhex"));
    }

    #[test]
    fn empty_secret_skips_verification() {
        // With empty secret we still call verify which will accept any
        // but parse_webhook skips if secret is empty.
        let headers = HeaderMap::new();
        let body_str = r#"{"event":"issue","action":"create","data":{"id":"123","name":"Test","labels":[]}}"#;
        let body = Bytes::from(body_str);
        let result = parse_webhook(b"", &headers, &body);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_create_event() {
        let body_str = r#"{"event":"issue","action":"create","data":{"id":"abc","name":"My Issue","labels":[]}}"#;
        let body = Bytes::from(body_str);
        let result = parse_webhook(b"", &HeaderMap::new(), &body).unwrap();
        matches!(result, PlaneInboundEvent::IssueCreated(_));
    }

    #[test]
    fn parse_delete_event() {
        let body_str = r#"{"event":"issue","action":"delete","data":{"id":"abc","name":"X","labels":[]}}"#;
        let body = Bytes::from(body_str);
        let result = parse_webhook(b"", &HeaderMap::new(), &body).unwrap();
        if let PlaneInboundEvent::IssueDeleted { issue_id } = result {
            assert_eq!(issue_id, "abc");
        } else {
            panic!("expected IssueDeleted");
        }
    }

    #[test]
    fn bad_json_returns_400() {
        let body = Bytes::from("not json");
        let result = parse_webhook(b"", &HeaderMap::new(), &body);
        assert!(result.is_err());
        let (code, _) = result.unwrap_err();
        assert_eq!(code, StatusCode::BAD_REQUEST);
    }
}
