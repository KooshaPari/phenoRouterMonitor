use super::helpers::{store_cycle, store_feature, store_module, test_storage};
use agileplus_domain::{
    domain::{cycle::CycleFeature, state_machine::FeatureState},
    error::DomainError,
    ports::StoragePort,
};

/// A feature without a module_id can be created (NULL module_id is allowed).
///
/// Traces to: FR-M03
#[tokio::test]
async fn feature_without_module_id_can_be_created() {
    let storage = test_storage();
    let feature_id = store_feature(&storage, "legacy-feat", "Legacy Feature").await;

    let feature = storage
        .get_feature_by_id(feature_id)
        .await
        .expect("get_feature_by_id should succeed")
        .expect("feature should exist");

    assert_eq!(
        feature.module_id, None,
        "new feature must have no module_id by default"
    );
}

/// A feature without a module_id appears in `list_all_features` normally.
///
/// Traces to: FR-M03
#[tokio::test]
async fn feature_without_module_id_lists_normally() {
    let storage = test_storage();
    let feature_id = store_feature(&storage, "unmodule-feat", "Unmodule Feature").await;

    let features = storage
        .list_all_features()
        .await
        .expect("list_all_features should succeed");

    let found = features.iter().any(|feature| feature.id == feature_id);
    assert!(
        found,
        "feature without module_id must appear in list_all_features"
    );
}

/// A feature without a module_id can be added to an unscoped cycle.
///
/// Traces to: FR-C03, FR-M03
#[tokio::test]
async fn feature_without_module_id_can_be_added_to_unscoped_cycle() {
    let storage = test_storage();
    let cycle_id = store_cycle(&storage, "Compat Cycle", None).await;
    let feature_id = store_feature(&storage, "compat-feat", "Compat Feature").await;

    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("unscoped cycle should accept a feature without module_id");
}

/// A feature without a module_id is rejected by a scoped cycle.
///
/// Traces to: FR-C04, FR-M03
#[tokio::test]
async fn feature_without_module_id_rejected_by_scoped_cycle() {
    let storage = test_storage();
    let module_id = store_module(&storage, "Strict Module", None).await;
    let cycle_id = store_cycle(&storage, "Strict Cycle", Some(module_id)).await;
    let feature_id = store_feature(&storage, "no-module-feat", "No Module Feature").await;

    let result = storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await;

    assert!(
        matches!(result, Err(DomainError::FeatureNotInModuleScope { .. })),
        "expected FeatureNotInModuleScope for feature with no module_id in scoped cycle, got: {result:?}"
    );
}

/// State transitions work normally on features that have no module_id.
///
/// Traces to: FR-M03
#[tokio::test]
async fn existing_feature_transitions_work_without_module_id() {
    let storage = test_storage();
    let feature_id = store_feature(&storage, "trans-feat", "Transition Feature").await;

    storage
        .update_feature_state(feature_id, FeatureState::Specified)
        .await
        .expect("state transition should succeed for feature without module_id");

    let feature = storage
        .get_feature_by_id(feature_id)
        .await
        .expect("get_feature_by_id should succeed")
        .expect("feature should exist");

    assert_eq!(feature.state, FeatureState::Specified);
    assert_eq!(
        feature.module_id, None,
        "module_id should remain None after transition"
    );
}
