use std::future::Future;

use agileplus_domain::domain::audit::AuditEntry;
use agileplus_domain::domain::governance::{Evidence, GovernanceContract, PolicyRule};
use agileplus_domain::domain::metric::Metric;
use agileplus_domain::domain::work_package::{WorkPackage, WpDependency, WpState};
use agileplus_domain::error::DomainError;

use super::MockStorage;

pub(crate) fn create_work_package(
    storage: &MockStorage,
    wp: &WorkPackage,
) -> impl Future<Output = Result<i64, DomainError>> + Send {
    let id = (storage
        .work_packages
        .lock()
        .expect("work_packages lock poisoned")
        .len()
        + 1) as i64;
    {
        let mut work_packages = storage
            .work_packages
            .lock()
            .expect("work_packages lock poisoned");
        let mut created = wp.clone();
        created.id = id;
        work_packages.push(created);
    }
    async move { Ok(id) }
}

pub(crate) fn get_work_package(
    storage: &MockStorage,
    id: i64,
) -> impl Future<Output = Result<Option<WorkPackage>, DomainError>> + Send {
    let found = storage
        .work_packages
        .lock()
        .expect("work_packages lock poisoned")
        .iter()
        .find(|w| w.id == id)
        .cloned();
    async move { Ok(found) }
}

pub(crate) fn update_wp_state(
    storage: &MockStorage,
    id: i64,
    state: WpState,
) -> impl Future<Output = Result<(), DomainError>> + Send {
    {
        let mut work_packages = storage
            .work_packages
            .lock()
            .expect("work_packages lock poisoned");
        if let Some(wp) = work_packages.iter_mut().find(|w| w.id == id) {
            wp.state = state;
            wp.updated_at = chrono::Utc::now();
        }
    }
    async move { Ok(()) }
}

pub(crate) fn update_work_package(
    storage: &MockStorage,
    wp: &WorkPackage,
) -> impl Future<Output = Result<(), DomainError>> + Send {
    {
        let mut work_packages = storage
            .work_packages
            .lock()
            .expect("work_packages lock poisoned");
        if let Some(existing) = work_packages.iter_mut().find(|w| w.id == wp.id) {
            *existing = wp.clone();
        }
    }
    async move { Ok(()) }
}

pub(crate) fn list_wps_by_feature(
    storage: &MockStorage,
    feature_id: i64,
) -> impl Future<Output = Result<Vec<WorkPackage>, DomainError>> + Send {
    let wps: Vec<WorkPackage> = storage
        .work_packages
        .lock()
        .expect("work_packages lock poisoned")
        .iter()
        .filter(|w| w.feature_id == feature_id)
        .cloned()
        .collect();
    async move { Ok(wps) }
}

pub(crate) fn add_wp_dependency(
    _storage: &MockStorage,
    _dep: &WpDependency,
) -> impl Future<Output = Result<(), DomainError>> + Send {
    async move { Ok(()) }
}

pub(crate) fn get_wp_dependencies(
    _storage: &MockStorage,
    _wp_id: i64,
) -> impl Future<Output = Result<Vec<WpDependency>, DomainError>> + Send {
    async move { Ok(vec![]) }
}

pub(crate) fn get_ready_wps(
    storage: &MockStorage,
    feature_id: i64,
) -> impl Future<Output = Result<Vec<WorkPackage>, DomainError>> + Send {
    let wps: Vec<WorkPackage> = storage
        .work_packages
        .lock()
        .expect("work_packages lock poisoned")
        .iter()
        .filter(|w| w.feature_id == feature_id)
        .cloned()
        .collect();
    async move { Ok(wps) }
}

pub(crate) fn append_audit_entry(
    storage: &MockStorage,
    entry: &AuditEntry,
) -> impl Future<Output = Result<i64, DomainError>> + Send {
    let id = (storage.audit.lock().expect("audit lock poisoned").len() + 1) as i64;
    let _ = entry;
    async move { Ok(id) }
}

pub(crate) fn get_audit_trail(
    storage: &MockStorage,
    feature_id: i64,
) -> impl Future<Output = Result<Vec<AuditEntry>, DomainError>> + Send {
    let trail: Vec<AuditEntry> = storage
        .audit
        .lock()
        .expect("audit lock poisoned")
        .iter()
        .filter(|e| e.feature_id == feature_id)
        .cloned()
        .collect();
    async move { Ok(trail) }
}

pub(crate) fn get_latest_audit_entry(
    storage: &MockStorage,
    feature_id: i64,
) -> impl Future<Output = Result<Option<AuditEntry>, DomainError>> + Send {
    let entry = storage
        .audit
        .lock()
        .expect("audit lock poisoned")
        .iter()
        .filter(|e| e.feature_id == feature_id)
        .last()
        .cloned();
    async move { Ok(entry) }
}

pub(crate) fn create_evidence(
    _storage: &MockStorage,
    _e: &Evidence,
) -> impl Future<Output = Result<i64, DomainError>> + Send {
    async move { Ok(1) }
}

pub(crate) fn get_evidence_by_wp(
    _storage: &MockStorage,
    _wp_id: i64,
) -> impl Future<Output = Result<Vec<Evidence>, DomainError>> + Send {
    async move { Ok(vec![]) }
}

pub(crate) fn get_evidence_by_fr(
    _storage: &MockStorage,
    _fr_id: &str,
) -> impl Future<Output = Result<Vec<Evidence>, DomainError>> + Send {
    async move { Ok(vec![]) }
}

pub(crate) fn create_policy_rule(
    _storage: &MockStorage,
    _r: &PolicyRule,
) -> impl Future<Output = Result<i64, DomainError>> + Send {
    async move { Ok(1) }
}

pub(crate) fn list_active_policies(
    _storage: &MockStorage,
) -> impl Future<Output = Result<Vec<PolicyRule>, DomainError>> + Send {
    async move { Ok(vec![]) }
}

pub(crate) fn record_metric(
    _storage: &MockStorage,
    _m: &Metric,
) -> impl Future<Output = Result<i64, DomainError>> + Send {
    async move { Ok(1) }
}

pub(crate) fn get_metrics_by_feature(
    _storage: &MockStorage,
    _feature_id: i64,
) -> impl Future<Output = Result<Vec<Metric>, DomainError>> + Send {
    async move { Ok(vec![]) }
}

pub(crate) fn create_governance_contract(
    _storage: &MockStorage,
    _c: &GovernanceContract,
) -> impl Future<Output = Result<i64, DomainError>> + Send {
    async move { Ok(1) }
}

pub(crate) fn get_governance_contract(
    storage: &MockStorage,
    feature_id: i64,
    version: i32,
) -> impl Future<Output = Result<Option<GovernanceContract>, DomainError>> + Send {
    let found = storage
        .governance
        .lock()
        .expect("governance lock poisoned")
        .iter()
        .find(|c| c.feature_id == feature_id && c.version == version)
        .cloned();
    async move { Ok(found) }
}

pub(crate) fn get_latest_governance_contract(
    storage: &MockStorage,
    feature_id: i64,
) -> impl Future<Output = Result<Option<GovernanceContract>, DomainError>> + Send {
    let found = storage
        .governance
        .lock()
        .expect("governance lock poisoned")
        .iter()
        .filter(|c| c.feature_id == feature_id)
        .max_by_key(|c| c.version)
        .cloned();
    async move { Ok(found) }
}
