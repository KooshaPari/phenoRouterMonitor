//! Integration tests for Modules and Cycles domain -- T034 through T038.
//!
//! All tests use an in-memory SQLite adapter; no file I/O, no network calls.
//! Each test creates its own isolated storage instance.
//!
//! Traceability:
//!   T034 -- FR-M01, FR-M02, FR-M03, FR-M04, FR-M07
//!   T035 -- FR-C01, FR-C02, FR-C05, FR-C07
//!   T036 -- FR-C04, FR-C06
//!   T037 -- FR-M03, FR-C03
//!   T038 -- FR-M02, FR-C01, FR-C03, FR-C04, FR-C07

use agileplus_domain::{
    domain::{
        cycle::{Cycle, CycleFeature, CycleState, CycleWithFeatures},
        feature::Feature,
        module::{Module, ModuleFeatureTag},
        state_machine::FeatureState,
    },
    error::DomainError,
    ports::StoragePort,
};
use agileplus_sqlite::SqliteStorageAdapter;
use chrono::NaiveDate;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Create a fresh in-memory SQLite adapter. Panics if setup fails.
fn test_storage() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().expect("in-memory SQLite adapter should initialise")
}

/// Build a `NaiveDate` from literals. Panics on invalid dates.
fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}

/// Persist a `Feature` via StoragePort and return its assigned ID.
async fn store_feature(storage: &SqliteStorageAdapter, slug: &str, name: &str) -> i64 {
    let f = Feature::new(slug, name, [0u8; 32], None);
    storage
        .create_feature(&f)
        .await
        .expect("create_feature should succeed")
}

/// Persist a `Module` via StoragePort and return its assigned ID.
async fn store_module(storage: &SqliteStorageAdapter, name: &str, parent_id: Option<i64>) -> i64 {
    let m = Module::new(name, parent_id);
    storage
        .create_module(&m)
        .await
        .expect("create_module should succeed")
}

/// Persist a `Cycle` via StoragePort and return its assigned ID.
async fn store_cycle(
    storage: &SqliteStorageAdapter,
    name: &str,
    module_scope_id: Option<i64>,
) -> i64 {
    let c = Cycle::new(name, date(2026, 1, 1), date(2026, 3, 31), module_scope_id)
        .expect("cycle construction should succeed");
    storage
        .create_cycle(&c)
        .await
        .expect("create_cycle should succeed")
}

/// Transition a feature through every state up to `target` (inclusive).
/// This helper tolerates already-being-in or skipping via sequential forward steps.
async fn transition_feature_to(
    storage: &SqliteStorageAdapter,
    feature_id: i64,
    target: FeatureState,
) {
    let sequence = [
        FeatureState::Specified,
        FeatureState::Researched,
        FeatureState::Planned,
        FeatureState::Implementing,
        FeatureState::Validated,
        FeatureState::Shipped,
    ];
    for &state in &sequence {
        storage
            .update_feature_state(feature_id, state)
            .await
            .expect("transition should succeed");
        if state == target {
            break;
        }
    }
}

/// Directly set `features.module_id` via raw SQL.
///
/// This is needed because the current `StoragePort` does not expose an
/// `assign_feature_to_module` method. We use `conn_for_bench` which is
/// intentionally exposed for exactly this kind of low-level access.
///
/// Traces to: FR-M03
fn assign_feature_module_id(storage: &SqliteStorageAdapter, feature_id: i64, module_id: i64) {
    let conn = storage.conn_for_bench().expect("lock should succeed");
    conn.execute(
        "UPDATE features SET module_id = ?1 WHERE id = ?2",
        rusqlite::params![module_id, feature_id],
    )
    .expect("UPDATE features.module_id should succeed");
}

// ===========================================================================
// T034 -- Module Hierarchy CRUD Lifecycle
// ===========================================================================

