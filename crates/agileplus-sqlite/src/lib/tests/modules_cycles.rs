use super::*;
use agileplus_domain::domain::cycle::{Cycle, CycleFeature, CycleState};
use agileplus_domain::domain::module::{Module, ModuleFeatureTag};

#[tokio::test]
async fn module_create_and_get() {
    let db = make_adapter();
    let m = Module::new("Auth Module", None);
    let id = StoragePort::create_module(&db, &m).await.unwrap();
    assert!(id > 0);
 
    let got = StoragePort::get_module(&db, id).await.unwrap().unwrap();
    assert_eq!(got.id, id);
    assert_eq!(got.slug, "auth-module");
    assert_eq!(got.friendly_name, "Auth Module");
    assert!(got.parent_module_id.is_none());
}
 
#[tokio::test]
async fn module_get_by_slug() {
    let db = make_adapter();
    let m = Module::new("Billing", None);
    let id = StoragePort::create_module(&db, &m).await.unwrap();
    let got = StoragePort::get_module_by_slug(&db, "billing").await.unwrap().unwrap();
    assert_eq!(got.id, id);
}
 
#[tokio::test]
async fn module_not_found_returns_none() {
    let db = make_adapter();
    assert!(StoragePort::get_module(&db, 9999).await.unwrap().is_none());
    assert!(StoragePort::get_module_by_slug(&db, "no-such").await.unwrap().is_none());
}
 
#[tokio::test]
async fn module_update() {
    let db = make_adapter();
    let m = Module::new("Old Name", None);
    let id = StoragePort::create_module(&db, &m).await.unwrap();
    StoragePort::update_module(&db, id, "New Name", Some("a description"))
        .await
        .unwrap();
    let got = StoragePort::get_module(&db, id).await.unwrap().unwrap();
    assert_eq!(got.friendly_name, "New Name");
    assert_eq!(got.slug, "new-name");
    assert_eq!(got.description.as_deref(), Some("a description"));
}
 
#[tokio::test]
async fn module_delete_simple() {
    let db = make_adapter();
    let m = Module::new("Temp", None);
    let id = StoragePort::create_module(&db, &m).await.unwrap();
    StoragePort::delete_module(&db, id).await.unwrap();
    assert!(StoragePort::get_module(&db, id).await.unwrap().is_none());
}
 
#[tokio::test]
async fn module_delete_with_children_fails() {
    let db = make_adapter();
    let parent = Module::new("Parent", None);
    let pid = StoragePort::create_module(&db, &parent).await.unwrap();
    let child = Module::new("Child", Some(pid));
    StoragePort::create_module(&db, &child).await.unwrap();
 
    let result = StoragePort::delete_module(&db, pid).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        agileplus_domain::error::DomainError::ModuleHasDependents(_)
    ));
}
 
#[tokio::test]
async fn module_delete_with_owned_features_fails() {
    let db = make_adapter();
    let m = Module::new("Owner", None);
    let mid = StoragePort::create_module(&db, &m).await.unwrap();
    let f = Feature::new("feat", "Feat", [0u8; 32], None);
    let fid = StoragePort::create_feature(&db, &f).await.unwrap();
    let conn = db.conn_for_bench().unwrap();
    conn.execute(
        "UPDATE features SET module_id = ?1 WHERE id = ?2",
        rusqlite::params![mid, fid],
    )
    .unwrap();
    drop(conn);
 
    let result = StoragePort::delete_module(&db, mid).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        agileplus_domain::error::DomainError::ModuleHasDependents(_)
    ));
}
 
#[tokio::test]
async fn module_list_root_and_children() {
    let db = make_adapter();
    let r1 = StoragePort::create_module(&db, &Module::new("Root1", None)).await.unwrap();
    let r2 = StoragePort::create_module(&db, &Module::new("Root2", None)).await.unwrap();
    let _ = StoragePort::create_module(&db, &Module::new("Child1", Some(r1)))
        .await
        .unwrap();
 
    let roots = StoragePort::list_root_modules(&db).await.unwrap();
    assert_eq!(roots.len(), 2);
 
    let children = StoragePort::list_child_modules(&db, r1).await.unwrap();
    assert_eq!(children.len(), 1);
 
    let r2_children = StoragePort::list_child_modules(&db, r2).await.unwrap();
    assert!(r2_children.is_empty());
}
 
