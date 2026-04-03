use std::future::Future;

use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::error::DomainError;

use super::MockStorage;

pub(crate) fn create_feature(
    storage: &MockStorage,
    f: &Feature,
) -> impl Future<Output = Result<i64, DomainError>> + Send {
    let id = (storage
        .features
        .lock()
        .expect("features lock poisoned")
        .len()
        + 1) as i64;
    {
        let mut features = storage.features.lock().expect("features lock poisoned");
        let mut feature = f.clone();
        feature.id = id;
        features.push(feature);
    }
    async move { Ok(id) }
}

pub(crate) fn get_feature_by_slug(
    storage: &MockStorage,
    slug: &str,
) -> impl Future<Output = Result<Option<Feature>, DomainError>> + Send {
    let found = storage
        .features
        .lock()
        .expect("features lock poisoned")
        .iter()
        .find(|f| f.slug == slug)
        .cloned();
    async move { Ok(found) }
}

pub(crate) fn get_feature_by_id(
    storage: &MockStorage,
    id: i64,
) -> impl Future<Output = Result<Option<Feature>, DomainError>> + Send {
    let found = storage
        .features
        .lock()
        .expect("features lock poisoned")
        .iter()
        .find(|f| f.id == id)
        .cloned();
    async move { Ok(found) }
}

pub(crate) fn update_feature_state(
    storage: &MockStorage,
    id: i64,
    state: FeatureState,
) -> impl Future<Output = Result<(), DomainError>> + Send {
    {
        let mut features = storage.features.lock().expect("features lock poisoned");
        if let Some(feature) = features.iter_mut().find(|f| f.id == id) {
            feature.state = state;
            feature.updated_at = chrono::Utc::now();
        }
    }
    async move { Ok(()) }
}

pub(crate) fn update_feature(
    storage: &MockStorage,
    feature: &Feature,
) -> impl Future<Output = Result<(), DomainError>> + Send {
    {
        let mut features = storage.features.lock().expect("features lock poisoned");
        if let Some(existing) = features.iter_mut().find(|f| f.id == feature.id) {
            *existing = feature.clone();
        }
    }
    async move { Ok(()) }
}

pub(crate) fn list_features_by_state(
    storage: &MockStorage,
    state: FeatureState,
) -> impl Future<Output = Result<Vec<Feature>, DomainError>> + Send {
    let features: Vec<Feature> = storage
        .features
        .lock()
        .expect("features lock poisoned")
        .iter()
        .filter(|f| f.state == state)
        .cloned()
        .collect();
    async move { Ok(features) }
}

pub(crate) fn list_all_features(
    storage: &MockStorage,
) -> impl Future<Output = Result<Vec<Feature>, DomainError>> + Send {
    let features = storage
        .features
        .lock()
        .expect("features lock poisoned")
        .clone();
    async move { Ok(features) }
}