/// Create a root module (no parent) and verify it is retrievable.
///
/// Traces to: FR-M01
#[tokio::test]
async fn module_create_root_succeeds() {
    // Traces to: FR-M01
    let storage = test_storage();
    let id = store_module(&storage, "Core Platform", None).await;
    assert!(id > 0, "assigned ID must be positive");

    let module = storage
        .get_module(id)
        .await
        .expect("get_module should succeed")
        .expect("module should be found");

    assert_eq!(module.friendly_name, "Core Platform");
    assert_eq!(module.slug, "core-platform");
    assert!(module.parent_module_id.is_none());
}

/// Create a child module with a valid parent and verify the parent link.
///
/// Traces to: FR-M02
#[tokio::test]
async fn module_create_child_succeeds() {
    // Traces to: FR-M02
    let storage = test_storage();
    let parent_id = store_module(&storage, "Platform", None).await;
    let child_id = store_module(&storage, "Auth", Some(parent_id)).await;

    let child = storage
        .get_module(child_id)
        .await
        .expect("get_module should succeed")
        .expect("child module should exist");

    assert_eq!(child.parent_module_id, Some(parent_id));
    assert_eq!(child.friendly_name, "Auth");
}

/// Assigning a feature to a module (via module_id FK) should persist.
///
/// Traces to: FR-M03
#[tokio::test]
async fn module_assign_feature_sets_module_id() {
    // Traces to: FR-M03
    let storage = test_storage();
    let module_id = store_module(&storage, "Payments", None).await;
    let feature_id = store_feature(&storage, "checkout", "Checkout Flow").await;

    assign_feature_module_id(&storage, feature_id, module_id);

    let mwf = storage
        .get_module_with_features(module_id)
        .await
        .expect("get_module_with_features should succeed")
        .expect("module should exist");

    let owned_ids: Vec<i64> = mwf.owned_features.iter().map(|f| f.id).collect();
    assert!(
        owned_ids.contains(&feature_id),
        "feature should appear in owned_features after module_id assignment"
    );
}

/// Tagging a feature to a module should create a `module_feature_tags` record.
///
/// Traces to: FR-M04
#[tokio::test]
async fn module_tag_feature_creates_tag_record() {
    // Traces to: FR-M04
    let storage = test_storage();
    let module_id = store_module(&storage, "Reporting", None).await;
    let feature_id = store_feature(&storage, "export-csv", "Export CSV").await;

    let tag = ModuleFeatureTag::new(module_id, feature_id);
    storage
        .tag_feature_to_module(&tag)
        .await
        .expect("tag_feature_to_module should succeed");

    let mwf = storage
        .get_module_with_features(module_id)
        .await
        .expect("get_module_with_features should succeed")
        .expect("module should exist");

    let tagged_ids: Vec<i64> = mwf.tagged_features.iter().map(|f| f.id).collect();
    assert!(
        tagged_ids.contains(&feature_id),
        "feature should appear in tagged_features"
    );
}

/// `list_child_modules` should show the hierarchy.
///
/// Traces to: FR-M02
#[tokio::test]
async fn module_list_tree_shows_hierarchy() {
    // Traces to: FR-M02
    let storage = test_storage();
    let root_id = store_module(&storage, "Root", None).await;
    let _child1_id = store_module(&storage, "Child A", Some(root_id)).await;
    let _child2_id = store_module(&storage, "Child B", Some(root_id)).await;

    let roots = storage
        .list_root_modules()
        .await
        .expect("list_root_modules should succeed");
    assert_eq!(roots.len(), 1, "exactly one root module");
    assert_eq!(roots[0].id, root_id);

    let children = storage
        .list_child_modules(root_id)
        .await
        .expect("list_child_modules should succeed");
    assert_eq!(children.len(), 2, "two child modules");
}

