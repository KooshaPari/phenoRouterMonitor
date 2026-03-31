use std::future::Future;

use agileplus_domain::domain::module::{Module, ModuleWithFeatures};
use agileplus_domain::error::DomainError;

use super::MockStorage;

pub(crate) fn create_module(
    storage: &MockStorage,
    module: &Module,
) -> impl Future<Output = Result<i64, DomainError>> + Send {
    let id = (storage.modules.lock().expect("modules lock poisoned").len() + 1) as i64;
    {
        let mut modules = storage.modules.lock().expect("modules lock poisoned");
        let mut created = module.clone();
        created.id = id;
        modules.push(created);
    }
    async move { Ok(id) }
}

pub(crate) fn get_module(
    storage: &MockStorage,
    id: i64,
) -> impl Future<Output = Result<Option<Module>, DomainError>> + Send {
    let found = storage
        .modules
        .lock()
        .expect("modules lock poisoned")
        .iter()
        .find(|module| module.id == id)
        .cloned();
    async move { Ok(found) }
}

pub(crate) fn get_module_by_slug(
    storage: &MockStorage,
    slug: &str,
) -> impl Future<Output = Result<Option<Module>, DomainError>> + Send {
    let found = storage
        .modules
        .lock()
        .expect("modules lock poisoned")
        .iter()
        .find(|module| module.slug == slug)
        .cloned();
    async move { Ok(found) }
}

pub(crate) fn update_module(
    storage: &MockStorage,
    id: i64,
    friendly_name: &str,
    description: Option<&str>,
) -> impl Future<Output = Result<(), DomainError>> + Send {
    let friendly_name = friendly_name.to_string();
    let description = description.map(ToString::to_string);
    let result = {
        let mut modules = storage.modules.lock().expect("modules lock poisoned");
        match modules.iter_mut().find(|module| module.id == id) {
            Some(module) => {
                module.friendly_name = friendly_name;
                module.slug = Module::slug_from_name(&module.friendly_name);
                module.description = description;
                module.updated_at = chrono::Utc::now();
                Ok(())
            }
            None => Err(DomainError::ModuleNotFound(id.to_string())),
        }
    };
    async move { result }
}

pub(crate) fn delete_module(
    storage: &MockStorage,
    id: i64,
) -> impl Future<Output = Result<(), DomainError>> + Send {
    let result = {
        let child_count = {
            let modules = storage.modules.lock().expect("modules lock poisoned");
            modules
                .iter()
                .filter(|module| module.parent_module_id == Some(id))
                .count()
        };
        if child_count > 0 {
            Err(DomainError::ModuleHasDependents(format!(
                "module {id} has {child_count} child module(s)"
            )))
        } else {
            let owned_feature_count = {
                let features = storage.features.lock().expect("features lock poisoned");
                features
                    .iter()
                    .filter(|feature| feature.module_id == Some(id))
                    .count()
            };
            if owned_feature_count > 0 {
                Err(DomainError::ModuleHasDependents(format!(
                    "module {id} owns {owned_feature_count} feature(s)"
                )))
            } else {
                let deleted = {
                    let mut modules = storage.modules.lock().expect("modules lock poisoned");
                    let before = modules.len();
                    modules.retain(|module| module.id != id);
                    modules.len() != before
                };
                if !deleted {
                    Err(DomainError::ModuleNotFound(id.to_string()))
                } else {
                    let mut tags = storage
                        .module_tags
                        .lock()
                        .expect("module_tags lock poisoned");
                    tags.retain(|tag| tag.module_id != id);
                    Ok(())
                }
            }
        }
    };
    async move { result }
}

pub(crate) fn list_root_modules(
    storage: &MockStorage,
) -> impl Future<Output = Result<Vec<Module>, DomainError>> + Send {
    let modules = storage
        .modules
        .lock()
        .expect("modules lock poisoned")
        .iter()
        .filter(|module| module.parent_module_id.is_none())
        .cloned()
        .collect();
    async move { Ok(modules) }
}

pub(crate) fn list_child_modules(
    storage: &MockStorage,
    parent_id: i64,
) -> impl Future<Output = Result<Vec<Module>, DomainError>> + Send {
    let modules = storage
        .modules
        .lock()
        .expect("modules lock poisoned")
        .iter()
        .filter(|module| module.parent_module_id == Some(parent_id))
        .cloned()
        .collect();
    async move { Ok(modules) }
}

pub(crate) fn get_module_with_features(
    storage: &MockStorage,
    id: i64,
) -> impl Future<Output = Result<Option<ModuleWithFeatures>, DomainError>> + Send {
    let module = storage
        .modules
        .lock()
        .expect("modules lock poisoned")
        .iter()
        .find(|module| module.id == id)
        .cloned();
    let result = if let Some(module) = module {
        let owned_features = storage
            .features
            .lock()
            .expect("features lock poisoned")
            .iter()
            .filter(|feature| feature.module_id == Some(id))
            .cloned()
            .collect();
        let tagged_feature_ids: Vec<i64> = storage
            .module_tags
            .lock()
            .expect("module_tags lock poisoned")
            .iter()
            .filter(|tag| tag.module_id == id)
            .map(|tag| tag.feature_id)
            .collect();
        let tagged_features = storage
            .features
            .lock()
            .expect("features lock poisoned")
            .iter()
            .filter(|feature| tagged_feature_ids.contains(&feature.id))
            .cloned()
            .collect();
        let child_modules = storage
            .modules
            .lock()
            .expect("modules lock poisoned")
            .iter()
            .filter(|child| child.parent_module_id == Some(id))
            .cloned()
            .collect();
        Some(ModuleWithFeatures {
            module,
            owned_features,
            tagged_features,
            child_modules,
        })
    } else {
        None
    };
    async move { Ok(result) }
}
