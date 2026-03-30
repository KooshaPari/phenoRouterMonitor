use super::helpers::{
    assign_feature_module_id, store_cycle, store_feature, store_module, test_storage,
};
use agileplus_domain::{
    domain::{cycle::CycleFeature, module::ModuleFeatureTag},
    error::DomainError,
    ports::StoragePort,
};

/// A cycle created with a module_scope_id stores that scope.
///
/// Traces to: FR-C04
#[tokio::test]
async fn scoped_cycle_created_with_module_scope() {
    let storage = test_storage();
    let module_id = store_module(&storage, "Scoped Module", None).await;
    let cycle_id = store_cycle(&storage, "Scoped Cycle", Some(module_id)).await;

    let cycle = storage
        .get_cycle(cycle_id)
        .await
        .expect("get_cycle should succeed")
        .expect("cycle should exist");

    assert_eq!(cycle.module_scope_id, Some(module_id));
}

/// A scoped cycle must reject features not owned by or tagged to the scope module.
///
/// Traces to: FR-C04, FR-C06
#[tokio::test]
async fn scoped_cycle_rejects_out_of_scope_feature() {
    let storage = test_storage();
    let scope_module_id = store_module(&storage, "Scope Module", None).await;
    let other_module_id = store_module(&storage, "Other Module", None).await;
    let cycle_id = store_cycle(&storage, "Rejecting Cycle", Some(scope_module_id)).await;
    let feature_id = store_feature(&storage, "out-of-scope", "Out Of Scope").await;
    assign_feature_module_id(&storage, feature_id, other_module_id);

    let result = storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await;

    assert!(
        matches!(result, Err(DomainError::FeatureNotInModuleScope { .. })),
        "expected FeatureNotInModuleScope, got: {result:?}"
    );
}

/// A scoped cycle must accept a feature owned by the scope module.
///
/// Traces to: FR-C04
#[tokio::test]
async fn scoped_cycle_accepts_owned_feature() {
    let storage = test_storage();
    let module_id = store_module(&storage, "Owner Module", None).await;
    let cycle_id = store_cycle(&storage, "Owner Cycle", Some(module_id)).await;
    let feature_id = store_feature(&storage, "owned-scope-feat", "Owned Scope Feature").await;
    assign_feature_module_id(&storage, feature_id, module_id);

    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("scoped cycle should accept an owned feature");

    let cycle = storage
        .get_cycle_with_features(cycle_id)
        .await
        .expect("get_cycle_with_features should succeed")
        .expect("cycle should exist");

    let feature_ids: Vec<i64> = cycle.features.iter().map(|feature| feature.id).collect();
    assert!(feature_ids.contains(&feature_id));
}

/// A scoped cycle must accept a feature tagged (not owned) to the scope module.
///
/// Traces to: FR-C04
#[tokio::test]
async fn scoped_cycle_accepts_tagged_feature() {
    let storage = test_storage();
    let module_id = store_module(&storage, "Tag Module", None).await;
    let cycle_id = store_cycle(&storage, "Tag Cycle", Some(module_id)).await;
    let feature_id = store_feature(&storage, "tagged-scope-feat", "Tagged Scope Feature").await;

    storage
        .tag_feature_to_module(&ModuleFeatureTag::new(module_id, feature_id))
        .await
        .expect("tag should succeed");

    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("scoped cycle should accept a tagged feature");
}

/// An unscoped cycle accepts any feature regardless of module membership.
///
/// Traces to: FR-C03
#[tokio::test]
async fn unscoped_cycle_accepts_any_feature() {
    let storage = test_storage();
    let cycle_id = store_cycle(&storage, "Unscoped Cycle", None).await;
    let feature_id = store_feature(&storage, "any-feat", "Any Feature").await;

    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("unscoped cycle should accept any feature");
}