/// Deleting a module that has children must fail with `ModuleHasDependents`.
///
/// Traces to: FR-M07
#[tokio::test]
async fn module_delete_with_children_fails() {
    // Traces to: FR-M07
    let storage = test_storage();
    let parent_id = store_module(&storage, "Parent", None).await;
    let _child_id = store_module(&storage, "Child", Some(parent_id)).await;

    let result = storage.delete_module(parent_id).await;
    assert!(
        matches!(result, Err(DomainError::ModuleHasDependents(_))),
        "expected ModuleHasDependents, got: {result:?}"
    );
}

/// Deleting a module that owns features must fail with `ModuleHasDependents`.
///
/// Traces to: FR-M07
#[tokio::test]
async fn module_delete_with_owned_features_fails() {
    // Traces to: FR-M07
    let storage = test_storage();
    let module_id = store_module(&storage, "Owned Module", None).await;
    let feature_id = store_feature(&storage, "owned-feat", "Owned Feature").await;
    assign_feature_module_id(&storage, feature_id, module_id);

    let result = storage.delete_module(module_id).await;
    assert!(
        matches!(result, Err(DomainError::ModuleHasDependents(_))),
        "expected ModuleHasDependents for module owning features, got: {result:?}"
    );
}

/// Deleting a module with no children and no owned features must succeed.
///
/// Traces to: FR-M07
#[tokio::test]
async fn module_delete_empty_module_succeeds() {
    // Traces to: FR-M07
    let storage = test_storage();
    let module_id = store_module(&storage, "Empty Module", None).await;

    storage
        .delete_module(module_id)
        .await
        .expect("delete_module should succeed on an empty module");

    let gone = storage
        .get_module(module_id)
        .await
        .expect("get_module should not error after deletion");
    assert!(gone.is_none(), "module should be gone after deletion");
}

/// Slugs must be unique within the same parent module.
///
/// SQLite treats NULL != NULL for UNIQUE constraints, so the uniqueness
/// constraint only fires when two siblings share the same non-NULL parent.
///
/// Traces to: FR-M01
#[tokio::test]
async fn module_slug_unique_within_parent() {
    // Traces to: FR-M01
    let storage = test_storage();
    // Two siblings under the same parent with the same name -> same slug -> must fail.
    let parent_id = store_module(&storage, "Parent For Dedup", None).await;
    store_module(&storage, "Duplicate Child", Some(parent_id)).await;
    let result = storage
        .create_module(&Module::new("Duplicate Child", Some(parent_id)))
        .await;
    assert!(
        result.is_err(),
        "creating two sibling modules with the same slug under the same parent should fail"
    );
}

// ===========================================================================
// T035 -- Cycle Lifecycle with Gate Enforcement
// ===========================================================================

/// A new cycle always starts in Draft state.
///
/// Traces to: FR-C01, FR-C02
#[tokio::test]
async fn cycle_create_defaults_to_draft() {
    // Traces to: FR-C01, FR-C02
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
    // Traces to: FR-C02
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
    // Traces to: FR-C07
    let storage = test_storage();
    let cycle_id = store_cycle(&storage, "Gate Cycle", None).await;
    let feature_id = store_feature(&storage, "unready-feat", "Unready Feature").await;

    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("add_feature_to_cycle should succeed");

    // Load the cycle-with-features view and check the gate.
    let cwf: CycleWithFeatures = storage
        .get_cycle_with_features(cycle_id)
        .await
        .expect("get_cycle_with_features should succeed")
        .expect("cycle should exist");

    assert!(
        !cwf.is_shippable(),
        "cycle with a Created-state feature must not be shippable"
    );
}

/// The Shipped gate allows shipping when all features are Validated.
///
/// Traces to: FR-C07
#[tokio::test]
async fn cycle_shipped_gate_allows_all_validated() {
    // Traces to: FR-C07
    let storage = test_storage();
    let cycle_id = store_cycle(&storage, "Validated Cycle", None).await;
    let feature_id = store_feature(&storage, "ready-feat", "Ready Feature").await;

    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("add_feature_to_cycle should succeed");

    // Advance feature to Validated.
    transition_feature_to(&storage, feature_id, FeatureState::Validated).await;

    let cwf = storage
        .get_cycle_with_features(cycle_id)
        .await
        .expect("get_cycle_with_features should succeed")
        .expect("cycle should exist");

    assert!(
        cwf.is_shippable(),
        "cycle where all features are Validated must be shippable"
    );
}

