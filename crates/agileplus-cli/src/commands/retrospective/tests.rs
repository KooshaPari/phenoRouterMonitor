use chrono::{Duration, Utc};

use agileplus_domain::domain::audit::AuditEntry;
use agileplus_domain::domain::governance::EvidenceRequirement;
use agileplus_domain::domain::governance::{EvidenceType, GovernanceContract, GovernanceRule};

use super::metrics::{
    FeatureMetrics, WpMetrics, compute_durations_from_audit, format_duration, generate_insights,
};
use super::report::generate_retro_markdown;
use agileplus_domain::domain::audit::hash_entry;

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

fn make_audit(transition: &str, ts: chrono::DateTime<Utc>) -> AuditEntry {
    let mut e = AuditEntry {
        id: 0,
        feature_id: 1,
        wp_id: None,
        timestamp: ts,
        actor: "user".into(),
        transition: transition.into(),
        evidence_refs: vec![],
        prev_hash: [0u8; 32],
        hash: [0u8; 32],
        event_id: None,
        archived_to: None,
    };
    e.hash = hash_entry(&e);
    e
}

#[test]
fn format_duration_secs() {
    assert_eq!(format_duration(5000), "5s");
}

#[test]
fn format_duration_minutes() {
    assert_eq!(format_duration(65000), "1m 5s");
}

#[test]
fn format_duration_hours() {
    assert_eq!(format_duration(3660000), "1h 1m");
}

#[test]
fn format_duration_days() {
    let ms = (2 * 24 * 3600 + 3 * 3600) * 1000i64;
    let result = format_duration(ms);
    assert!(result.starts_with("2d"));
}

#[test]
fn format_duration_negative() {
    assert_eq!(format_duration(-100), "N/A");
}

#[test]
fn compute_durations_empty_trail() {
    let now = Utc::now();
    let (total, phases) = compute_durations_from_audit(&[], &now);
    assert_eq!(total, 0);
    assert!(phases.is_empty());
}

#[test]
fn compute_durations_single_entry() {
    let created = Utc::now() - Duration::hours(2);
    let e = make_audit("Created -> Specified", Utc::now());
    let (total, phases) = compute_durations_from_audit(&[e], &created);
    assert!(total > 0);
    assert!(phases.is_empty());
}

#[test]
fn compute_durations_two_entries() {
    let base = Utc::now();
    let t0 = base - Duration::hours(3);
    let t1 = base - Duration::hours(1);
    let e0 = make_audit("Created -> Specified", t0);
    let e1 = make_audit("Specified -> Researched", t1);
    let (total, phases) = compute_durations_from_audit(&[e0, e1], &t0);
    assert!(total > 0);
    assert_eq!(phases.len(), 1);
    assert!(phases[0].1 > 0);
}

#[test]
fn generate_insights_healthy() {
    let metrics = FeatureMetrics {
        total_duration_ms: 3600000,
        wp_count: 5,
        total_agent_runs: 10,
        total_review_cycles: 5,
        avg_review_cycles_per_wp: 1.0,
        state_transition_durations: vec![],
        governance_exceptions: vec![],
        high_review_wps: vec![],
        wp_metrics: vec![],
    };
    let insights = generate_insights(&metrics);
    assert!(!insights.is_empty());
    assert!(insights[0].contains("healthy") || insights[0].contains("No significant"));
}

#[test]
fn generate_insights_high_review_cycles() {
    let metrics = FeatureMetrics {
        total_duration_ms: 3600000,
        wp_count: 3,
        total_agent_runs: 20,
        total_review_cycles: 15,
        avg_review_cycles_per_wp: 5.0,
        state_transition_durations: vec![],
        governance_exceptions: vec![],
        high_review_wps: vec![(1, "Auth Module".to_string(), 5)],
        wp_metrics: vec![],
    };
    let insights = generate_insights(&metrics);
    let combined = insights.join(" ");
    assert!(combined.contains("review cycles") || combined.contains("bottleneck"));
}

#[test]
fn generate_retro_markdown_contains_summary() {
    let metrics = FeatureMetrics {
        total_duration_ms: 86400000,
        wp_count: 3,
        total_agent_runs: 6,
        total_review_cycles: 4,
        avg_review_cycles_per_wp: 1.3,
        state_transition_durations: vec![("Created -> Specified".to_string(), 3600000)],
        governance_exceptions: vec![],
        high_review_wps: vec![],
        wp_metrics: vec![WpMetrics {
            sequence: 1,
            title: "Auth Module".to_string(),
            agent_runs: 2,
            review_cycles: 2,
            duration_ms: 28800000,
        }],
    };
    let report = generate_retro_markdown("my-feat", "My Feature", &metrics, false);
    assert!(report.contains("# Retrospective: My Feature"));
    assert!(report.contains("## Summary"));
    assert!(report.contains("## WP Performance"));
    assert!(report.contains("## Insights"));
    assert!(report.contains("## Suggested Constitution Amendments"));
}
