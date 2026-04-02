use super::helpers::{store_cycle, store_feature, test_storage, transition_feature_to};
use agileplus_domain::{
    domain::{
        cycle::{CycleFeature, CycleState, CycleWithFeatures},
        state_machine::FeatureState,
    },
    ports::StoragePort,
};

/// A new cycle always starts in Draft state.
///
/// Traces to: FR-C01, FR-C02
#[tokio::test]
async fn cycle_create_defaults_to_draft() {
    let storage = test_storage();
    let cycle_id = store_cycle(&storage, "Sprint 1", None).await;

    let cycle = storage
        .get_cycle(cycle_id)
        .await
        .expect("get_cycle should succeed")
        .expect("cycle should exist");

    assert_eq!(cycle.state, CycleState::Draft);
}

/// A Draft cycle can be transitioned to Active.
///
/// Traces to: FR-C02
#[tokio::test]
async fn cycle_transition_draft_to_active() {
    let storage = test_storage();
    let cycle_id = store_cycle(&storage, "Sprint 2", None).await;

    storage
        .update_cycle_state(cycle_id, CycleState::Active)
        .await
        .expect("Draft->Active transition should succeed");

    let cycle = storage
        .get_cycle(cycle_id)
        .await
        .expect("get_cycle should succeed")
        .expect("cycle should exist");

    assert_eq!(cycle.state, CycleState::Active);
}

/// The Shipped gate must block when not all features are Validated or Shipped.
///
/// Traces to: FR-C07
#[tokio::test]
async fn cycle_shipped_gate_blocks_non_validated_features() {
    let storage = test_storage();
    let cycle_id = store_cycle(&storage, "Gate Cycle", None).await;
    let feature_id = store_feature(&storage, "unready-feat", "Unready Feature").await;

    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("add_feature_to_cycle should succeed");

    let cycle: CycleWithFeatures = storage
        .get_cycle_with_features(cycle_id)
        .await
        .expect("get_cycle_with_features should succeed")
        .expect("cycle should exist");

    assert!(
        !cycle.is_shippable(),
        "cycle with a Created-state feature must not be shippable"
    );
}

/// The Shipped gate allows shipping when all features are Validated.
///
/// Traces to: FR-C07
#[tokio::test]
async fn cycle_shipped_gate_allows_all_validated() {
    let storage = test_storage();
    let cycle_id = store_cycle(&storage, "Validated Cycle", None).await;
    let feature_id = store_feature(&storage, "ready-feat", "Ready Feature").await;

    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("add_feature_to_cycle should succeed");

    transition_feature_to(&storage, feature_id, FeatureState::Validated).await;

    let cycle = storage
        .get_cycle_with_features(cycle_id)
        .await
        .expect("get_cycle_with_features should succeed")
        .expect("cycle should exist");

    assert!(
        cycle.is_shippable(),
        "cycle where all features are Validated must be shippable"
    );
}

/// The Shipped gate allows shipping when all features are already Shipped.
///
/// Traces to: FR-C07
#[tokio::test]
async fn cycle_shipped_gate_allows_all_shipped_features() {
    let storage = test_storage();
    let cycle_id = store_cycle(&storage, "Shipped Cycle", None).await;
    let feature_id = store_feature(&storage, "shipped-feat", "Shipped Feature").await;

    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("add_feature_to_cycle should succeed");

    transition_feature_to(&storage, feature_id, FeatureState::Shipped).await;

    let cycle = storage
        .get_cycle_with_features(cycle_id)
        .await
        .expect("get_cycle_with_features should succeed")
        .expect("cycle should exist");

    assert!(
        cycle.is_shippable(),
        "cycle where all features are Shipped must be shippable"
    );
}

/// `get_cycle_with_features` returns a `WpProgressSummary` (may be zeroed for no WPs).
///
/// Traces to: FR-C05
#[tokio::test]
async fn cycle_show_returns_wp_progress() {
    let storage = test_storage();
    let cycle_id = store_cycle(&storage, "Progress Cycle", None).await;

    let cycle = storage
        .get_cycle_with_features(cycle_id)
        .await
        .expect("get_cycle_with_features should succeed")
        .expect("cycle should exist");

    assert_eq!(cycle.wp_progress.total, 0);
    assert_eq!(cycle.wp_progress.done, 0);
}
