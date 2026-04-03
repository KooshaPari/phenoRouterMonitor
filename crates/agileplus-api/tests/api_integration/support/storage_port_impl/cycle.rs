use std::future::Future;

use agileplus_domain::domain::cycle::{Cycle, CycleFeature, CycleState, CycleWithFeatures};
use agileplus_domain::domain::module::ModuleFeatureTag;
use agileplus_domain::domain::work_package::WpState;
use agileplus_domain::error::DomainError;

use super::MockStorage;

pub(crate) fn create_cycle(
    storage: &MockStorage,
    cycle: &Cycle,
) -> impl Future<Output = Result<i64, DomainError>> + Send {
    let id = (storage.cycles.lock().expect("cycles lock poisoned").len() + 1) as i64;
    {
        let mut cycles = storage.cycles.lock().expect("cycles lock poisoned");
        let mut created = cycle.clone();
        created.id = id;
        cycles.push(created);
    }
    async move { Ok(id) }
}

pub(crate) fn get_cycle(
    storage: &MockStorage,
    id: i64,
) -> impl Future<Output = Result<Option<Cycle>, DomainError>> + Send {
    let found = storage
        .cycles
        .lock()
        .expect("cycles lock poisoned")
        .iter()
        .find(|cycle| cycle.id == id)
        .cloned();
    async move { Ok(found) }
}

pub(crate) fn update_cycle_state(
    storage: &MockStorage,
    id: i64,
    state: CycleState,
) -> impl Future<Output = Result<(), DomainError>> + Send {
    let result = {
        let mut cycles = storage.cycles.lock().expect("cycles lock poisoned");
        match cycles.iter_mut().find(|cycle| cycle.id == id) {
            Some(cycle) => {
                cycle.state = state;
                cycle.updated_at = chrono::Utc::now();
                Ok(())
            }
            None => Err(DomainError::CycleNotFound(id.to_string())),
        }
    };
    async move { result }
}

pub(crate) fn list_cycles_by_state(
    storage: &MockStorage,
    state: CycleState,
) -> impl Future<Output = Result<Vec<Cycle>, DomainError>> + Send {
    let cycles = storage
        .cycles
        .lock()
        .expect("cycles lock poisoned")
        .iter()
        .filter(|cycle| cycle.state == state)
        .cloned()
        .collect();
    async move { Ok(cycles) }
}

pub(crate) fn list_cycles_by_module(
    storage: &MockStorage,
    module_id: i64,
) -> impl Future<Output = Result<Vec<Cycle>, DomainError>> + Send {
    let cycles = storage
        .cycles
        .lock()
        .expect("cycles lock poisoned")
        .iter()
        .filter(|cycle| cycle.module_scope_id == Some(module_id))
        .cloned()
        .collect();
    async move { Ok(cycles) }
}

pub(crate) fn list_all_cycles(
    storage: &MockStorage,
) -> impl Future<Output = Result<Vec<Cycle>, DomainError>> + Send {
    let cycles = storage.cycles.lock().expect("cycles lock poisoned").clone();
    async move { Ok(cycles) }
}

pub(crate) fn get_cycle_with_features(
    storage: &MockStorage,
    id: i64,
) -> impl Future<Output = Result<Option<CycleWithFeatures>, DomainError>> + Send {
    let cycle = storage
        .cycles
        .lock()
        .expect("cycles lock poisoned")
        .iter()
        .find(|cycle| cycle.id == id)
        .cloned();
    let result = if let Some(cycle) = cycle {
        let feature_ids: Vec<i64> = storage
            .cycle_features
            .lock()
            .expect("cycle_features lock poisoned")
            .iter()
            .filter(|entry| entry.cycle_id == id)
            .map(|entry| entry.feature_id)
            .collect();
        let features = storage
            .features
            .lock()
            .expect("features lock poisoned")
            .iter()
            .filter(|feature| feature_ids.contains(&feature.id))
            .cloned()
            .collect();
        let mut wp_progress = agileplus_domain::domain::cycle::WpProgressSummary::default();
        let feature_id_set = feature_ids;
        for wp in storage
            .work_packages
            .lock()
            .expect("work_packages lock poisoned")
            .iter()
            .filter(|wp| feature_id_set.contains(&wp.feature_id))
        {
            wp_progress.total += 1;
            match wp.state {
                WpState::Planned => wp_progress.planned += 1,
                WpState::Doing | WpState::Review => wp_progress.in_progress += 1,
                WpState::Done => wp_progress.done += 1,
                WpState::Blocked => wp_progress.blocked += 1,
            }
        }
        Some(CycleWithFeatures {
            cycle,
            features,
            wp_progress,
        })
    } else {
        None
    };
    async move { Ok(result) }
}

