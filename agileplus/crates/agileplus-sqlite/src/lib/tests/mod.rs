use super::*;
use agileplus_domain::domain::{
    audit::{AuditEntry, hash_entry},
    feature::Feature,
    governance::{
        Evidence, EvidenceType, GovernanceContract, GovernanceRule, PolicyCheck, PolicyDefinition,
        PolicyDomain, PolicyRule,
    },
    metric::Metric,
    state_machine::FeatureState,
    work_package::{DependencyType, WorkPackage, WpDependency, WpState},
};
use agileplus_domain::ports::{ContentStoragePort, StoragePort};

mod feature_work_packages;
mod governance_metrics;
mod modules_cycles;

fn make_adapter() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().expect("in-memory adapter")
}

fn make_audit_entry(feature_id: i64, prev_hash: [u8; 32]) -> AuditEntry {
    let mut entry = AuditEntry {
        id: 0,
        feature_id,
        wp_id: None,
        timestamp: chrono::Utc::now(),
        actor: "agent".into(),
        transition: "created->specified".into(),
        evidence_refs: vec![],
        prev_hash,
        hash: [0u8; 32],
        event_id: None,
        archived_to: None,
    };
    entry.hash = hash_entry(&entry);
    entry
}

fn make_date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}
