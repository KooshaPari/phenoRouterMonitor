use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::state_machine::StateTransition;
use crate::error::DomainError;

// ---------------------------------------------------------------------------
// GovernanceContract (T018)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceContract {
    pub id: i64,
    pub feature_id: i64,
    pub version: i32,
    pub rules: Vec<GovernanceRule>,
    pub bound_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceRule {
    pub transition: String,
    pub required_evidence: Vec<EvidenceRequirement>,
    pub policy_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRequirement {
    pub fr_id: String,
    pub evidence_type: EvidenceType,
    pub threshold: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    TestResult,
    CiOutput,
    ReviewApproval,
    SecurityScan,
    LintResult,
    ManualAttestation,
}

/// All valid FeatureState names for transition parsing.
const VALID_STATES: &[&str] = &[
    "created",
    "specified",
    "researched",
    "planned",
    "implementing",
    "validated",
    "shipped",
    "retrospected",
];

fn is_valid_state(s: &str) -> bool {
    VALID_STATES.contains(&s)
}

impl GovernanceContract {
    pub fn new(feature_id: i64, version: i32, rules: Vec<GovernanceRule>) -> Self {
        Self {
            id: 0,
            feature_id,
            version,
            rules,
            bound_at: Utc::now(),
        }
    }

    /// Validates that all rule transitions are valid `from -> to` state pairs.
    pub fn validate_rules(&self) -> Result<(), DomainError> {
        for rule in &self.rules {
            let parts: Vec<&str> = rule.transition.split(" -> ").collect();
            if parts.len() != 2 {
                return Err(DomainError::InvalidGovernanceRule(format!(
                    "malformed transition: '{}'",
                    rule.transition
                )));
            }
            if !is_valid_state(parts[0]) {
                return Err(DomainError::InvalidGovernanceRule(format!(
                    "unknown state: '{}'",
                    parts[0]
                )));
            }
            if !is_valid_state(parts[1]) {
                return Err(DomainError::InvalidGovernanceRule(format!(
                    "unknown state: '{}'",
                    parts[1]
                )));
            }
        }
        Ok(())
    }

    /// Returns all rules whose transition string matches the given `StateTransition`.
    pub fn rules_for_transition(&self, transition: &StateTransition) -> Vec<&GovernanceRule> {
        let key = transition.to_string();
        self.rules.iter().filter(|r| r.transition == key).collect()
    }
}

// ---------------------------------------------------------------------------
// Evidence (T022)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: i64,
    pub wp_id: i64,
    pub fr_id: String,
    pub evidence_type: EvidenceType,
    pub artifact_path: String,
    pub metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
}

impl Evidence {
    pub fn new(
        wp_id: i64,
        fr_id: impl Into<String>,
        evidence_type: EvidenceType,
        artifact_path: impl Into<String>,
        metadata: Option<Value>,
    ) -> Self {
        Self {
            id: 0,
            wp_id,
            fr_id: fr_id.into(),
            evidence_type,
            artifact_path: artifact_path.into(),
            metadata,
            created_at: Utc::now(),
        }
    }

    /// Returns true if this evidence satisfies the given requirement.
    pub fn satisfies_requirement(&self, req: &EvidenceRequirement) -> bool {
        if self.fr_id != req.fr_id {
            return false;
        }
        if self.evidence_type != req.evidence_type {
            return false;
        }
        // If a threshold is specified, check metadata meets it.
        if let Some(threshold) = &req.threshold {
            match &self.metadata {
                Some(meta) => {
                    // Numeric comparison: metadata value >= threshold value.
                    if let (Some(tv), Some(mv)) = (threshold.as_f64(), meta.as_f64()) {
                        return mv >= tv;
                    }
                    // Fall back to equality.
                    meta == threshold
                }
                None => false,
            }
        } else {
            true
        }
    }
}

/// Describes a missing piece of evidence.
#[derive(Debug, Clone)]
pub struct MissingEvidence {
    pub transition: String,
    pub requirement: EvidenceRequirement,
}

/// Checks which evidence requirements are not satisfied by the given evidence.
pub fn check_evidence_completeness(
    evidence: &[Evidence],
    rules: &[GovernanceRule],
) -> Vec<MissingEvidence> {
    let mut missing = Vec::new();
    for rule in rules {
        for req in &rule.required_evidence {
            let satisfied = evidence.iter().any(|e| e.satisfies_requirement(req));
            if !satisfied {
                missing.push(MissingEvidence {
                    transition: rule.transition.clone(),
                    requirement: req.clone(),
                });
            }
        }
    }
    missing
}

