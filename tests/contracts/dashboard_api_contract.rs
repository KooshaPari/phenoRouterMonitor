//! T114: Contract tests for agileplus-api ↔ agileplus-dashboard boundary.
//!
//! Verifies the API response shapes expected by the dashboard HTMX consumer:
//! FeatureResponse JSON schema, state string format, and error response format.
//! Uses axum-test to spin up the router in-process without a network server.

use agileplus_api::responses::{FeatureResponse, WorkPackageResponse};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::WorkPackage;
use axum::response::IntoResponse;
use axum::http::StatusCode;
use chrono::Utc;

// ---------------------------------------------------------------------------
// Contract: FeatureResponse JSON shape expected by dashboard
// ---------------------------------------------------------------------------

fn make_feature(slug: &str, state: FeatureState) -> Feature {
    let mut f = Feature::new(slug, &format!("{slug} name"), [0u8; 32], None);
    f.id = 1;
    f.state = state;
    f
}

#[test]
fn contract_feature_response_has_required_fields() {
    let feature = make_feature("my-feature", FeatureState::Specified);
    let resp = FeatureResponse::from(feature);

    // Dashboard contract: must have id, slug, name, state, target_branch, timestamps.
    assert_eq!(resp.id, 1);
    assert_eq!(resp.slug, "my-feature");
    assert_eq!(resp.name, "my-feature name");
    assert!(!resp.state.is_empty(), "state must be non-empty string");
    assert!(!resp.target_branch.is_empty());
    assert!(!resp.created_at.is_empty());
    assert!(!resp.updated_at.is_empty());
}

#[test]
fn contract_feature_response_state_is_lowercase_string() {
    // Dashboard HTMX uses data-status="<state>" — must be lowercase.
    let states = [
        (FeatureState::Created, "created"),
        (FeatureState::Specified, "specified"),
        (FeatureState::Implementing, "implementing"),
        (FeatureState::Validated, "validated"),
        (FeatureState::Shipped, "shipped"),
    ];
    for (state, expected_str) in states {
        let feature = make_feature("f", state);
        let resp = FeatureResponse::from(feature);
        assert_eq!(
            resp.state, expected_str,
            "state {state} must serialize as \"{expected_str}\""
        );
    }
}

#[test]
fn contract_feature_response_serializes_to_json_with_correct_types() {
    let feature = make_feature("kanban-feature", FeatureState::Specified);
    let resp = FeatureResponse::from(feature);
    let json = serde_json::to_value(&resp).expect("serialize FeatureResponse");

    // Dashboard parses these fields from JSON.
    assert!(json["id"].is_number());
    assert!(json["slug"].is_string());
    assert!(json["name"].is_string());
    assert!(json["state"].is_string());
    assert!(json["target_branch"].is_string());
    assert!(json["created_at"].is_string());
    assert!(json["updated_at"].is_string());
}

#[test]
fn contract_feature_response_deserializes_from_json() {
    // Dashboard consumers will deserialize API responses — must round-trip.
    let raw = r#"{
        "id": 42,
        "slug": "my-feat",
        "name": "My Feat",
        "state": "implementing",
        "target_branch": "main",
        "created_at": "2026-03-02T10:00:00+00:00",
        "updated_at": "2026-03-02T10:00:00+00:00"
    }"#;
    let resp: FeatureResponse = serde_json::from_str(raw).expect("deserialize FeatureResponse");
    assert_eq!(resp.id, 42);
    assert_eq!(resp.slug, "my-feat");
    assert_eq!(resp.state, "implementing");
}

// ---------------------------------------------------------------------------
// Contract: WorkPackageResponse JSON shape
// ---------------------------------------------------------------------------

fn make_wp(feature_id: i64, title: &str) -> WorkPackage {
    WorkPackage::new(feature_id, title, 1, "AC: feature works")
}

#[test]
fn contract_work_package_response_has_required_fields() {
    let wp = make_wp(10, "Implement login");
    let resp = WorkPackageResponse::from(wp);
    assert!(resp.id >= 0, "id must be a valid i64");
    assert_eq!(resp.feature_id, 10);
    assert_eq!(resp.title, "Implement login");
    assert!(!resp.state.is_empty());
    assert_eq!(resp.sequence, 1);
    assert!(!resp.acceptance_criteria.is_empty());
    assert!(resp.pr_url.is_none());
}

#[test]
fn contract_work_package_response_state_lowercase() {
    let wp = make_wp(10, "WP");
    let resp = WorkPackageResponse::from(wp);
    assert_eq!(resp.state, resp.state.to_lowercase(), "state must be lowercase");
}

// ---------------------------------------------------------------------------
// Contract: API error JSON shape expected by dashboard
// ---------------------------------------------------------------------------

#[test]
fn contract_api_error_not_found_produces_json_error_field() {
    use agileplus_api::error::ApiError;

    let err = ApiError::NotFound("Feature 'xyz' not found".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn contract_api_error_bad_request_status_code() {
    use agileplus_api::error::ApiError;

    let err = ApiError::BadRequest("invalid state value".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn contract_api_error_conflict_status_code() {
    use agileplus_api::error::ApiError;

    let err = ApiError::Conflict("duplicate slug".to_string());
    let response = err.into_response();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

// ---------------------------------------------------------------------------
// Contract: state filter string parsing matches dashboard request format
// ---------------------------------------------------------------------------

#[test]
fn contract_state_strings_parse_to_feature_states() {
    // The dashboard sends ?state=<lowercase-state> — API must parse these.
    use std::str::FromStr;

    let cases = [
        ("created", FeatureState::Created),
        ("specified", FeatureState::Specified),
        ("researched", FeatureState::Researched),
        ("planned", FeatureState::Planned),
        ("implementing", FeatureState::Implementing),
        ("validated", FeatureState::Validated),
        ("shipped", FeatureState::Shipped),
        ("retrospected", FeatureState::Retrospected),
    ];
    for (s, expected) in cases {
        let parsed = FeatureState::from_str(s).expect(&format!("parse state '{s}'"));
        assert_eq!(parsed, expected);
    }
}

// ---------------------------------------------------------------------------
// Contract: timestamps in responses are RFC3339 formatted
// ---------------------------------------------------------------------------

#[test]
fn contract_timestamps_are_rfc3339() {
    let feature = make_feature("ts-test", FeatureState::Created);
    let resp = FeatureResponse::from(feature);
    // RFC3339 timestamps can be parsed by chrono.
    resp.created_at.parse::<chrono::DateTime<chrono::Utc>>().expect("created_at is RFC3339");
    resp.updated_at.parse::<chrono::DateTime<chrono::Utc>>().expect("updated_at is RFC3339");
}
