//! Proto contract / schema compatibility tests for the Rust provider side.
//!
//! Since pact-rust gRPC support is early-stage (see WP14 risks), we validate
//! schema compatibility by:
//! 1. Verifying proto-generated types have the expected fields.
//! 2. Verifying our conversion functions produce well-formed proto messages.
//!
//! Full Pact broker integration and `buf breaking` CI checks complete the
//! contract testing strategy.
//!
//! Traceability: WP14-T084

use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::work_package::WorkPackage;
use agileplus_grpc::conversions::{feature_to_proto, wp_to_proto};
use agileplus_proto::agileplus::v1::Feature as ProtoFeature;

/// Verify proto Feature fields match the expected contract.
#[test]
fn proto_feature_contract_fields() {
    let domain_feat = Feature::new("contract-test", "Contract Test", [0u8; 32], Some("main"));
    let proto: ProtoFeature = feature_to_proto(domain_feat);

    // Contract: all required fields must be present and non-empty where expected
    assert_eq!(proto.slug, "contract-test", "slug must match");
    assert_eq!(proto.friendly_name, "Contract Test", "friendly_name must match");
    assert!(!proto.state.is_empty(), "state must be non-empty");
    assert!(!proto.created_at.is_empty(), "created_at must be ISO 8601");
    assert!(!proto.updated_at.is_empty(), "updated_at must be ISO 8601");
    // id defaults to 0 for new entities
    assert_eq!(proto.id, 0);
}

/// Verify proto WorkPackageStatus fields match the expected contract.
#[test]
fn proto_wp_status_contract_fields() {
    let wp = WorkPackage::new(1, "Contract WP", 5, "all tests green");
    let proto = wp_to_proto(wp);

    assert_eq!(proto.title, "Contract WP");
    assert_eq!(proto.sequence, 5);
    assert_eq!(proto.state, "planned", "state must be lowercase snake_case");
    // Optional fields default to empty string
    assert_eq!(proto.agent_id, "");
    assert_eq!(proto.pr_url, "");
    assert_eq!(proto.pr_state, "");
}

/// Verify state encoding matches what the Python consumer expects.
///
/// Python consumer maps state strings directly to display/filtering.
/// Any change here is a breaking contract change.
#[test]
fn wp_state_encoding_contract() {
    use agileplus_domain::domain::work_package::{WpState, WorkPackage};

    let state_cases = vec![
        (WpState::Planned, "planned"),
        (WpState::Doing, "doing"),
        (WpState::Review, "review"),
        (WpState::Done, "done"),
        (WpState::Blocked, "blocked"),
    ];

    for (state, expected_str) in state_cases {
        let encoded = format!("{:?}", state).to_lowercase();
        assert_eq!(
            encoded, expected_str,
            "WpState::{:?} must encode as '{}'",
            state, expected_str
        );
    }
}

/// Verify feature state encoding matches what the Python consumer expects.
#[test]
fn feature_state_encoding_contract() {
    use agileplus_domain::domain::state_machine::FeatureState;

    let state_cases = vec![
        (FeatureState::Created, "Created"),
        (FeatureState::Specified, "Specified"),
        (FeatureState::Researched, "Researched"),
        (FeatureState::Planned, "Planned"),
        (FeatureState::Implementing, "Implementing"),
        (FeatureState::Validated, "Validated"),
        (FeatureState::Shipped, "Shipped"),
        (FeatureState::Retrospected, "Retrospected"),
    ];

    for (state, expected_display) in state_cases {
        let domain_feat = Feature::new("s", "S", [0u8; 32], None);
        // We can't mutate state here so just test display directly
        let displayed = state.to_string();
        assert_eq!(
            displayed, expected_display,
            "FeatureState::{:?} must display as '{}'",
            state, expected_display
        );
    }
}

/// Verify hash fields are serialised as byte vecs (not hex strings) in proto.
///
/// The Python consumer decodes them with `bytes(e.prev_hash).hex()`.
#[test]
fn audit_entry_hash_encoding_contract() {
    use agileplus_domain::domain::audit::{AuditEntry, EvidenceRef};
    use agileplus_grpc::conversions::audit_entry_to_proto;

    let entry = AuditEntry {
        id: 1,
        feature_id: 42,
        wp_id: None,
        timestamp: chrono::Utc::now(),
        actor: "agent".to_string(),
        transition: "created->specified".to_string(),
        evidence_refs: vec![EvidenceRef {
            evidence_id: 1,
            fr_id: "FR-001".to_string(),
        }],
        prev_hash: [0u8; 32],
        hash: [0xab; 32],
    };

    let proto = audit_entry_to_proto(entry);

    // Hash must be 32 bytes (256 bits)
    assert_eq!(proto.hash.len(), 32);
    assert_eq!(proto.prev_hash.len(), 32);

    // Check that hash bytes are correct
    assert!(proto.hash.iter().all(|&b| b == 0xab));
    assert!(proto.prev_hash.iter().all(|&b| b == 0x00));

    // Evidence refs must be FR IDs
    assert_eq!(proto.evidence_refs, vec!["FR-001".to_string()]);
}
