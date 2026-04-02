use chrono::Utc;

use super::{EvidenceCheck, PolicyEvalResult};

/// Aggregated validation report.
#[derive(Debug)]
pub struct ValidationReport {
    pub feature_slug: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub overall_pass: bool,
    pub evidence_results: Vec<EvidenceCheck>,
    pub policy_results: Vec<PolicyEvalResult>,
    pub missing_evidence: Vec<(String, String)>,
    pub governance_exceptions: Vec<String>,
}

impl ValidationReport {
    pub(crate) fn to_markdown(&self) -> String {
        let status = if self.overall_pass { "PASS" } else { "FAIL" };
        let mut lines = vec![
            format!("# Validation Report: {}", self.feature_slug),
            format!(
                "**Timestamp**: {} | **Result**: {}",
                self.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
                status
            ),
            String::new(),
            "## Evidence Checks".to_string(),
            String::new(),
        ];

        if self.evidence_results.is_empty() {
            lines.push("_(no evidence requirements defined in governance contract)_".to_string());
        } else {
            lines.push("| FR ID | Type | Found | Threshold Met | Notes |".to_string());
            lines.push("|-------|------|-------|---------------|-------|".to_string());
            for check in &self.evidence_results {
                lines.push(format!(
                    "| {} | {} | {} | {} | {} |",
                    check.fr_id,
                    check.evidence_type,
                    if check.found { "Yes" } else { "No" },
                    if check.threshold_met { "Yes" } else { "N/A" },
                    check.message,
                ));
            }
        }

        if !self.policy_results.is_empty() {
            lines.push(String::new());
            lines.push("## Policy Checks".to_string());
            lines.push(String::new());
            lines.push("| Policy ID | Domain | Passed | Notes |".to_string());
            lines.push("|-----------|--------|--------|-------|".to_string());
            for p in &self.policy_results {
                lines.push(format!(
                    "| {} | {} | {} | {} |",
                    p.policy_id,
                    p.domain,
                    if p.passed { "Yes" } else { "No" },
                    p.message,
                ));
            }
        }

        if !self.missing_evidence.is_empty() {
            lines.push(String::new());
            lines.push("## Missing Evidence".to_string());
            lines.push(String::new());
            for (fr_id, ev_type) in &self.missing_evidence {
                lines.push(format!("- FR `{}`: missing `{}` evidence", fr_id, ev_type));
            }
        }

        if !self.governance_exceptions.is_empty() {
            lines.push(String::new());
            lines.push("## Governance Exceptions".to_string());
            lines.push(String::new());
            for exc in &self.governance_exceptions {
                lines.push(format!("- {exc}"));
            }
        }

        lines.push(String::new());
        lines.join("\n")
    }

    pub(crate) fn to_json(&self) -> String {
        let missing: Vec<serde_json::Value> = self
            .missing_evidence
            .iter()
            .map(|(f, t)| serde_json::json!({"fr_id": f, "type": t}))
            .collect();
        let evidence: Vec<serde_json::Value> = self
            .evidence_results
            .iter()
            .map(|e| {
                serde_json::json!({
                    "fr_id": e.fr_id,
                    "type": e.evidence_type,
                    "found": e.found,
                    "threshold_met": e.threshold_met,
                    "message": e.message,
                })
            })
            .collect();
        let policies: Vec<serde_json::Value> = self
            .policy_results
            .iter()
            .map(|p| {
                serde_json::json!({
                    "policy_id": p.policy_id,
                    "domain": p.domain,
                    "passed": p.passed,
                    "message": p.message,
                })
            })
            .collect();
        serde_json::to_string_pretty(&serde_json::json!({
            "feature_slug": self.feature_slug,
            "timestamp": self.timestamp.to_rfc3339(),
            "overall_pass": self.overall_pass,
            "evidence_results": evidence,
            "policy_results": policies,
            "missing_evidence": missing,
            "governance_exceptions": self.governance_exceptions,
        }))
        .unwrap_or_default()
    }
}