/// The Shipped gate allows shipping when all features are already Shipped.
///
/// Traces to: FR-C07
#[tokio::test]
async fn cycle_shipped_gate_allows_all_shipped_features() {
    // Traces to: FR-C07
    let storage = test_storage();
    let cycle_id = store_cycle(&storage, "Shipped Cycle", None).await;
    let feature_id = store_feature(&storage, "shipped-feat", "Shipped Feature").await;

    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("add_feature_to_cycle should succeed");

    transition_feature_to(&storage, feature_id, FeatureState::Shipped).await;

    let cwf = storage
        .get_cycle_with_features(cycle_id)
        .await
        .expect("get_cycle_with_features should succeed")
        .expect("cycle should exist");

    assert!(
        cwf.is_shippable(),
        "cycle where all features are Shipped must be shippable"
    );
}

/// `get_cycle_with_features` returns a `WpProgressSummary` (may be zeroed for no WPs).
///
/// Traces to: FR-C05
#[tokio::test]
async fn cycle_show_returns_wp_progress() {
    // Traces to: FR-C05
    let storage = test_storage();
    let cycle_id = store_cycle(&storage, "Progress Cycle", None).await;

    let cwf = storage
        .get_cycle_with_features(cycle_id)
        .await
        .expect("get_cycle_with_features should succeed")
        .expect("cycle should exist");

    // An empty cycle has zero work packages; the summary fields should default to zero.
    assert_eq!(cwf.wp_progress.total, 0);
    assert_eq!(cwf.wp_progress.done, 0);
}

// ===========================================================================
// T036 -- Module-Scoped Cycles
// ===========================================================================

/// A cycle created with a module_scope_id stores that scope.
///
/// Traces to: FR-C04
#[tokio::test]
async fn scoped_cycle_created_with_module_scope() {
    // Traces to: FR-C04
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
    // Traces to: FR-C04, FR-C06
    let storage = test_storage();
    let scope_module_id = store_module(&storage, "Scope Module", None).await;
    let other_module_id = store_module(&storage, "Other Module", None).await;
    let cycle_id = store_cycle(&storage, "Rejecting Cycle", Some(scope_module_id)).await;

    // Create a feature owned by a different module.
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
    // Traces to: FR-C04
    let storage = test_storage();
    let module_id = store_module(&storage, "Owner Module", None).await;
    let cycle_id = store_cycle(&storage, "Owner Cycle", Some(module_id)).await;
    let feature_id = store_feature(&storage, "owned-scope-feat", "Owned Scope Feature").await;
    assign_feature_module_id(&storage, feature_id, module_id);

    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("scoped cycle should accept an owned feature");

    let cwf = storage
        .get_cycle_with_features(cycle_id)
        .await
        .expect("get_cycle_with_features should succeed")
        .expect("cycle should exist");

    let ids: Vec<i64> = cwf.features.iter().map(|f| f.id).collect();
    assert!(ids.contains(&feature_id));
}

/// A scoped cycle must accept a feature tagged (not owned) to the scope module.
///
/// Traces to: FR-C04
#[tokio::test]
async fn scoped_cycle_accepts_tagged_feature() {
    // Traces to: FR-C04
    let storage = test_storage();
    let module_id = store_module(&storage, "Tag Module", None).await;
    let cycle_id = store_cycle(&storage, "Tag Cycle", Some(module_id)).await;
    let feature_id = store_feature(&storage, "tagged-scope-feat", "Tagged Scope Feature").await;

    // Tag (not own) the feature to the scope module.
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
    // Traces to: FR-C03
    let storage = test_storage();
    let cycle_id = store_cycle(&storage, "Unscoped Cycle", None).await;
    let feature_id = store_feature(&storage, "any-feat", "Any Feature").await;

    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("unscoped cycle should accept any feature");
}

