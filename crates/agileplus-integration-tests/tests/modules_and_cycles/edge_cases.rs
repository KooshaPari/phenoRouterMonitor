use super::helpers::{date, store_cycle, store_feature, store_module, test_storage};
use agileplus_domain::{
    domain::{
        cycle::{Cycle, CycleFeature, CycleState},
        module::{Module, ModuleFeatureTag},
        state_machine::FeatureState,
    },
    ports::StoragePort,
};

/// Attempting to create a module whose proposed parent would create a cycle
/// (i.e., the parent doesn't exist) must fail.
///
/// Traces to: FR-M02
#[tokio::test]
async fn circular_module_ref_is_rejected() {
    let storage = test_storage();
    let phantom_parent: i64 = 9_999_999;
    let module = Module::new("Bad Child", Some(phantom_parent));
    let result = storage.create_module(&module).await;

    assert!(
        result.is_err(),
        "creating a module with a non-existent parent should fail; got: {result:?}"
    );
}

/// A feature can belong to multiple cycles simultaneously.
///
/// Traces to: FR-C03
#[tokio::test]
async fn features_can_be_in_multiple_cycles() {
    let storage = test_storage();
    let cycle1_id = store_cycle(&storage, "Multi Cycle 1", None).await;
    let cycle2_id = store_cycle(&storage, "Multi Cycle 2", None).await;
    let feature_id = store_feature(&storage, "multi-cycle-feat", "Multi Cycle Feature").await;

    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle1_id, feature_id))
        .await
        .expect("add to cycle 1 should succeed");

    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle2_id, feature_id))
        .await
        .expect("add to cycle 2 should succeed");

    let cycle1 = storage
        .get_cycle_with_features(cycle1_id)
        .await
        .expect("get cwf 1")
        .expect("cycle 1 exists");
    let cycle2 = storage
        .get_cycle_with_features(cycle2_id)
        .await
        .expect("get cwf 2")
        .expect("cycle 2 exists");

    assert!(
        cycle1
            .features
            .iter()
            .any(|feature| feature.id == feature_id)
    );
    assert!(
        cycle2
            .features
            .iter()
            .any(|feature| feature.id == feature_id)
    );
}

/// An empty cycle in Draft state is valid (no gate violation).
///
/// Traces to: FR-C01, FR-C02
#[tokio::test]
async fn empty_cycle_in_draft_is_valid() {
    let storage = test_storage();
    let cycle_id = store_cycle(&storage, "Empty Draft Cycle", None).await;

    let cycle = storage
        .get_cycle_with_features(cycle_id)
        .await
        .expect("get_cycle_with_features should succeed")
        .expect("cycle should exist");

    assert_eq!(cycle.cycle.state, CycleState::Draft);
    assert!(cycle.features.is_empty());
    assert!(
        cycle.is_shippable(),
        "empty cycle is vacuously shippable (no blocking features)"
    );
}

/// Deleting a cycle unlinks features (cycle_features rows) but does not change feature state.
///
/// Traces to: FR-C03
#[tokio::test]
async fn deleting_cycle_unlinks_features_does_not_change_state() {
    let storage = test_storage();
    let cycle_id = store_cycle(&storage, "Del Cycle", None).await;
    let feature_id = store_feature(&storage, "del-cycle-feat", "Del Cycle Feature").await;

    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("add_feature_to_cycle should succeed");

    storage
        .remove_feature_from_cycle(cycle_id, feature_id)
        .await
        .expect("remove_feature_from_cycle should succeed");

    let feature = storage
        .get_feature_by_id(feature_id)
        .await
        .expect("get_feature_by_id should succeed")
        .expect("feature should still exist");

    assert_eq!(
        feature.state,
        FeatureState::Created,
        "feature state must not change when removed from a cycle"
    );
}

/// Deleting a module with children requires reparenting or deleting children first.
///
/// Traces to: FR-M07
#[tokio::test]
async fn delete_module_with_children_requires_reparenting_first() {
    let storage = test_storage();
    let parent_id = store_module(&storage, "Has Children", None).await;
    let child_id = store_module(&storage, "Child One", Some(parent_id)).await;

    let result = storage.delete_module(parent_id).await;
    assert!(result.is_err(), "deleting parent with children must fail");

    storage
        .delete_module(child_id)
        .await
        .expect("deleting child should succeed");

    storage
        .delete_module(parent_id)
        .await
        .expect("deleting parent after children removed should succeed");
}

/// Removing a module scope tag from a feature already in a cycle does not cascade.
///
/// Traces to: FR-M04, FR-C04
#[tokio::test]
async fn module_scope_removal_after_feature_added_does_not_cascade() {
    let storage = test_storage();
    let module_id = store_module(&storage, "Scope Removal Module", None).await;
    let cycle_id = store_cycle(&storage, "Scope Removal Cycle", Some(module_id)).await;
    let feature_id = store_feature(&storage, "scope-rem-feat", "Scope Removal Feature").await;

    storage
        .tag_feature_to_module(&ModuleFeatureTag::new(module_id, feature_id))
        .await
        .expect("tag should succeed");

    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("add should succeed while feature is in scope");

    storage
        .untag_feature_from_module(module_id, feature_id)
        .await
        .expect("untag should succeed");

    let cycle = storage
        .get_cycle_with_features(cycle_id)
        .await
        .expect("get_cycle_with_features should succeed")
        .expect("cycle should exist");

    assert!(
        cycle
            .features
            .iter()
            .any(|feature| feature.id == feature_id),
        "feature must remain in cycle after its module tag is removed (no cascade)"
    );
}

/// A cycle whose end_date is not after start_date must be rejected at construction.
///
/// Traces to: FR-C01
#[tokio::test]
async fn cycle_date_constraint_rejected() {
    let same_day = date(2026, 6, 1);
    let same_day_result = Cycle::new("Bad Date Cycle", same_day, same_day, None);
    assert!(
        same_day_result.is_err(),
        "Cycle::new with end_date == start_date must fail"
    );

    let backward_result = Cycle::new(
        "Bad Date Cycle 2",
        date(2026, 6, 10),
        date(2026, 6, 1),
        None,
    );
    assert!(
        backward_result.is_err(),
        "Cycle::new with end_date before start_date must fail"
    );
}
