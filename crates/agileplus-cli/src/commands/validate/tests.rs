use super::*;
use agileplus_domain::domain::governance::{
    EvidenceRequirement, EvidenceType, GovernanceContract, GovernanceRule,
};
use chrono::Utc;

#[allow(dead_code)]
fn make_contract(feature_id: i64) -> GovernanceContract {
    GovernanceContract {
        id: 1,
        feature_id,
        version: 1,
        rules: vec![GovernanceRule {
            transition: "Implementing -> Validated".to_string(),
            required_evidence: vec![EvidenceRequirement {
                fr_id: "FR-001".to_string(),
                evidence_type: EvidenceType::CiOutput,
                threshold: None,
            }],
            policy_refs: vec![],
        }],
        bound_at: Utc::now(),
    }
}

#[test]
fn report_to_markdown_pass() {
    let report = ValidationReport {
        feature_slug: "my-feat".to_string(),
        timestamp: Utc::now(),
        overall_pass: true,
        evidence_results: vec![EvidenceCheck {
            fr_id: "FR-001".to_string(),
            evidence_type: "CiOutput".to_string(),
            found: true,
            threshold_met: true,
            message: "OK".to_string(),
        }],
        policy_results: vec![],
        missing_evidence: vec![],
        governance_exceptions: vec![],
    };
    let md = report.to_markdown();
    assert!(md.contains("PASS"));
    assert!(md.contains("FR-001"));
}

#[test]
fn report_to_markdown_fail_missing_evidence() {
    let report = ValidationReport {
        feature_slug: "my-feat".to_string(),
        timestamp: Utc::now(),
        overall_pass: false,
        evidence_results: vec![EvidenceCheck {
            fr_id: "FR-001".to_string(),
            evidence_type: "CiOutput".to_string(),
            found: false,
            threshold_met: false,
            message: "No evidence found for FR `FR-001`".to_string(),
        }],
        policy_results: vec![],
        missing_evidence: vec![("FR-001".to_string(), "CiOutput".to_string())],
        governance_exceptions: vec![],
    };
    let md = report.to_markdown();
    assert!(md.contains("FAIL"));
    assert!(md.contains("Missing Evidence"));
}

#[test]
fn report_to_json_has_required_fields() {
    let report = ValidationReport {
        feature_slug: "feat".to_string(),
        timestamp: Utc::now(),
        overall_pass: true,
        evidence_results: vec![],
        policy_results: vec![],
        missing_evidence: vec![],
        governance_exceptions: vec![],
    };
    let json = report.to_json();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["feature_slug"], "feat");
    assert_eq!(v["overall_pass"], true);
}

#[test]
fn evaluate_threshold_min_coverage_pass() {
    use agileplus_domain::domain::governance::Evidence;
    let ev = Evidence {
        id: 1,
        wp_id: 1,
        fr_id: "FR-001".to_string(),
        evidence_type: EvidenceType::TestResult,
        artifact_path: "ci.log".to_string(),
        metadata: Some(serde_json::json!({"coverage": 85.0})),
        created_at: Utc::now(),
    };
    let threshold = serde_json::json!({"min_coverage": 80.0});
    assert!(super::evidence::evaluate_threshold(&[&ev], &threshold));
}

#[test]
fn evaluate_threshold_min_coverage_fail() {
    use agileplus_domain::domain::governance::Evidence;
    let ev = Evidence {
        id: 1,
        wp_id: 1,
        fr_id: "FR-001".to_string(),
        evidence_type: EvidenceType::TestResult,
        artifact_path: "ci.log".to_string(),
        metadata: Some(serde_json::json!({"coverage": 60.0})),
        created_at: Utc::now(),
    };
    let threshold = serde_json::json!({"min_coverage": 80.0});
    assert!(!super::evidence::evaluate_threshold(&[&ev], &threshold));
}

#[test]
fn evaluate_threshold_max_critical_pass() {
    use agileplus_domain::domain::governance::Evidence;
    let ev = Evidence {
        id: 1,
        wp_id: 1,
        fr_id: "FR-SEC".to_string(),
        evidence_type: EvidenceType::SecurityScan,
        artifact_path: "scan.json".to_string(),
        metadata: Some(serde_json::json!({"critical_count": 0})),
        created_at: Utc::now(),
    };
    let threshold = serde_json::json!({"max_critical": 0});
    assert!(super::evidence::evaluate_threshold(&[&ev], &threshold));
}

#[test]
fn evaluate_threshold_max_critical_fail() {
    use agileplus_domain::domain::governance::Evidence;
    let ev = Evidence {
        id: 1,
        wp_id: 1,
        fr_id: "FR-SEC".to_string(),
        evidence_type: EvidenceType::SecurityScan,
        artifact_path: "scan.json".to_string(),
        metadata: Some(serde_json::json!({"critical_count": 3})),
        created_at: Utc::now(),
    };
    let threshold = serde_json::json!({"max_critical": 0});
    assert!(!super::evidence::evaluate_threshold(&[&ev], &threshold));
}