#[tokio::test]
async fn module_tag_and_untag_feature() {
    let db = make_adapter();
    let mid = StoragePort::create_module(&db, &Module::new("M", None)).await.unwrap();
    let fid = StoragePort::create_feature(&db, &Feature::new("f-tag", "FTag", [0u8; 32], None))
        .await
        .unwrap();
 
    let tag = ModuleFeatureTag::new(mid, fid);
    StoragePort::tag_feature_to_module(&db, &tag).await.unwrap();
    StoragePort::tag_feature_to_module(&db, &tag).await.unwrap();
 
    let mwf = StoragePort::get_module_with_features(&db, mid).await.unwrap().unwrap();
    assert_eq!(mwf.tagged_features.len(), 1);
    assert_eq!(mwf.tagged_features[0].id, fid);
 
    StoragePort::untag_feature_from_module(&db, mid, fid).await.unwrap();
    let mwf2 = StoragePort::get_module_with_features(&db, mid).await.unwrap().unwrap();
    assert!(mwf2.tagged_features.is_empty());
}
 
#[tokio::test]
async fn module_get_with_features_none_for_missing() {
    let db = make_adapter();
    assert!(StoragePort::get_module_with_features(&db, 9999).await.unwrap().is_none());
}
 
#[tokio::test]
async fn cycle_create_and_get() {
    let db = make_adapter();
    let c = Cycle::new(
        "Q1-2026",
        make_date(2026, 1, 1),
        make_date(2026, 3, 31),
        None,
    )
    .unwrap();
    let id = StoragePort::create_cycle(&db, &c).await.unwrap();
    assert!(id > 0);
 
    let got = StoragePort::get_cycle(&db, id).await.unwrap().unwrap();
    assert_eq!(got.id, id);
    assert_eq!(got.name, "Q1-2026");
    assert_eq!(got.state, CycleState::Draft);
    assert!(got.module_scope_id.is_none());
}
 
#[tokio::test]
async fn cycle_not_found_returns_none() {
    let db = make_adapter();
    assert!(StoragePort::get_cycle(&db, 9999).await.unwrap().is_none());
}
 
#[tokio::test]
async fn cycle_update_state() {
    let db = make_adapter();
    let c = Cycle::new(
        "Cycle-A",
        make_date(2026, 1, 1),
        make_date(2026, 2, 1),
        None,
    )
    .unwrap();
    let id = StoragePort::create_cycle(&db, &c).await.unwrap();
    StoragePort::update_cycle_state(&db, id, CycleState::Active).await.unwrap();
    let got = StoragePort::get_cycle(&db, id).await.unwrap().unwrap();
    assert_eq!(got.state, CycleState::Active);
}
 
#[tokio::test]
async fn cycle_list_by_state() {
    let db = make_adapter();
    let c1 = Cycle::new(
        "Draft-1",
        make_date(2026, 1, 1),
        make_date(2026, 2, 1),
        None,
    )
    .unwrap();
    let c2 = Cycle::new(
        "Draft-2",
        make_date(2026, 3, 1),
        make_date(2026, 4, 1),
        None,
    )
    .unwrap();
    let id1 = StoragePort::create_cycle(&db, &c1).await.unwrap();
    let id2 = StoragePort::create_cycle(&db, &c2).await.unwrap();
    StoragePort::update_cycle_state(&db, id1, CycleState::Active)
        .await
        .unwrap();
 
    let drafts = StoragePort::list_cycles_by_state(&db, CycleState::Draft).await.unwrap();
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].id, id2);
 
    let actives = StoragePort::list_cycles_by_state(&db, CycleState::Active).await.unwrap();
    assert_eq!(actives.len(), 1);
    assert_eq!(actives[0].id, id1);
}
 