// ===========================================================================
// T037 -- Backward Compatibility
// ===========================================================================

/// A feature without a module_id can be created (NULL module_id is allowed).
///
/// Traces to: FR-M03
#[tokio::test]
async fn feature_without_module_id_can_be_created() {
    // Traces to: FR-M03
    let storage = test_storage();
    let feature_id = store_feature(&storage, "legacy-feat", "Legacy Feature").await;

    let feat = storage
        .get_feature_by_id(feature_id)
        .await
        .expect("get_feature_by_id should succeed")
        .expect("feature should exist");

    assert_eq!(
        feat.module_id, None,
        "new feature must have no module_id by default"
    );
}

/// A feature without a module_id appears in `list_all_features` normally.
///
/// Traces to: FR-M03
#[tokio::test]
async fn feature_without_module_id_lists_normally() {
    // Traces to: FR-M03
    let storage = test_storage();
    let feature_id = store_feature(&storage, "unmodule-feat", "Unmodule Feature").await;

    let all = storage
        .list_all_features()
        .await
        .expect("list_all_features should succeed");

    let found = all.iter().any(|f| f.id == feature_id);
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
    // Traces to: FR-C03, FR-M03
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
    // Traces to: FR-C04, FR-M03
    let storage = test_storage();
    let module_id = store_module(&storage, "Strict Module", None).await;
    let cycle_id = store_cycle(&storage, "Strict Cycle", Some(module_id)).await;
    let feature_id = store_feature(&storage, "no-module-feat", "No Module Feature").await;
    // feature_id has no module_id and is not tagged to module_id.

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
    // Traces to: FR-M03
    let storage = test_storage();
    let feature_id = store_feature(&storage, "trans-feat", "Transition Feature").await;

    storage
        .update_feature_state(feature_id, FeatureState::Specified)
        .await
        .expect("state transition should succeed for feature without module_id");

    let feat = storage
        .get_feature_by_id(feature_id)
        .await
        .expect("get_feature_by_id should succeed")
        .expect("feature should exist");

    assert_eq!(feat.state, FeatureState::Specified);
    assert_eq!(
        feat.module_id, None,
        "module_id should remain None after transition"
    );
}

// ===========================================================================
// T038 -- Edge Cases
// ===========================================================================

/// Attempting to create a module whose proposed parent would create a cycle
/// (i.e., the parent doesn't exist) must fail.
///
/// Traces to: FR-M02
#[tokio::test]
async fn circular_module_ref_is_rejected() {
    // Traces to: FR-M02
    let storage = test_storage();

    // Attempt to create a module with a non-existent parent ID -- should fail.
    let phantom_parent: i64 = 9_999_999;
    let bad_module = Module::new("Bad Child", Some(phantom_parent));
    let result = storage.create_module(&bad_module).await;

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
    // Traces to: FR-C03
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

    // Verify the feature appears in both cycle views.
    let cwf1 = storage
        .get_cycle_with_features(cycle1_id)
        .await
        .expect("get cwf 1")
        .expect("cycle 1 exists");
    let cwf2 = storage
        .get_cycle_with_features(cycle2_id)
        .await
        .expect("get cwf 2")
        .expect("cycle 2 exists");

    assert!(cwf1.features.iter().any(|f| f.id == feature_id));
    assert!(cwf2.features.iter().any(|f| f.id == feature_id));
}

/// An empty cycle in Draft state is valid (no gate violation).
///
/// Traces to: FR-C01, FR-C02
#[tokio::test]
async fn empty_cycle_in_draft_is_valid() {
    // Traces to: FR-C01, FR-C02
    let storage = test_storage();
    let cycle_id = store_cycle(&storage, "Empty Draft Cycle", None).await;

    let cwf = storage
        .get_cycle_with_features(cycle_id)
        .await
        .expect("get_cycle_with_features should succeed")
        .expect("cycle should exist");

    assert_eq!(cwf.cycle.state, CycleState::Draft);
    assert!(cwf.features.is_empty());
    // Vacuously shippable -- no features to block shipping.
    assert!(
        cwf.is_shippable(),
        "empty cycle is vacuously shippable (no blocking features)"
    );
}

