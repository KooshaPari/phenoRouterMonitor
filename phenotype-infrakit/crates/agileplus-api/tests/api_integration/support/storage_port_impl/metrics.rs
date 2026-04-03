use std::future::Future;

use agileplus_domain::domain::metric::Metric;
use agileplus_domain::error::DomainError;

use super::MockStorage;

pub(crate) fn record_metric(
    _storage: &MockStorage,
    _metric: &Metric,
) -> impl Future<Output = Result<i64, DomainError>> + Send {
    async move { Ok(1) }
}

pub(crate) fn get_metrics_by_feature(
    _storage: &MockStorage,
    _feature_id: i64,
) -> impl Future<Output = Result<Vec<Metric>, DomainError>> + Send {
    async move { Ok(vec![]) }
}
