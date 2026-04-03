use std::future::Future;

use agileplus_domain::domain::backlog::{
    BacklogFilters, BacklogItem, BacklogPriority, BacklogSort, BacklogStatus,
};
use agileplus_domain::error::DomainError;

use super::MockStorage;

pub(crate) fn get_backlog_item(
    storage: &MockStorage,
    id: i64,
) -> impl Future<Output = Result<Option<BacklogItem>, DomainError>> + Send {
    let found = storage
        .backlog
        .lock()
        .expect("backlog lock poisoned")
        .iter()
        .find(|item| item.id == Some(id))
        .cloned();
    async move { Ok(found) }
}

pub(crate) fn list_backlog_items(
    storage: &MockStorage,
    filters: &BacklogFilters,
) -> impl Future<Output = Result<Vec<BacklogItem>, DomainError>> + Send {
    let mut items = storage
        .backlog
        .lock()
        .expect("backlog lock poisoned")
        .clone();

    if let Some(intent) = filters.intent {
        items.retain(|item| item.intent == intent);
    }
    if let Some(status) = filters.status {
        items.retain(|item| item.status == status);
    }
    if let Some(priority) = filters.priority {
        items.retain(|item| item.priority == priority);
    }
    if let Some(feature_slug) = &filters.feature_slug {
        items.retain(|item| item.feature_slug.as_deref() == Some(feature_slug.as_str()));
    }
    if let Some(source) = &filters.source {
        items.retain(|item| item.source == *source);
    }

    match filters.sort {
        BacklogSort::Age => items.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
        BacklogSort::Priority | BacklogSort::Impact => items.sort_by(|a, b| {
            (a.priority.rank(), a.created_at).cmp(&(b.priority.rank(), b.created_at))
        }),
    }

    if let Some(limit) = filters.limit {
        items.truncate(limit);
    }

    async move { Ok(items) }
}

pub(crate) fn create_backlog_item(
    storage: &MockStorage,
    item: &BacklogItem,
) -> impl Future<Output = Result<i64, DomainError>> + Send {
    let id = (storage.backlog.lock().expect("backlog lock poisoned").len() + 1) as i64;
    {
        let mut backlog = storage.backlog.lock().expect("backlog lock poisoned");
        let mut created = item.clone();
        created.id = Some(id);
        backlog.push(created);
    }
    async move { Ok(id) }
}

pub(crate) fn update_backlog_status(
    storage: &MockStorage,
    id: i64,
    status: BacklogStatus,
) -> impl Future<Output = Result<(), DomainError>> + Send {
    {
        let mut backlog = storage.backlog.lock().expect("backlog lock poisoned");
        if let Some(item) = backlog.iter_mut().find(|item| item.id == Some(id)) {
            item.status = status;
            item.updated_at = chrono::Utc::now();
        }
    }
    async move { Ok(()) }
}

pub(crate) fn update_backlog_priority(
    storage: &MockStorage,
    id: i64,
    priority: BacklogPriority,
) -> impl Future<Output = Result<(), DomainError>> + Send {
    {
        let mut backlog = storage.backlog.lock().expect("backlog lock poisoned");
        if let Some(item) = backlog.iter_mut().find(|item| item.id == Some(id)) {
            item.priority = priority;
            item.updated_at = chrono::Utc::now();
        }
    }
    async move { Ok(()) }
}

pub(crate) fn pop_next_backlog_item(
    storage: &MockStorage,
) -> impl Future<Output = Result<Option<BacklogItem>, DomainError>> + Send {
    let mut backlog = storage.backlog.lock().expect("backlog lock poisoned");
    let next = backlog
        .iter()
        .filter(|item| item.status.is_open())
        .min_by(|a, b| (a.priority.rank(), a.created_at).cmp(&(b.priority.rank(), b.created_at)))
        .cloned();

    if let Some(item) = next.clone() {
        if let Some(id) = item.id {
            if let Some(existing) = backlog.iter_mut().find(|entry| entry.id == Some(id)) {
                existing.status = BacklogStatus::Triaged;
                existing.updated_at = chrono::Utc::now();
            }
        }
    }

    async move { Ok(next) }
}