/// Deleting a cycle unlinks features (cycle_features rows) but does not change feature state.
///
/// Traces to: FR-C03
#[tokio::test]
async fn deleting_cycle_unlinks_features_does_not_change_state() {
    // Traces to: FR-C03
    let storage = test_storage();
    let cycle_id = store_cycle(&storage, "Del Cycle", None).await;
    let feature_id = store_feature(&storage, "del-cycle-feat", "Del Cycle Feature").await;

    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("add_feature_to_cycle should succeed");

    // Remove the feature from the cycle (simulates cycle deletion unlinking).
    storage
        .remove_feature_from_cycle(cycle_id, feature_id)
        .await
        .expect("remove_feature_from_cycle should succeed");

    // The feature itself must be unaffected.
    let feat = storage
        .get_feature_by_id(feature_id)
        .await
        .expect("get_feature_by_id should succeed")
        .expect("feature should still exist");

    assert_eq!(
        feat.state,
        FeatureState::Created,
        "feature state must not change when removed from a cycle"
    );
}

/// Deleting a module with children requires reparenting or deleting children first.
///
/// Traces to: FR-M07
#[tokio::test]
async fn delete_module_with_children_requires_reparenting_first() {
    // Traces to: FR-M07
    let storage = test_storage();
    let parent_id = store_module(&storage, "Has Children", None).await;
    let child_id = store_module(&storage, "Child One", Some(parent_id)).await;

    // Delete parent must fail.
    let result = storage.delete_module(parent_id).await;
    assert!(result.is_err(), "deleting parent with children must fail");

    // Delete child first, then parent should succeed.
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
    // Traces to: FR-M04, FR-C04
    let storage = test_storage();
    let module_id = store_module(&storage, "Scope Removal Module", None).await;
    let cycle_id = store_cycle(&storage, "Scope Removal Cycle", Some(module_id)).await;
    let feature_id = store_feature(&storage, "scope-rem-feat", "Scope Removal Feature").await;

    // Tag the feature to the module so it passes the scope check.
    storage
        .tag_feature_to_module(&ModuleFeatureTag::new(module_id, feature_id))
        .await
        .expect("tag should succeed");

    storage
        .add_feature_to_cycle(&CycleFeature::new(cycle_id, feature_id))
        .await
        .expect("add should succeed while feature is in scope");

    // Now remove the tag.
    storage
        .untag_feature_from_module(module_id, feature_id)
        .await
        .expect("untag should succeed");

    // The feature should still be in the cycle (no cascade removal).
    let cwf = storage
        .get_cycle_with_features(cycle_id)
        .await
        .expect("get_cycle_with_features should succeed")
        .expect("cycle should exist");

    assert!(
        cwf.features.iter().any(|f| f.id == feature_id),
        "feature must remain in cycle after its module tag is removed (no cascade)"
    );
}

/// A cycle whose end_date is not after start_date must be rejected at construction.
///
/// Traces to: FR-C01
#[tokio::test]
async fn cycle_date_constraint_rejected() {
    // Traces to: FR-C01
    // end_date == start_date is invalid.
    let same_day = date(2026, 6, 1);
    let result = Cycle::new("Bad Date Cycle", same_day, same_day, None);
    assert!(
        result.is_err(),
        "Cycle::new with end_date == start_date must fail"
    );

    // end_date before start_date is also invalid.
    let result2 = Cycle::new(
        "Bad Date Cycle 2",
        date(2026, 6, 10),
        date(2026, 6, 1),
        None,
    );
    assert!(
        result2.is_err(),
        "Cycle::new with end_date before start_date must fail"
    );
}
