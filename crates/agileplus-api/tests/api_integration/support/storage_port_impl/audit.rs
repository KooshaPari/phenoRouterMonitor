use std::future::Future;

use agileplus_domain::domain::audit::AuditEntry;
use agileplus_domain::error::DomainError;

use super::MockStorage;

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
