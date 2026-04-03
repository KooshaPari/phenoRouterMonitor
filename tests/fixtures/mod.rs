//! Test fixture helpers for AgilePlus acceptance and integration tests.
//!
//! Provides typed loaders for all fixture files in `tests/fixtures/`.
//!
//! Traceability: WP16-T097

use std::path::PathBuf;

use agileplus_domain::domain::{
    audit::AuditEntry,
    governance::GovernanceContract,
};

/// Return the absolute path to the `tests/fixtures/` directory.
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Load a raw fixture file as a `String`.
///
/// # Panics
///
/// Panics if the file does not exist or cannot be read.
pub fn load_fixture(name: &str) -> String {
    let path = fixtures_dir().join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {e}", path.display()))
}

/// Load and deserialise `sample-governance.json` into a `GovernanceContract`.
///
/// The fixture only contains `version` and `rules`; the remaining fields
/// (`id`, `feature_id`, `bound_at`) are set to test defaults.
pub fn sample_governance_contract() -> GovernanceContract {
    use chrono::Utc;
    #[derive(serde::Deserialize)]
    struct FixtureContract {
        version: i32,
        rules: Vec<agileplus_domain::domain::governance::GovernanceRule>,
    }
    let json = load_fixture("sample-governance.json");
    let fixture: FixtureContract = serde_json::from_str(&json)
        .expect("sample-governance.json must be valid JSON");
    GovernanceContract {
        id: 1,
        feature_id: 1,
        version: fixture.version,
        rules: fixture.rules,
        bound_at: Utc::now(),
    }
}

/// Load `sample-audit-chain.jsonl` and return all entries in order.
///
/// The returned entries form a valid hash-linked chain.
pub fn sample_audit_chain() -> Vec<AuditEntry> {
    let jsonl = load_fixture("sample-audit-chain.jsonl");
    jsonl
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            serde_json::from_str(line)
                .unwrap_or_else(|e| panic!("Failed to parse audit entry: {e}\nLine: {line}"))
        })
        .collect()
}

/// Load `sample-spec.md` as a raw Markdown string.
pub fn sample_spec() -> String {
    load_fixture("sample-spec.md")
}

/// Load `sample-plan.md` as a raw Markdown string.
pub fn sample_plan() -> String {
    load_fixture("sample-plan.md")
}

/// Load `sample-meta.json` as an untyped JSON value.
pub fn sample_meta() -> serde_json::Value {
    let json = load_fixture("sample-meta.json");
    serde_json::from_str(&json).expect("sample-meta.json must be valid JSON")
}

/// Load a sample evidence JSON file for the given WP sequence and evidence type.
///
/// # Example
///
/// ```rust,ignore
/// let ev = load_evidence(1, "test-results");
/// assert_eq!(ev["evidence_type"], "test_result");
/// ```
pub fn load_evidence(wp_sequence: u32, evidence_type: &str) -> serde_json::Value {
    let path = format!("sample-evidence/WP{wp_sequence:02}/{evidence_type}.json");
    let json = load_fixture(&path);
    serde_json::from_str(&json)
        .unwrap_or_else(|e| panic!("Failed to parse evidence fixture {path}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::domain::audit::AuditChain;

    #[test]
    fn sample_audit_chain_loads_and_verifies() {
        let entries = sample_audit_chain();
        assert_eq!(entries.len(), 5, "Expected 5 audit entries");
        let chain = AuditChain { entries };
        chain
            .verify_chain()
            .expect("Sample audit chain must verify as valid");
    }

    #[test]
    fn sample_governance_contract_loads() {
        let contract = sample_governance_contract();
        assert_eq!(contract.version, 1);
        assert!(!contract.rules.is_empty(), "Contract must have at least one rule");
    }

    #[test]
    fn sample_spec_loads() {
        let spec = sample_spec();
        assert!(spec.contains("FR-001"), "spec must reference FR-001");
    }

    #[test]
    fn sample_plan_loads() {
        let plan = sample_plan();
        assert!(plan.contains("WP01"), "plan must contain WP01");
    }

    #[test]
    fn sample_meta_loads() {
        let meta = sample_meta();
        assert_eq!(meta["slug"], "test-feature");
        assert_eq!(meta["state"], "implementing");
    }

    #[test]
    fn evidence_fixtures_load() {
        let ev = load_evidence(1, "test-results");
        assert_eq!(ev["evidence_type"], "test_result");
        assert!(ev["passed"].as_u64().unwrap_or(0) > 0);

        let rev = load_evidence(1, "review-approval");
        assert_eq!(rev["evidence_type"], "review_approval");
        assert_eq!(rev["approved"], true);
    }
}
