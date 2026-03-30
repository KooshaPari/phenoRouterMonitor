use std::future::Future;

use agileplus_domain::domain::governance::Evidence;
use agileplus_domain::error::DomainError;

use super::MockStorage;

pub(crate) fn create_evidence(
    _storage: &MockStorage,
    _evidence: &Evidence,
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