pub(crate) fn tag_feature_to_module(
    storage: &MockStorage,
    tag: &ModuleFeatureTag,
) -> impl Future<Output = Result<(), DomainError>> + Send {
    {
        let mut tags = storage
            .module_tags
            .lock()
            .expect("module_tags lock poisoned");
        if !tags.iter().any(|existing| {
            existing.module_id == tag.module_id && existing.feature_id == tag.feature_id
        }) {
            tags.push(tag.clone());
        }
    }
    async move { Ok(()) }
}

pub(crate) fn untag_feature_from_module(
    storage: &MockStorage,
    module_id: i64,
    feature_id: i64,
) -> impl Future<Output = Result<(), DomainError>> + Send {
    {
        let mut tags = storage
            .module_tags
            .lock()
            .expect("module_tags lock poisoned");
        tags.retain(|tag| !(tag.module_id == module_id && tag.feature_id == feature_id));
    }
    async move { Ok(()) }
}

pub(crate) fn add_feature_to_cycle(
    storage: &MockStorage,
    entry: &CycleFeature,
) -> impl Future<Output = Result<(), DomainError>> + Send {
    let result = {
        let cycle = storage
            .cycles
            .lock()
            .expect("cycles lock poisoned")
            .iter()
            .find(|cycle| cycle.id == entry.cycle_id)
            .cloned();
        let feature = storage
            .features
            .lock()
            .expect("features lock poisoned")
            .iter()
            .find(|feature| feature.id == entry.feature_id)
            .cloned();

        match (cycle, feature) {
            (None, _) => Err(DomainError::CycleNotFound(entry.cycle_id.to_string())),
            (_, None) => Err(DomainError::NotFound(entry.feature_id.to_string())),
            (Some(cycle), Some(feature)) => {
                if let Some(module_scope_id) = cycle.module_scope_id {
                    let owned = feature.module_id == Some(module_scope_id);
                    let tagged = storage
                        .module_tags
                        .lock()
                        .expect("module_tags lock poisoned")
                        .iter()
                        .any(|tag| {
                            tag.module_id == module_scope_id && tag.feature_id == feature.id
                        });
                    if !owned && !tagged {
                        let module_slug = storage
                            .modules
                            .lock()
                            .expect("modules lock poisoned")
                            .iter()
                            .find(|module| module.id == module_scope_id)
                            .map(|module| module.slug.clone())
                            .unwrap_or_else(|| module_scope_id.to_string());
                        Err(DomainError::FeatureNotInModuleScope {
                            feature_slug: feature.slug,
                            module_slug,
                        })
                    } else {
                        let mut joins = storage
                            .cycle_features
                            .lock()
                            .expect("cycle_features lock poisoned");
                        if !joins.iter().any(|join| {
                            join.cycle_id == entry.cycle_id && join.feature_id == entry.feature_id
                        }) {
                            joins.push(entry.clone());
                        }
                        Ok(())
                    }
                } else {
                    let mut joins = storage
                        .cycle_features
                        .lock()
                        .expect("cycle_features lock poisoned");
                    if !joins.iter().any(|join| {
                        join.cycle_id == entry.cycle_id && join.feature_id == entry.feature_id
                    }) {
                        joins.push(entry.clone());
                    }
                    Ok(())
                }
            }
        }
    };
    async move { result }
}

pub(crate) fn remove_feature_from_cycle(
    storage: &MockStorage,
    cycle_id: i64,
    feature_id: i64,
) -> impl Future<Output = Result<(), DomainError>> + Send {
    {
        let mut joins = storage
            .cycle_features
            .lock()
            .expect("cycle_features lock poisoned");
        joins.retain(|entry| !(entry.cycle_id == cycle_id && entry.feature_id == feature_id));
    }
    async move { Ok(()) }
}
