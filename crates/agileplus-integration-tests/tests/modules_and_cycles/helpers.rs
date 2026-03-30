use agileplus_domain::{
    domain::{cycle::Cycle, feature::Feature, module::Module, state_machine::FeatureState},
    ports::StoragePort,
};
use agileplus_sqlite::SqliteStorageAdapter;
use chrono::NaiveDate;

/// Create a fresh in-memory SQLite adapter. Panics if setup fails.
pub(super) fn test_storage() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().expect("in-memory SQLite adapter should initialise")
}

/// Build a `NaiveDate` from literals. Panics on invalid dates.
pub(super) fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}

/// Persist a `Feature` via StoragePort and return its assigned ID.
pub(super) async fn store_feature(storage: &SqliteStorageAdapter, slug: &str, name: &str) -> i64 {
    let feature = Feature::new(slug, name, [0u8; 32], None);
    storage
        .create_feature(&feature)
        .await
        .expect("create_feature should succeed")
}

/// Persist a `Module` via StoragePort and return its assigned ID.
pub(super) async fn store_module(
    storage: &SqliteStorageAdapter,
    name: &str,
    parent_id: Option<i64>,
) -> i64 {
    let module = Module::new(name, parent_id);
    storage
        .create_module(&module)
        .await
        .expect("create_module should succeed")
}

/// Persist a `Cycle` via StoragePort and return its assigned ID.
pub(super) async fn store_cycle(
    storage: &SqliteStorageAdapter,
    name: &str,
    module_scope_id: Option<i64>,
) -> i64 {
    let cycle = Cycle::new(name, date(2026, 1, 1), date(2026, 3, 31), module_scope_id)
        .expect("cycle construction should succeed");
    storage
        .create_cycle(&cycle)
        .await
        .expect("create_cycle should succeed")
}

/// Transition a feature through every state up to `target` (inclusive).
pub(super) async fn transition_feature_to(
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
pub(super) fn assign_feature_module_id(
    storage: &SqliteStorageAdapter,
    feature_id: i64,
    module_id: i64,
) {
    let conn = storage.conn_for_bench().expect("lock should succeed");
    conn.execute(
        "UPDATE features SET module_id = ?1 WHERE id = ?2",
        rusqlite::params![module_id, feature_id],
    )
    .expect("UPDATE features.module_id should succeed");
}
