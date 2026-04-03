use std::future::Future;

use agileplus_domain::domain::governance::PolicyRule;
use agileplus_domain::error::DomainError;

use super::MockStorage;

pub(crate) fn create_policy_rule(
    _storage: &MockStorage,
    _rule: &PolicyRule,
) -> impl Future<Output = Result<i64, DomainError>> + Send {
    async move { Ok(1) }
}

pub(crate) fn list_active_policies(
    _storage: &MockStorage,
) -> impl Future<Output = Result<Vec<PolicyRule>, DomainError>> + Send {
    async move { Ok(vec![]) }
}
