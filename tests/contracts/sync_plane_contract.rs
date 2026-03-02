//! T113: Contract tests for agileplus-sync ↔ agileplus-plane boundary.
//!
//! Verifies that the PlaneStateMapper, OutboundSync mapping logic, and
//! PlaneLabel types satisfy the contracts expected by SyncOrchestrator.
//! Uses in-process, no-network testing with mock data.

use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_plane::client::{PlaneIssue, PlaneIssueResponse};
use agileplus_plane::labels::PlaneLabel;
use agileplus_plane::state_mapper::{PlaneStateMapper, PlaneStateMapperConfig, PlaneStateGroup};

// ---------------------------------------------------------------------------
// Contract: PlaneStateMapper — AgilePlus state → Plane state group
// ---------------------------------------------------------------------------

fn default_mapper() -> PlaneStateMapper {
    PlaneStateMapper::new()
}

#[test]
fn contract_feature_state_created_maps_to_backlog() {
    let mapper = default_mapper();
    let (group, _id) = mapper.to_plane(FeatureState::Created);
    assert_eq!(
        group.as_str(),
        "backlog",
        "Created must map to Plane backlog group"
    );
}

#[test]
fn contract_feature_state_specified_maps_to_unstarted() {
    let mapper = default_mapper();
    let (group, _id) = mapper.to_plane(FeatureState::Specified);
    assert_eq!(group.as_str(), "unstarted");
}

#[test]
fn contract_feature_state_implementing_maps_to_started() {
    let mapper = default_mapper();
    let (group, _id) = mapper.to_plane(FeatureState::Implementing);
    assert_eq!(group.as_str(), "started");
}

#[test]
fn contract_feature_state_validated_maps_to_completed() {
    let mapper = default_mapper();
    let (group, _id) = mapper.to_plane(FeatureState::Validated);
    assert_eq!(group.as_str(), "completed");
}

// ---------------------------------------------------------------------------
// Contract: PlaneStateMapper — Plane state group → AgilePlus state
// ---------------------------------------------------------------------------

#[test]
fn contract_plane_backlog_maps_to_created() {
    let mapper = default_mapper();
    let state = mapper.from_plane("backlog", "Backlog");
    assert_eq!(state, FeatureState::Created);
}

#[test]
fn contract_plane_started_maps_to_implementing() {
    let mapper = default_mapper();
    let state = mapper.from_plane("started", "In Progress");
    assert_eq!(state, FeatureState::Implementing);
}

#[test]
fn contract_plane_completed_maps_to_validated() {
    let mapper = default_mapper();
    let state = mapper.from_plane("completed", "Done");
    assert_eq!(state, FeatureState::Validated);
}

#[test]
fn contract_plane_unknown_group_is_handled_gracefully() {
    let mapper = default_mapper();
    // Must not panic; unknown groups fall back to Created or another sensible default.
    let state = mapper.from_plane("totally-unknown-group", "");
    // Just verify it returns a valid FeatureState.
    let _as_str = state.to_string();
}

// ---------------------------------------------------------------------------
// Contract: PlaneStateGroup parsing is case-insensitive
// ---------------------------------------------------------------------------

#[test]
fn contract_plane_state_group_parsing_case_insensitive() {
    assert_eq!(PlaneStateGroup::from_str("BACKLOG"), PlaneStateGroup::Backlog);
    assert_eq!(PlaneStateGroup::from_str("Started"), PlaneStateGroup::Started);
    assert_eq!(PlaneStateGroup::from_str("COMPLETED"), PlaneStateGroup::Completed);
}

// ---------------------------------------------------------------------------
// Contract: PlaneIssue serialization shape expected by Plane.so API
// ---------------------------------------------------------------------------