// ---------------------------------------------------------------------------
// PolicyRule (T023)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: i64,
    pub domain: PolicyDomain,
    pub rule: PolicyDefinition,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDomain {
    Quality,
    Security,
    Reliability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDefinition {
    pub name: String,
    pub description: String,
    pub check: PolicyCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyCheck {
    MinCoverage { threshold: f64 },
    MaxCriticalFindings { threshold: u32 },
    RequiredEvidenceTypes { types: Vec<EvidenceType> },
    LintClean { strict: bool },
    ReviewApproved,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PolicyResult {
    Pass,
    Fail { reason: String },
    Skip { reason: String },
}

impl PolicyRule {
    pub fn evaluate(&self, evidence: &[Evidence]) -> PolicyResult {
        if !self.active {
            return PolicyResult::Skip {
                reason: "policy is inactive".to_string(),
            };
        }
        match &self.rule.check {
            PolicyCheck::MinCoverage { threshold } => {
                // Look for test_result evidence with numeric metadata representing coverage.
                let coverage = evidence
                    .iter()
                    .filter(|e| e.evidence_type == EvidenceType::TestResult)
                    .filter_map(|e| e.metadata.as_ref()?.as_f64())
                    .fold(f64::NEG_INFINITY, f64::max);
                if coverage >= *threshold {
                    PolicyResult::Pass
                } else if coverage == f64::NEG_INFINITY {
                    PolicyResult::Fail {
                        reason: "no coverage data found".to_string(),
                    }
                } else {
                    PolicyResult::Fail {
                        reason: format!("coverage {coverage:.1}% < {threshold:.1}%"),
                    }
                }
            }
            PolicyCheck::MaxCriticalFindings { threshold } => {
                let count = evidence
                    .iter()
                    .filter(|e| e.evidence_type == EvidenceType::SecurityScan)
                    .filter_map(|e| e.metadata.as_ref()?.as_u64())
                    .sum::<u64>();
                if count <= u64::from(*threshold) {
                    PolicyResult::Pass
                } else {
                    PolicyResult::Fail {
                        reason: format!("{count} critical findings > {threshold}"),
                    }
                }
            }
            PolicyCheck::RequiredEvidenceTypes { types } => {
                let present: std::collections::HashSet<EvidenceType> =
                    evidence.iter().map(|e| e.evidence_type).collect();
                let missing: Vec<_> = types.iter().filter(|t| !present.contains(t)).collect();
                if missing.is_empty() {
                    PolicyResult::Pass
                } else {
                    PolicyResult::Fail {
                        reason: format!("missing evidence types: {missing:?}"),
                    }
                }
            }
            PolicyCheck::LintClean { strict } => {
                let has_lint = evidence
                    .iter()
                    .any(|e| e.evidence_type == EvidenceType::LintResult);
                if !has_lint {
                    return PolicyResult::Fail {
                        reason: "no lint results".to_string(),
                    };
                }
                if *strict {
                    // In strict mode, metadata must be 0 (zero warnings).
                    let all_clean = evidence
                        .iter()
                        .filter(|e| e.evidence_type == EvidenceType::LintResult)
                        .all(|e| e.metadata.as_ref().and_then(|m| m.as_u64()) == Some(0));
                    if all_clean {
                        PolicyResult::Pass
                    } else {
                        PolicyResult::Fail {
                            reason: "lint warnings found in strict mode".to_string(),
                        }
                    }
                } else {
                    PolicyResult::Pass
                }
            }
            PolicyCheck::ReviewApproved => {
                let approved = evidence
                    .iter()
                    .any(|e| e.evidence_type == EvidenceType::ReviewApproval);
                if approved {
                    PolicyResult::Pass
                } else {
                    PolicyResult::Fail {
                        reason: "no review approval".to_string(),
                    }
                }
            }
        }
    }
}

pub fn evaluate_all_policies<'a>(
    policies: &'a [PolicyRule],
    evidence: &[Evidence],
) -> Vec<(&'a PolicyRule, PolicyResult)> {
    policies.iter().map(|p| (p, p.evaluate(evidence))).collect()
}

// Bring FeatureState into scope so it's accessible from tests that reference governance types.
pub use super::state_machine::FeatureState;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::state_machine::StateTransition;

    fn make_contract(rules: Vec<GovernanceRule>) -> GovernanceContract {
        GovernanceContract::new(1, 1, rules)
    }

    fn make_rule(transition: &str, evidence: Vec<EvidenceRequirement>) -> GovernanceRule {
        GovernanceRule {
            transition: transition.to_string(),
            required_evidence: evidence,
            policy_refs: vec![],
        }
    }

    // -- GovernanceContract tests --

    #[test]
    fn test_validate_rules_valid() {
        let contract = make_contract(vec![make_rule("created -> specified", vec![])]);
        assert!(contract.validate_rules().is_ok());
    }

    #[test]
    fn test_validate_rules_invalid_state() {
        let contract = make_contract(vec![make_rule("created -> bogus", vec![])]);
        assert!(contract.validate_rules().is_err());
    }

    #[test]
    fn test_validate_rules_malformed() {
        let contract = make_contract(vec![make_rule("no_arrow", vec![])]);
        assert!(contract.validate_rules().is_err());
    }

    #[test]
    fn test_rules_for_transition() {
        let contract = make_contract(vec![
            make_rule("created -> specified", vec![]),
            make_rule("specified -> researched", vec![]),
            make_rule("created -> specified", vec![]),
        ]);
        let t = StateTransition {
            from: FeatureState::Created,
            to: FeatureState::Specified,
            skipped: vec![],
        };
        assert_eq!(contract.rules_for_transition(&t).len(), 2);
    }

    // -- Evidence tests --

    #[test]
    fn test_evidence_satisfies_requirement() {
        let e = Evidence::new(1, "FR-01", EvidenceType::TestResult, "/path", None);
        let req = EvidenceRequirement {
            fr_id: "FR-01".to_string(),
            evidence_type: EvidenceType::TestResult,
            threshold: None,
        };
        assert!(e.satisfies_requirement(&req));
    }

    #[test]
    fn test_evidence_wrong_type() {
        let e = Evidence::new(1, "FR-01", EvidenceType::LintResult, "/path", None);
        let req = EvidenceRequirement {
            fr_id: "FR-01".to_string(),
            evidence_type: EvidenceType::TestResult,
            threshold: None,
        };
        assert!(!e.satisfies_requirement(&req));
    }

    #[test]
    fn test_evidence_wrong_fr_id() {
        let e = Evidence::new(1, "FR-02", EvidenceType::TestResult, "/path", None);
        let req = EvidenceRequirement {
            fr_id: "FR-01".to_string(),
            evidence_type: EvidenceType::TestResult,
            threshold: None,
        };
        assert!(!e.satisfies_requirement(&req));
    }

    #[test]
    fn test_evidence_threshold_pass() {
        let e = Evidence::new(
            1,
            "FR-01",
            EvidenceType::TestResult,
            "/p",
            Some(serde_json::json!(85.0)),
        );
        let req = EvidenceRequirement {
            fr_id: "FR-01".to_string(),
            evidence_type: EvidenceType::TestResult,
            threshold: Some(serde_json::json!(80.0)),
        };
        assert!(e.satisfies_requirement(&req));
    }

    #[test]
    fn test_evidence_threshold_fail() {
        let e = Evidence::new(
            1,
            "FR-01",
            EvidenceType::TestResult,
            "/p",
            Some(serde_json::json!(70.0)),
        );
        let req = EvidenceRequirement {
            fr_id: "FR-01".to_string(),
            evidence_type: EvidenceType::TestResult,
            threshold: Some(serde_json::json!(80.0)),
        };
        assert!(!e.satisfies_requirement(&req));
    }

    #[test]
    fn test_evidence_completeness() {
        let rules = vec![make_rule(
            "created -> specified",
            vec![EvidenceRequirement {
                fr_id: "FR-01".to_string(),
                evidence_type: EvidenceType::TestResult,
                threshold: None,
            }],
        )];
        let missing = check_evidence_completeness(&[], &rules);
        assert_eq!(missing.len(), 1);

        let evidence = vec![Evidence::new(
            1,
            "FR-01",
            EvidenceType::TestResult,
            "/p",
            None,
        )];
        let missing = check_evidence_completeness(&evidence, &rules);
        assert_eq!(missing.len(), 0);
    }

    // -- Policy tests --

    fn make_policy(check: PolicyCheck, active: bool) -> PolicyRule {
        let now = Utc::now();
        PolicyRule {
            id: 1,
            domain: PolicyDomain::Quality,
            rule: PolicyDefinition {
                name: "test".to_string(),
                description: "test".to_string(),
                check,
            },
            active,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_policy_inactive_skip() {
        let p = make_policy(PolicyCheck::ReviewApproved, false);
        assert_eq!(
            p.evaluate(&[]),
            PolicyResult::Skip {
                reason: "policy is inactive".to_string()
            }
        );
    }

    #[test]
    fn test_policy_min_coverage_pass() {
        let p = make_policy(PolicyCheck::MinCoverage { threshold: 80.0 }, true);
        let e = vec![Evidence::new(
            1,
            "FR-01",
            EvidenceType::TestResult,
            "/p",
            Some(serde_json::json!(85.0)),
        )];
        assert_eq!(p.evaluate(&e), PolicyResult::Pass);
    }

    #[test]
    fn test_policy_min_coverage_fail() {
        let p = make_policy(PolicyCheck::MinCoverage { threshold: 80.0 }, true);
        let e = vec![Evidence::new(
            1,
            "FR-01",
            EvidenceType::TestResult,
            "/p",
            Some(serde_json::json!(70.0)),
        )];
        assert!(matches!(p.evaluate(&e), PolicyResult::Fail { .. }));
    }

    #[test]
    fn test_policy_max_critical_findings_pass() {
        let p = make_policy(PolicyCheck::MaxCriticalFindings { threshold: 0 }, true);
        let e = vec![Evidence::new(
            1,
            "FR-01",
            EvidenceType::SecurityScan,
            "/p",
            Some(serde_json::json!(0)),
        )];
        assert_eq!(p.evaluate(&e), PolicyResult::Pass);
    }

    #[test]
    fn test_policy_max_critical_findings_fail() {
        let p = make_policy(PolicyCheck::MaxCriticalFindings { threshold: 0 }, true);
        let e = vec![Evidence::new(
            1,
            "FR-01",
            EvidenceType::SecurityScan,
            "/p",
            Some(serde_json::json!(3)),
        )];
        assert!(matches!(p.evaluate(&e), PolicyResult::Fail { .. }));
    }

    #[test]
    fn test_policy_required_evidence_types_pass() {
        let p = make_policy(
            PolicyCheck::RequiredEvidenceTypes {
                types: vec![EvidenceType::TestResult],
            },
            true,
        );
        let e = vec![Evidence::new(
            1,
            "FR-01",
            EvidenceType::TestResult,
            "/p",
            None,
        )];
        assert_eq!(p.evaluate(&e), PolicyResult::Pass);
    }

    #[test]
    fn test_policy_required_evidence_types_fail() {
        let p = make_policy(
            PolicyCheck::RequiredEvidenceTypes {
                types: vec![EvidenceType::ReviewApproval],
            },
            true,
        );
        assert!(matches!(p.evaluate(&[]), PolicyResult::Fail { .. }));
    }

    #[test]
    fn test_policy_lint_clean_pass() {
        let p = make_policy(PolicyCheck::LintClean { strict: true }, true);
        let e = vec![Evidence::new(
            1,
            "FR-01",
            EvidenceType::LintResult,
            "/p",
            Some(serde_json::json!(0)),
        )];
        assert_eq!(p.evaluate(&e), PolicyResult::Pass);
    }

    #[test]
    fn test_policy_lint_clean_fail_strict() {
        let p = make_policy(PolicyCheck::LintClean { strict: true }, true);
        let e = vec![Evidence::new(
            1,
            "FR-01",
            EvidenceType::LintResult,
            "/p",
            Some(serde_json::json!(5)),
        )];
        assert!(matches!(p.evaluate(&e), PolicyResult::Fail { .. }));
    }

    #[test]
    fn test_policy_review_approved_pass() {
        let p = make_policy(PolicyCheck::ReviewApproved, true);
        let e = vec![Evidence::new(
            1,
            "FR-01",
            EvidenceType::ReviewApproval,
            "/p",
            None,
        )];
        assert_eq!(p.evaluate(&e), PolicyResult::Pass);
    }

    #[test]
    fn test_policy_review_approved_fail() {
        let p = make_policy(PolicyCheck::ReviewApproved, true);
        assert!(matches!(p.evaluate(&[]), PolicyResult::Fail { .. }));
    }

    #[test]
    fn test_evaluate_all_policies() {
        let policies = vec![
            make_policy(PolicyCheck::ReviewApproved, true),
            make_policy(PolicyCheck::LintClean { strict: false }, false),
        ];
        let results = evaluate_all_policies(&policies, &[]);
        assert_eq!(results.len(), 2);
        assert!(matches!(results[0].1, PolicyResult::Fail { .. }));
        assert!(matches!(results[1].1, PolicyResult::Skip { .. }));
    }
}
