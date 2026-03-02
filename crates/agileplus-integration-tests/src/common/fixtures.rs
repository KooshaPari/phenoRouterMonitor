//! Test data fixtures for integration tests.
//!
//! Traceability: WP19-T107

use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use chrono::Utc;

/// A collection of pre-built test fixtures for use across integration tests.
pub struct TestFixtures {
    /// A feature ready to be inserted as "feature-1".
    pub feature1: Feature,
    /// A second feature ready to be inserted as "feature-2".
    pub feature2: Feature,
}

impl TestFixtures {
    /// Build fixtures using deterministic values (no DB interaction).
    pub fn new() -> Self {
        let feature1 = Feature {
            id: 1,
            slug: "implement-caching-layer".to_string(),
            friendly_name: "Implement caching layer".to_string(),
            state: FeatureState::Created,
            spec_hash: [0x01u8; 32],
            target_branch: "main".to_string(),
            plane_issue_id: None,
            plane_state_id: None,
            labels: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let feature2 = Feature {
            id: 2,
            slug: "add-api-rate-limiting".to_string(),
            friendly_name: "Add API rate limiting".to_string(),
            state: FeatureState::Created,
            spec_hash: [0x02u8; 32],
            target_branch: "main".to_string(),
            plane_issue_id: None,
            plane_state_id: None,
            labels: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Self { feature1, feature2 }
    }
}

impl Default for TestFixtures {
    fn default() -> Self {
        Self::new()
    }
}

/// Create and return in-memory test fixtures.
///
/// For full integration tests this would also seed a database; here we return
/// pure data so that unit tests can use fixtures without any I/O.
pub async fn seed_test_data() -> TestFixtures {
    TestFixtures::new()
}

/// Build a canonical JSON payload for creating a feature via the API.
pub fn feature_create_payload(title: &str, description: &str) -> serde_json::Value {
    serde_json::json!({
        "title": title,
        "description": description,
    })
}

/// Build a canonical JSON payload for a state transition request.
pub fn transition_payload(target_state: &str) -> serde_json::Value {
    serde_json::json!({
        "target_state": target_state,
    })
}

/// Build a canonical Plane.so webhook payload simulating an issue update.
pub fn plane_webhook_payload(feature_id: i64, title: &str, description: &str) -> serde_json::Value {
    serde_json::json!({
        "event": "issue.updated",
        "data": {
            "id": feature_id.to_string(),
            "title": title,
            "description": description,
        }
    })
}

// ---------------------------------------------------------------------------
// Unit tests — always run (no external services)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixtures_build_without_panic() {
        let f = TestFixtures::new();
        assert_eq!(f.feature1.slug, "implement-caching-layer");
        assert_eq!(f.feature2.slug, "add-api-rate-limiting");
        assert_eq!(f.feature1.state, FeatureState::Created);
        assert_eq!(f.feature2.state, FeatureState::Created);
    }

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

    #[tokio::test]
    async fn seed_test_data_returns_fixtures() {
        let f = seed_test_data().await;
        assert_eq!(f.feature1.id, 1);
        assert_eq!(f.feature2.id, 2);
    }
}
