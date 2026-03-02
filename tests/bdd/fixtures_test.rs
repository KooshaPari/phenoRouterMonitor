//! Tests that validate all fixture files load and parse correctly.
//!
//! Traceability: WP16-T097

use agileplus_domain::domain::audit::AuditChain;
use agileplus_domain::domain::governance::GovernanceContract;
use std::path::PathBuf;

// ─────────────────────────────────────────────────────────────────────────────
// Fixture loading helpers
// ─────────────────────────────────────────────────────────────────────────────

fn fixtures_root() -> PathBuf {
    // CARGO_MANIFEST_DIR for agileplus-bdd is tests/bdd/
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures")
}

fn load_fixture(name: &str) -> String {
    let path = fixtures_root().join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Cannot read fixture '{}': {e}", path.display()))
}

fn sample_audit_chain() -> Vec<agileplus_domain::domain::audit::AuditEntry> {
    let jsonl = load_fixture("sample-audit-chain.jsonl");
    jsonl
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("audit parse: {e}")))
        .collect()
}

fn sample_governance() -> GovernanceContract {
    use chrono::Utc;
    #[derive(serde::Deserialize)]
    struct Partial {
        version: i32,
        rules: Vec<agileplus_domain::domain::governance::GovernanceRule>,
    }
    let json = load_fixture("sample-governance.json");
    let p: Partial = serde_json::from_str(&json).unwrap();
    GovernanceContract {
        id: 1,
        feature_id: 1,
        version: p.version,
        rules: p.rules,
        bound_at: Utc::now(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn sample_audit_chain_verifies() {
    let entries = sample_audit_chain();
    assert_eq!(entries.len(), 5, "Expected 5 audit entries in sample chain");
    let chain = AuditChain { entries };
    chain
        .verify_chain()
        .expect("sample-audit-chain.jsonl must form a valid hash chain");
}

#[test]
fn sample_governance_parses() {
    let contract = sample_governance();
    assert_eq!(contract.version, 1);
    assert!(!contract.rules.is_empty());
}

#[test]
fn sample_spec_contains_frs() {
    let spec = load_fixture("sample-spec.md");
    assert!(spec.contains("FR-001"), "sample-spec.md must reference FR-001");
    assert!(spec.contains("FR-002"), "sample-spec.md must reference FR-002");
    assert!(spec.contains("FR-003"), "sample-spec.md must reference FR-003");
}

#[test]
fn sample_plan_contains_wps() {
    let plan = load_fixture("sample-plan.md");
    assert!(plan.contains("WP01"), "sample-plan.md must contain WP01");
    assert!(plan.contains("WP02"), "sample-plan.md must contain WP02");
    assert!(plan.contains("WP03"), "sample-plan.md must contain WP03");
}

#[test]
fn sample_meta_parses() {
    let json = load_fixture("sample-meta.json");
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["slug"], "test-feature");
    assert_eq!(v["state"], "implementing");
}

#[test]
fn evidence_fixtures_parse() {
    let ev = serde_json::from_str::<serde_json::Value>(
        &load_fixture("sample-evidence/WP01/test-results.json"),
    )
    .unwrap();
    assert_eq!(ev["evidence_type"], "test_result");
    assert!(ev["passed"].as_u64().unwrap_or(0) > 0);

    let rev = serde_json::from_str::<serde_json::Value>(
        &load_fixture("sample-evidence/WP01/review-approval.json"),
    )
    .unwrap();
    assert_eq!(rev["evidence_type"], "review_approval");
    assert_eq!(rev["approved"], true);

    let wp2_ev = serde_json::from_str::<serde_json::Value>(
        &load_fixture("sample-evidence/WP02/test-results.json"),
    )
    .unwrap();
    assert_eq!(wp2_ev["evidence_type"], "test_result");
    assert_eq!(wp2_ev["wp_sequence"], 2);
}

#[test]
fn pact_fixture_parses() {
    let pact_path = fixtures_root()
        .parent()
        .unwrap()
        .join("contract/pacts/AgilePlusMCP-AgilePlusCore.json");
    let json = std::fs::read_to_string(&pact_path)
        .unwrap_or_else(|e| panic!("Cannot read pact fixture: {e}"));
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["consumer"]["name"], "AgilePlusMCP");
    assert_eq!(v["provider"]["name"], "AgilePlusCore");
    let interactions = v["interactions"].as_array().unwrap();
    assert!(!interactions.is_empty(), "Pact must have at least one interaction");
}
