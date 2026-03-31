//! API payload builders for testing HTTP handlers.
//!
//! Provides canonical JSON payloads for feature creation, state transitions,
//! and webhook simulations.

use serde_json::{json, Value};

/// Build a canonical JSON payload for creating a feature via the API.
pub fn feature_create_payload(title: &str, description: &str) -> Value {
    json!({
        "title": title,
        "description": description,
    })
}

/// Build a canonical JSON payload for a state transition request.
pub fn transition_payload(target_state: &str) -> Value {
    json!({
        "target_state": target_state,
    })
}

/// Build a canonical Plane.so webhook payload simulating an issue update.
pub fn plane_webhook_payload(feature_id: i64, title: &str, description: &str) -> Value {
    json!({
        "event": "issue.updated",
        "data": {
            "id": feature_id.to_string(),
            "title": title,
            "description": description,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_create_payload_is_valid_json() {
        let p = feature_create_payload("My Feature", "Some description");
        assert_eq!(p["title"], "My Feature");
        assert_eq!(p["description"], "Some description");
    }

    #[test]
    fn transition_payload_contains_target_state() {
        let p = transition_payload("specified");
        assert_eq!(p["target_state"], "specified");
    }

    #[test]
    fn plane_webhook_payload_structure() {
        let p = plane_webhook_payload(42, "Title", "Desc");
        assert_eq!(p["event"], "issue.updated");
        assert_eq!(p["data"]["id"], "42");
        assert_eq!(p["data"]["title"], "Title");
    }
}
