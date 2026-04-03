use std::future::Future;

use agileplus_domain::domain::sync_mapping::SyncMapping;
use agileplus_domain::error::DomainError;

use super::MockStorage;

pub(crate) fn get_sync_mapping(
    _storage: &MockStorage,
    _entity_type: &str,
    _entity_id: i64,
) -> impl Future<Output = Result<Option<SyncMapping>, DomainError>> + Send {
    async move { Ok(None) }
}

pub(crate) fn upsert_sync_mapping(
    _storage: &MockStorage,
    _mapping: &SyncMapping,
) -> impl Future<Output = Result<(), DomainError>> + Send {
    async move { Ok(()) }
}

pub(crate) fn get_sync_mapping_by_plane_id(
    _storage: &MockStorage,
    _entity_type: &str,
    _plane_issue_id: &str,
) -> impl Future<Output = Result<Option<SyncMapping>, DomainError>> + Send {
    async move { Ok(None) }
}

pub(crate) fn delete_sync_mapping(
    _storage: &MockStorage,
    _entity_type: &str,
    _entity_id: i64,
) -> impl Future<Output = Result<(), DomainError>> + Send {
    async move { Ok(()) }
}
