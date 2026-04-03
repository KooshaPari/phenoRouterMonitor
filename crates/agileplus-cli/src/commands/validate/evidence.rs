use anyhow::Result;

use agileplus_domain::domain::governance::{Evidence, GovernanceContract};
use agileplus_domain::ports::StoragePort;

use super::EvidenceCheck;

/// Evaluate governance evidence requirements against stored evidence.
pub(crate) async fn evaluate_evidence<S: StoragePort>(
    storage: &S,
    contract: &GovernanceContract,
    _feature_id: i64,
) -> Result<(Vec<EvidenceCheck>, Vec<(String, String)>)> {
    let mut results = Vec::new();
    let mut missing = Vec::new();
    let target_transition_keywords = ["Implementing", "validated", "Validate", "implementing"];

    for rule in &contract.rules {
        let is_validate_rule = target_transition_keywords
            .iter()
            .any(|kw| rule.transition.to_lowercase().contains(&kw.to_lowercase()));

        for req in &rule.required_evidence {
            let evidence_list = storage
                .get_evidence_by_fr(&req.fr_id)
                .await
                .unwrap_or_default();

            let relevant: Vec<&Evidence> = evidence_list.iter().collect();
            let found = !relevant.is_empty();

            let threshold_met = if let (true, Some(threshold)) = (found, &req.threshold) {
                evaluate_threshold(relevant.as_slice(), threshold)
            } else {
                found
            };

            let message = if !found {
                format!("No evidence found for FR `{}`", req.fr_id)
            } else if !threshold_met {
                format!("Threshold not met for FR `{}`", req.fr_id)
            } else {
                "OK".to_string()
            };

            if !found {
                missing.push((req.fr_id.clone(), format!("{:?}", req.evidence_type)));
            }

            results.push(EvidenceCheck {
                fr_id: req.fr_id.clone(),
                evidence_type: format!("{:?}", req.evidence_type),
                found,
                threshold_met,
                message,
            });

            let _ = is_validate_rule;
        }
    }

    Ok((results, missing))
}

/// Check if evidence meets a threshold defined in the governance contract.
pub(crate) fn evaluate_threshold(evidence: &[&Evidence], threshold: &serde_json::Value) -> bool {
    if let Some(min_cov) = threshold.get("min_coverage").and_then(|v| v.as_f64()) {
        for ev in evidence {
            if let Some(meta) = &ev.metadata {
                if let Some(cov) = meta.get("coverage").and_then(|v| v.as_f64()) {
                    if cov >= min_cov {
                        return true;
                    }
                }
            }
        }
        return false;
    }
    if let Some(max_crit) = threshold.get("max_critical").and_then(|v| v.as_u64()) {
        let critical_count: u64 = evidence
            .iter()
            .filter_map(|ev| ev.metadata.as_ref())
            .filter_map(|meta| meta.get("critical_count"))
            .filter_map(|v| v.as_u64())
            .sum();
        return critical_count <= max_crit;
    }
    true
}