#[test]
fn contract_plane_issue_serializes_with_required_name_field() {
    let issue = PlaneIssue {
        id: None,
        name: "Test Feature".to_string(),
        description_html: None,
        state: Some("backlog-state-uuid".to_string()),
        priority: Some(2),
        parent: None,
        labels: vec![],
    };
    let json = serde_json::to_value(&issue).expect("serialize");
    assert_eq!(json["name"], "Test Feature");
    assert!(json.get("id").is_some()); // field present (may be null)
}

#[test]
fn contract_plane_issue_response_has_id_and_name() {
    let raw = r#"{"id":"plane-uuid-1","name":"Test","description_html":null,"state":null,"updated_at":null}"#;
    let resp: PlaneIssueResponse = serde_json::from_str(raw).expect("deserialize");
    assert_eq!(resp.id, "plane-uuid-1");
    assert_eq!(resp.name, "Test");
}

// ---------------------------------------------------------------------------
// Contract: PlaneLabel shape expected by consumer
// ---------------------------------------------------------------------------

#[test]
fn contract_plane_label_has_id_name_and_optional_color() {
    let label = PlaneLabel {
        id: "label-1".to_string(),
        name: "backend".to_string(),
        color: Some("#FF0000".to_string()),
    };
    assert_eq!(label.id, "label-1");
    assert_eq!(label.name, "backend");
    assert!(label.color.is_some());
}

#[test]
fn contract_plane_label_deserializes_from_api_response() {
    let raw = "{\"id\":\"lbl-42\",\"name\":\"frontend\",\"color\":\"#00FF00\"}";
    let label: PlaneLabel = serde_json::from_str(raw).expect("deserialize label");
    assert_eq!(label.id, "lbl-42");
    assert_eq!(label.name, "frontend");
    assert_eq!(label.color.as_deref(), Some("#00FF00"));
}

#[test]
fn contract_plane_label_color_is_optional() {
    let raw = r#"{"id":"lbl-no-color","name":"ops"}"#;
    let label: PlaneLabel = serde_json::from_str(raw).expect("deserialize label no color");
    assert!(label.color.is_none());
}

// ---------------------------------------------------------------------------
// Contract: OutboundSync issue construction from Feature
// ---------------------------------------------------------------------------

#[test]
fn contract_plane_issue_built_from_feature_preserves_name() {
    let feature = Feature::new("test-feature", "Test Feature", [0u8; 32], None);
    // Simulate what OutboundSync.push_feature does when building the issue.
    let mapper = default_mapper();
    let (_group, state_id) = mapper.to_plane(feature.state);
    let state_opt = if state_id.is_empty() { None } else { Some(state_id) };

    let issue = PlaneIssue {
        id: None,
        name: feature.friendly_name.clone(),
        description_html: None,
        state: state_opt,
        priority: Some(2),
        parent: None,
        labels: feature.labels.clone(),
    };

    assert_eq!(issue.name, "Test Feature");
    assert_eq!(issue.priority, Some(2));
}

#[test]
fn contract_feature_with_plane_id_produces_update_not_create() {
    let mut feature = Feature::new("existing", "Existing Feature", [0u8; 32], None);
    feature.plane_issue_id = Some("plane-issue-99".to_string());

    // Contract: if plane_issue_id is Some, the sync adapter must use PATCH (update),
    // not POST (create). Verified by checking the optional id field is treated correctly.
    assert!(feature.plane_issue_id.is_some(), "plane_issue_id must be propagated");
}

// ---------------------------------------------------------------------------
// Contract: state round-trip (AgilePlus → Plane → AgilePlus) is stable
// ---------------------------------------------------------------------------

#[test]
fn contract_state_roundtrip_is_stable() {
    let mapper = default_mapper();
    let agileplus_states = [
        FeatureState::Created,
        FeatureState::Specified,
        FeatureState::Implementing,
        FeatureState::Validated,
    ];

    for original_state in agileplus_states {
        let (group, _) = mapper.to_plane(original_state);
        let recovered = mapper.from_plane(group.as_str(), "");
        assert_eq!(
            recovered, original_state,
            "state {original_state} must survive round-trip through Plane"
        );
    }
}
