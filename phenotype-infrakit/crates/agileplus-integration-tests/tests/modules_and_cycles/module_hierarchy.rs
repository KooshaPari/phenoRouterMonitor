use super::helpers::{assign_feature_module_id, store_feature, store_module, test_storage};
use agileplus_domain::{
    domain::module::{Module, ModuleFeatureTag},
    error::DomainError,
    ports::StoragePort,
};

/// Create a root module (no parent) and verify it is retrievable.
///
/// Traces to: FR-M01
#[tokio::test]
async fn module_create_root_succeeds() {
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
    let storage = test_storage();
    let module_id = store_module(&storage, "Payments", None).await;
    let feature_id = store_feature(&storage, "checkout", "Checkout Flow").await;

    assign_feature_module_id(&storage, feature_id, module_id);

    let module = storage
        .get_module_with_features(module_id)
        .await
        .expect("get_module_with_features should succeed")
        .expect("module should exist");

    let owned_ids: Vec<i64> = module
        .owned_features
        .iter()
        .map(|feature| feature.id)
        .collect();
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
    let storage = test_storage();
    let module_id = store_module(&storage, "Reporting", None).await;
    let feature_id = store_feature(&storage, "export-csv", "Export CSV").await;

    let tag = ModuleFeatureTag::new(module_id, feature_id);
    storage
        .tag_feature_to_module(&tag)
        .await
        .expect("tag_feature_to_module should succeed");

    let module = storage
        .get_module_with_features(module_id)
        .await
        .expect("get_module_with_features should succeed")
        .expect("module should exist");

    let tagged_ids: Vec<i64> = module
        .tagged_features
        .iter()
        .map(|feature| feature.id)
        .collect();
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
    let storage = test_storage();
    let root_id = store_module(&storage, "Root", None).await;
    let _ = store_module(&storage, "Child A", Some(root_id)).await;
    let _ = store_module(&storage, "Child B", Some(root_id)).await;

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
    let storage = test_storage();
    let parent_id = store_module(&storage, "Parent", None).await;
    let _ = store_module(&storage, "Child", Some(parent_id)).await;

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
    let storage = test_storage();
    let module_id = store_module(&storage, "Empty Module", None).await;

    storage
        .delete_module(module_id)
        .await
        .expect("delete_module should succeed on an empty module");

    let module = storage
        .get_module(module_id)
        .await
        .expect("get_module should not error after deletion");
    assert!(module.is_none(), "module should be gone after deletion");
}

/// Slugs must be unique within the same parent module.
///
/// Traces to: FR-M01
#[tokio::test]
async fn module_slug_unique_within_parent() {
    let storage = test_storage();
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