#[tokio::test]
async fn cycle_list_by_module() {
    let db = make_adapter();
    let mid = StoragePort::create_module(&db, &Module::new("ScopeModule", None))
        .await
        .unwrap();
    let c1 = Cycle::new(
        "Scoped",
        make_date(2026, 1, 1),
        make_date(2026, 2, 1),
        Some(mid),
    )
    .unwrap();
    let c2 = Cycle::new(
        "Unscoped",
        make_date(2026, 3, 1),
        make_date(2026, 4, 1),
        None,
    )
    .unwrap();
    let id1 = StoragePort::create_cycle(&db, &c1).await.unwrap();
    StoragePort::create_cycle(&db, &c2).await.unwrap();
 
    let scoped = StoragePort::list_cycles_by_module(&db, mid).await.unwrap();
    assert_eq!(scoped.len(), 1);
    assert_eq!(scoped[0].id, id1);
}
 
#[tokio::test]
async fn cycle_add_and_remove_feature() {
    let db = make_adapter();
    let c = Cycle::new("C1", make_date(2026, 1, 1), make_date(2026, 2, 1), None).unwrap();
    let cid = StoragePort::create_cycle(&db, &c).await.unwrap();
    let fid = StoragePort::create_feature(&db, &Feature::new("cyc-feat", "CycFeat", [0u8; 32], None))
        .await
        .unwrap();
 
    let entry = CycleFeature::new(cid, fid);
    StoragePort::add_feature_to_cycle(&db, &entry).await.unwrap();
    StoragePort::add_feature_to_cycle(&db, &entry).await.unwrap();
 
    let cwf = StoragePort::get_cycle_with_features(&db, cid).await.unwrap().unwrap();
    assert_eq!(cwf.features.len(), 1);
    assert_eq!(cwf.features[0].id, fid);
 
    StoragePort::remove_feature_from_cycle(&db, cid, fid).await.unwrap();
    let cwf2 = StoragePort::get_cycle_with_features(&db, cid).await.unwrap().unwrap();
    assert!(cwf2.features.is_empty());
}
 
#[tokio::test]
async fn cycle_with_features_none_for_missing() {
    let db = make_adapter();
    assert!(StoragePort::get_cycle_with_features(&db, 9999).await.unwrap().is_none());
}
 
#[tokio::test]
async fn cycle_module_scope_enforcement() {
    let db = make_adapter();
    let mid = StoragePort::create_module(&db, &Module::new("Scope", None)).await.unwrap();
    let c = Cycle::new(
        "Scoped-Cycle",
        make_date(2026, 1, 1),
        make_date(2026, 2, 1),
        Some(mid),
    )
    .unwrap();
    let cid = StoragePort::create_cycle(&db, &c).await.unwrap();
 
    let fid = StoragePort::create_feature(&db, &Feature::new("out-of-scope", "OOS", [0u8; 32], None))
        .await
        .unwrap();
    let entry = CycleFeature::new(cid, fid);
    let result = StoragePort::add_feature_to_cycle(&db, &entry).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        agileplus_domain::error::DomainError::FeatureNotInModuleScope { .. }
    ));
 
    StoragePort::tag_feature_to_module(&db, &ModuleFeatureTag::new(mid, fid))
        .await
        .unwrap();
    StoragePort::add_feature_to_cycle(&db, &CycleFeature::new(cid, fid))
        .await
        .unwrap();
    let cwf = StoragePort::get_cycle_with_features(&db, cid).await.unwrap().unwrap();
    assert_eq!(cwf.features.len(), 1);
}
 
#[tokio::test]
async fn cycle_wp_progress_summary() {
    let db = make_adapter();
    let c = Cycle::new(
        "WP-Prog",
        make_date(2026, 1, 1),
        make_date(2026, 2, 1),
        None,
    )
    .unwrap();
    let cid = StoragePort::create_cycle(&db, &c).await.unwrap();
    let fid = StoragePort::create_feature(&db, &Feature::new("prog-feat", "Prog", [0u8; 32], None))
        .await
        .unwrap();
    StoragePort::add_feature_to_cycle(&db, &CycleFeature::new(cid, fid))
        .await
        .unwrap();
 
    let _wp1 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP1", 1, "c"))
        .await
        .unwrap();
    let wp2 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP2", 2, "c"))
        .await
        .unwrap();
    StoragePort::update_wp_state(&db, wp2, WpState::Done).await.unwrap();
 
    let cwf = StoragePort::get_cycle_with_features(&db, cid).await.unwrap().unwrap();
    assert_eq!(cwf.wp_progress.total, 2);
    assert_eq!(cwf.wp_progress.planned, 1);
    assert_eq!(cwf.wp_progress.done, 1);
}
