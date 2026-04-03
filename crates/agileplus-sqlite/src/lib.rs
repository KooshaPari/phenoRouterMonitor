//! AgilePlus SQLite adapter — persistence layer.
//!
//! Implements `StoragePort` using rusqlite with WAL mode and foreign keys.
//! Traceability: WP06

pub mod migrations;
pub mod rebuild;
pub mod repository;

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use agileplus_domain::{
    domain::{
        audit::AuditEntry,
        backlog::{BacklogFilters, BacklogItem, BacklogPriority, BacklogStatus},
        cycle::{Cycle, CycleFeature, CycleState, CycleWithFeatures},
        feature::Feature,
        governance::{Evidence, GovernanceContract, PolicyRule},
        metric::Metric,
        module::{Module, ModuleFeatureTag, ModuleWithFeatures},
        state_machine::FeatureState,
        work_package::{WorkPackage, WpDependency, WpState},
    },
    error::DomainError,
    ports::{ContentStoragePort, StoragePort},
};

use agileplus_domain::domain::event::Event;
use agileplus_events::{EventError, EventStore};

use crate::migrations::MigrationRunner;
use agileplus_domain::domain::project::Project;
use agileplus_domain::domain::sync_mapping::SyncMapping;

use crate::repository::{
    audit, backlog, cycles, events, evidence, features, governance, metrics, modules, projects,
    sync_mappings, work_packages,
};

/// SQLite-backed storage adapter.
///
/// Uses a single write-serialized connection protected by a Mutex.
/// WAL mode is enabled to allow concurrent reads; all writes are serialized.
pub struct SqliteStorageAdapter {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStorageAdapter {
    /// Open a file-backed database, enable WAL + FK pragma, and run all migrations.
    pub fn new(db_path: &Path) -> Result<Self, DomainError> {
        let conn = Connection::open(db_path)
            .map_err(|e| DomainError::Storage(format!("failed to open db: {e}")))?;
        Self::configure_and_migrate(conn)
    }

    /// Open an in-memory database (for tests).
    pub fn in_memory() -> Result<Self, DomainError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| DomainError::Storage(format!("failed to open in-memory db: {e}")))?;
        Self::configure_and_migrate(conn)
    }

    fn configure_and_migrate(conn: Connection) -> Result<Self, DomainError> {
        // Enable WAL mode for concurrent reads
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| DomainError::Storage(format!("WAL pragma failed: {e}")))?;

        // Enable foreign key enforcement
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|e| DomainError::Storage(format!("FK pragma failed: {e}")))?;

        // Run migrations
        let runner = MigrationRunner::new(&conn);
        runner.run_all()?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Get a locked guard to the connection.
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, DomainError> {
        self.conn
            .lock()
            .map_err(|e| DomainError::Storage(format!("mutex poisoned: {e}")))
    }

    /// Expose a locked connection guard for benchmarks and test helpers.
    ///
    /// This method is intentionally public so that benchmark crates can access
    /// the underlying rusqlite `Connection` to call repository functions directly
    /// without going through the async `StoragePort` trait.
    pub fn conn_for_bench(&self) -> Result<std::sync::MutexGuard<'_, Connection>, DomainError> {
        self.lock()
    }
}

impl StoragePort for SqliteStorageAdapter {
    // -- Feature CRUD --

    async fn create_feature(&self, feature: &Feature) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        features::create_feature(&conn, feature)
    }

    async fn get_feature_by_slug(&self, slug: &str) -> Result<Option<Feature>, DomainError> {
        let conn = self.lock()?;
        features::get_feature_by_slug(&conn, slug)
    }

    async fn get_feature_by_id(&self, id: i64) -> Result<Option<Feature>, DomainError> {
        let conn = self.lock()?;
        features::get_feature_by_id(&conn, id)
    }

    async fn update_feature_state(&self, id: i64, state: FeatureState) -> Result<(), DomainError> {
        let conn = self.lock()?;
        features::update_feature_state(&conn, id, state)
    }

    async fn list_features_by_state(
        &self,
        state: FeatureState,
    ) -> Result<Vec<Feature>, DomainError> {
        let conn = self.lock()?;
        features::list_features_by_state(&conn, state)
    }

    async fn list_all_features(&self) -> Result<Vec<Feature>, DomainError> {
        let conn = self.lock()?;
        features::list_all_features(&conn)
    }

    // -- Work Package CRUD --

    async fn create_work_package(&self, wp: &WorkPackage) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        work_packages::create_work_package(&conn, wp)
    }

    async fn get_work_package(&self, id: i64) -> Result<Option<WorkPackage>, DomainError> {
        let conn = self.lock()?;
        work_packages::get_work_package(&conn, id)
    }

    async fn update_wp_state(&self, id: i64, state: WpState) -> Result<(), DomainError> {
        let conn = self.lock()?;
        work_packages::update_wp_state(&conn, id, state)
    }

    async fn list_wps_by_feature(&self, feature_id: i64) -> Result<Vec<WorkPackage>, DomainError> {
        let conn = self.lock()?;
        work_packages::list_wps_by_feature(&conn, feature_id)
    }

    async fn add_wp_dependency(&self, dep: &WpDependency) -> Result<(), DomainError> {
        let conn = self.lock()?;
        work_packages::add_wp_dependency(&conn, dep)
    }

    async fn get_wp_dependencies(&self, wp_id: i64) -> Result<Vec<WpDependency>, DomainError> {
        let conn = self.lock()?;
        work_packages::get_wp_dependencies(&conn, wp_id)
    }

    async fn get_ready_wps(&self, feature_id: i64) -> Result<Vec<WorkPackage>, DomainError> {
        let conn = self.lock()?;
        work_packages::get_ready_wps(&conn, feature_id)
    }

    // -- Audit CRUD --

    async fn append_audit_entry(&self, entry: &AuditEntry) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        audit::append_audit_entry(&conn, entry)
    }

    async fn get_audit_trail(&self, feature_id: i64) -> Result<Vec<AuditEntry>, DomainError> {
        let conn = self.lock()?;
        audit::get_audit_trail(&conn, feature_id)
    }

    async fn get_latest_audit_entry(
        &self,
        feature_id: i64,
    ) -> Result<Option<AuditEntry>, DomainError> {
        let conn = self.lock()?;
        audit::get_latest_audit_entry(&conn, feature_id)
    }

    // -- Evidence + Policy + Metric CRUD --

    async fn create_evidence(&self, ev: &Evidence) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        evidence::create_evidence(&conn, ev)
    }

    async fn get_evidence_by_wp(&self, wp_id: i64) -> Result<Vec<Evidence>, DomainError> {
        let conn = self.lock()?;
        evidence::get_evidence_by_wp(&conn, wp_id)
    }

    async fn get_evidence_by_fr(&self, fr_id: &str) -> Result<Vec<Evidence>, DomainError> {
        let conn = self.lock()?;
        evidence::get_evidence_by_fr(&conn, fr_id)
    }

    async fn create_policy_rule(&self, rule: &PolicyRule) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        governance::create_policy_rule(&conn, rule)
    }

    async fn list_active_policies(&self) -> Result<Vec<PolicyRule>, DomainError> {
        let conn = self.lock()?;
        governance::list_active_policies(&conn)
    }

    async fn record_metric(&self, metric: &Metric) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        metrics::record_metric(&conn, metric)
    }

    async fn get_metrics_by_feature(&self, feature_id: i64) -> Result<Vec<Metric>, DomainError> {
        let conn = self.lock()?;
        metrics::get_metrics_by_feature(&conn, feature_id)
    }

    // -- Governance --

    async fn create_governance_contract(
        &self,
        contract: &GovernanceContract,
    ) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        governance::create_governance_contract(&conn, contract)
    }

    async fn get_governance_contract(
        &self,
        feature_id: i64,
        version: i32,
    ) -> Result<Option<GovernanceContract>, DomainError> {
        let conn = self.lock()?;
        governance::get_governance_contract(&conn, feature_id, version)
    }

    async fn get_latest_governance_contract(
        &self,
        feature_id: i64,
    ) -> Result<Option<GovernanceContract>, DomainError> {
        let conn = self.lock()?;
        governance::get_latest_governance_contract(&conn, feature_id)
    }

    // -- Module CRUD (T007) --

    async fn create_module(&self, module: &Module) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        modules::create_module(&conn, module)
    }

    async fn get_module(&self, id: i64) -> Result<Option<Module>, DomainError> {
        let conn = self.lock()?;
        modules::get_module(&conn, id)
    }

    async fn get_module_by_slug(&self, slug: &str) -> Result<Option<Module>, DomainError> {
        let conn = self.lock()?;
        modules::get_module_by_slug(&conn, slug)
    }

    async fn update_module(
        &self,
        id: i64,
        friendly_name: &str,
        description: Option<&str>,
    ) -> Result<(), DomainError> {
        let conn = self.lock()?;
        modules::update_module(&conn, id, friendly_name, description)
    }

    async fn delete_module(&self, id: i64) -> Result<(), DomainError> {
        let conn = self.lock()?;
        modules::delete_module(&conn, id)
    }

    async fn list_root_modules(&self) -> Result<Vec<Module>, DomainError> {
        let conn = self.lock()?;
        modules::list_root_modules(&conn)
    }

    async fn list_child_modules(&self, parent_id: i64) -> Result<Vec<Module>, DomainError> {
        let conn = self.lock()?;
        modules::list_child_modules(&conn, parent_id)
    }

    async fn get_module_with_features(
        &self,
        id: i64,
    ) -> Result<Option<ModuleWithFeatures>, DomainError> {
        let conn = self.lock()?;
        modules::get_module_with_features(&conn, id)
    }

    // -- Cycle CRUD (T008) --

    async fn create_cycle(&self, cycle: &Cycle) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        cycles::create_cycle(&conn, cycle)
    }

    async fn get_cycle(&self, id: i64) -> Result<Option<Cycle>, DomainError> {
        let conn = self.lock()?;
        cycles::get_cycle(&conn, id)
    }

    async fn update_cycle_state(&self, id: i64, state: CycleState) -> Result<(), DomainError> {
        let conn = self.lock()?;
        cycles::update_cycle_state(&conn, id, state)
    }

    async fn list_cycles_by_state(&self, state: CycleState) -> Result<Vec<Cycle>, DomainError> {
        let conn = self.lock()?;
        cycles::list_cycles_by_state(&conn, state)
    }

    async fn list_cycles_by_module(&self, module_id: i64) -> Result<Vec<Cycle>, DomainError> {
        let conn = self.lock()?;
        cycles::list_cycles_by_module(&conn, module_id)
    }

    async fn list_all_cycles(&self) -> Result<Vec<Cycle>, DomainError> {
        let conn = self.lock()?;
        cycles::list_all_cycles(&conn)
    }

    async fn get_cycle_with_features(
        &self,
        id: i64,
    ) -> Result<Option<CycleWithFeatures>, DomainError> {
        let conn = self.lock()?;
        cycles::get_cycle_with_features(&conn, id)
    }

    // -- Join table ops (T009) --

    async fn tag_feature_to_module(&self, tag: &ModuleFeatureTag) -> Result<(), DomainError> {
        let conn = self.lock()?;
        modules::tag_feature_to_module(&conn, tag)
    }

    async fn untag_feature_from_module(
        &self,
        module_id: i64,
        feature_id: i64,
    ) -> Result<(), DomainError> {
        let conn = self.lock()?;
        modules::untag_feature_from_module(&conn, module_id, feature_id)
    }

    async fn add_feature_to_cycle(&self, entry: &CycleFeature) -> Result<(), DomainError> {
        let conn = self.lock()?;
        cycles::add_feature_to_cycle(&conn, entry)
    }

    async fn remove_feature_from_cycle(
        &self,
        cycle_id: i64,
        feature_id: i64,
    ) -> Result<(), DomainError> {
        let conn = self.lock()?;
        cycles::remove_feature_from_cycle(&conn, cycle_id, feature_id)
    }

    // -- Sync Mapping CRUD (WP06-T033) --

    async fn get_sync_mapping(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<Option<SyncMapping>, DomainError> {
        let conn = self.lock()?;
        sync_mappings::get_sync_mapping(&conn, entity_type, entity_id)
    }

    async fn upsert_sync_mapping(&self, mapping: &SyncMapping) -> Result<(), DomainError> {
        let conn = self.lock()?;
        sync_mappings::upsert_sync_mapping(&conn, mapping)
    }

    async fn get_sync_mapping_by_plane_id(
        &self,
        entity_type: &str,
        plane_issue_id: &str,
    ) -> Result<Option<SyncMapping>, DomainError> {
        let conn = self.lock()?;
        sync_mappings::get_sync_mapping_by_plane_id(&conn, entity_type, plane_issue_id)
    }

    async fn delete_sync_mapping(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<(), DomainError> {
        let conn = self.lock()?;
        sync_mappings::delete_sync_mapping(&conn, entity_type, entity_id)
    }

    async fn create_project(&self, project: &Project) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        projects::create_project(&conn, project)
    }

    async fn get_project_by_slug(&self, slug: &str) -> Result<Option<Project>, DomainError> {
        let conn = self.lock()?;
        projects::get_project_by_slug(&conn, slug)
    }
}

impl ContentStoragePort for SqliteStorageAdapter {
    async fn create_feature(&self, feature: &Feature) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        features::create_feature(&conn, feature)
    }

    async fn get_feature_by_slug(&self, slug: &str) -> Result<Option<Feature>, DomainError> {
        let conn = self.lock()?;
        features::get_feature_by_slug(&conn, slug)
    }

    async fn get_feature_by_id(&self, id: i64) -> Result<Option<Feature>, DomainError> {
        let conn = self.lock()?;
        features::get_feature_by_id(&conn, id)
    }

    async fn update_feature_state(&self, id: i64, state: FeatureState) -> Result<(), DomainError> {
        let conn = self.lock()?;
        features::update_feature_state(&conn, id, state)
    }

    async fn update_feature(&self, feature: &Feature) -> Result<(), DomainError> {
        let conn = self.lock()?;
        features::update_feature(&conn, feature)
    }

    async fn list_features_by_state(
        &self,
        state: FeatureState,
    ) -> Result<Vec<Feature>, DomainError> {
        let conn = self.lock()?;
        features::list_features_by_state(&conn, state)
    }

    async fn list_all_features(&self) -> Result<Vec<Feature>, DomainError> {
        let conn = self.lock()?;
        features::list_all_features(&conn)
    }

    async fn create_backlog_item(&self, item: &BacklogItem) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        backlog::create_backlog_item(&conn, item)
    }

    async fn get_backlog_item(&self, id: i64) -> Result<Option<BacklogItem>, DomainError> {
        let conn = self.lock()?;
        backlog::get_backlog_item(&conn, id)
    }

    async fn list_backlog_items(
        &self,
        filters: &BacklogFilters,
    ) -> Result<Vec<BacklogItem>, DomainError> {
        let conn = self.lock()?;
        backlog::list_backlog_items(&conn, filters)
    }

    async fn update_backlog_status(
        &self,
        id: i64,
        status: BacklogStatus,
    ) -> Result<(), DomainError> {
        let conn = self.lock()?;
        backlog::update_backlog_status(&conn, id, status)
    }

    async fn update_backlog_priority(
        &self,
        id: i64,
        priority: BacklogPriority,
    ) -> Result<(), DomainError> {
        let conn = self.lock()?;
        backlog::update_backlog_priority(&conn, id, priority)
    }

    async fn pop_next_backlog_item(&self) -> Result<Option<BacklogItem>, DomainError> {
        let conn = self.lock()?;
        backlog::pop_next_backlog_item(&conn)
    }

    async fn create_work_package(&self, wp: &WorkPackage) -> Result<i64, DomainError> {
        let conn = self.lock()?;
        work_packages::create_work_package(&conn, wp)
    }

    async fn get_work_package(&self, id: i64) -> Result<Option<WorkPackage>, DomainError> {
        let conn = self.lock()?;
        work_packages::get_work_package(&conn, id)
    }

    async fn update_wp_state(&self, id: i64, state: WpState) -> Result<(), DomainError> {
        let conn = self.lock()?;
        work_packages::update_wp_state(&conn, id, state)
    }

    async fn update_work_package(&self, wp: &WorkPackage) -> Result<(), DomainError> {
        let conn = self.lock()?;
        work_packages::update_work_package(&conn, wp)
    }

    async fn list_wps_by_feature(&self, feature_id: i64) -> Result<Vec<WorkPackage>, DomainError> {
        let conn = self.lock()?;
        work_packages::list_wps_by_feature(&conn, feature_id)
    }

    async fn add_wp_dependency(&self, dep: &WpDependency) -> Result<(), DomainError> {
        let conn = self.lock()?;
        work_packages::add_wp_dependency(&conn, dep)
    }

    async fn get_wp_dependencies(&self, wp_id: i64) -> Result<Vec<WpDependency>, DomainError> {
        let conn = self.lock()?;
        work_packages::get_wp_dependencies(&conn, wp_id)
    }

    async fn get_ready_wps(&self, feature_id: i64) -> Result<Vec<WorkPackage>, DomainError> {
        let conn = self.lock()?;
        work_packages::get_ready_wps(&conn, feature_id)
    }
}

#[async_trait::async_trait]
impl EventStore for SqliteStorageAdapter {
    async fn append(&self, event: &Event) -> Result<i64, EventError> {
        let conn = self
            .lock()
            .map_err(|e| EventError::StorageError(e.to_string()))?;
        events::append_event(&conn, event).map_err(|e| EventError::StorageError(e.to_string()))
    }

    async fn get_events(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<Vec<Event>, EventError> {
        let conn = self
            .lock()
            .map_err(|e| EventError::StorageError(e.to_string()))?;
        events::get_events(&conn, entity_type, entity_id)
            .map_err(|e| EventError::StorageError(e.to_string()))
    }

    async fn get_events_since(
        &self,
        entity_type: &str,
        entity_id: i64,
        sequence: i64,
    ) -> Result<Vec<Event>, EventError> {
        let conn = self
            .lock()
            .map_err(|e| EventError::StorageError(e.to_string()))?;
        events::get_events_since(&conn, entity_type, entity_id, sequence)
            .map_err(|e| EventError::StorageError(e.to_string()))
    }

    async fn get_events_by_range(
        &self,
        entity_type: &str,
        entity_id: i64,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Event>, EventError> {
        let conn = self
            .lock()
            .map_err(|e| EventError::StorageError(e.to_string()))?;
        events::get_events_by_range(
            &conn,
            entity_type,
            entity_id,
            &from.to_rfc3339(),
            &to.to_rfc3339(),
        )
        .map_err(|e| EventError::StorageError(e.to_string()))
    }

    async fn get_latest_sequence(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<i64, EventError> {
        let conn = self
            .lock()
            .map_err(|e| EventError::StorageError(e.to_string()))?;
        events::get_latest_sequence(&conn, entity_type, entity_id)
            .map_err(|e| EventError::StorageError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::domain::{
        audit::{AuditEntry, hash_entry},
        feature::Feature,
        governance::{
            Evidence, EvidenceType, GovernanceContract, GovernanceRule, PolicyCheck,
            PolicyDefinition, PolicyDomain, PolicyRule,
        },
        metric::Metric,
        state_machine::FeatureState,
        work_package::{DependencyType, WorkPackage, WpDependency, WpState},
    };

    fn make_adapter() -> SqliteStorageAdapter {
        SqliteStorageAdapter::in_memory().expect("in-memory adapter")
    }

    // -- Feature tests --

    #[tokio::test]
    async fn feature_create_and_get_by_slug() {
        let db = make_adapter();
        let f = Feature::new("my-feat", "My Feature", [0u8; 32], None);
        let id = StoragePort::create_feature(&db, &f).await.unwrap();
        assert!(id > 0);

        let got = StoragePort::get_feature_by_slug(&db, "my-feat")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got.id, id);
        assert_eq!(got.slug, "my-feat");
        assert_eq!(got.friendly_name, "My Feature");
        assert_eq!(got.spec_hash, [0u8; 32]);
        assert_eq!(got.state, FeatureState::Created);
    }

    #[tokio::test]
    async fn feature_get_by_id() {
        let db = make_adapter();
        let f = Feature::new("feat-id", "Feat", [1u8; 32], None);
        let id = StoragePort::create_feature(&db, &f).await.unwrap();
        let got = StoragePort::get_feature_by_id(&db, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got.slug, "feat-id");
    }

    #[tokio::test]
    async fn feature_update_state() {
        let db = make_adapter();
        let f = Feature::new("upd-feat", "Upd", [0u8; 32], None);
        let id = StoragePort::create_feature(&db, &f).await.unwrap();

        StoragePort::update_feature_state(&db, id, FeatureState::Specified)
            .await
            .unwrap();
        let got = StoragePort::get_feature_by_id(&db, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got.state, FeatureState::Specified);
    }

    #[tokio::test]
    async fn feature_list_by_state() {
        let db = make_adapter();
        let f1 = Feature::new("f1", "F1", [0u8; 32], None);
        let f2 = Feature::new("f2", "F2", [0u8; 32], None);
        let id1 = StoragePort::create_feature(&db, &f1).await.unwrap();
        let _id2 = StoragePort::create_feature(&db, &f2).await.unwrap();
        StoragePort::update_feature_state(&db, id1, FeatureState::Specified)
            .await
            .unwrap();

        let specified = StoragePort::list_features_by_state(&db, FeatureState::Specified)
            .await
            .unwrap();
        assert_eq!(specified.len(), 1);
        assert_eq!(specified[0].slug, "f1");

        let created = StoragePort::list_features_by_state(&db, FeatureState::Created)
            .await
            .unwrap();
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].slug, "f2");
    }

    #[tokio::test]
    async fn feature_list_all() {
        let db = make_adapter();
        StoragePort::create_feature(&db, &Feature::new("aa", "AA", [0u8; 32], None))
            .await
            .unwrap();
        StoragePort::create_feature(&db, &Feature::new("bb", "BB", [0u8; 32], None))
            .await
            .unwrap();
        let all = StoragePort::list_all_features(&db).await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn feature_duplicate_slug_fails() {
        let db = make_adapter();
        let f = Feature::new("dup", "Dup", [0u8; 32], None);
        StoragePort::create_feature(&db, &f).await.unwrap();
        let result = StoragePort::create_feature(&db, &f).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn feature_not_found_returns_none() {
        let db = make_adapter();
        let got = StoragePort::get_feature_by_slug(&db, "no-such-slug")
            .await
            .unwrap();
        assert!(got.is_none());
        let got2 = StoragePort::get_feature_by_id(&db, 9999).await.unwrap();
        assert!(got2.is_none());
    }

    // -- Work Package tests --

    #[tokio::test]
    async fn wp_create_and_get() {
        let db = make_adapter();
        let feat = Feature::new("wp-feat", "WP Feat", [0u8; 32], None);
        let fid = StoragePort::create_feature(&db, &feat).await.unwrap();

        let mut wp = WorkPackage::new(fid, "Task A", 1, "criteria");
        wp.file_scope = vec!["src/main.rs".into()];
        let wp_id = StoragePort::create_work_package(&db, &wp).await.unwrap();
        assert!(wp_id > 0);

        let got = StoragePort::get_work_package(&db, wp_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got.title, "Task A");
        assert_eq!(got.file_scope, vec!["src/main.rs"]);
        assert_eq!(got.state, WpState::Planned);
        assert_eq!(got.feature_id, fid);
    }

    #[tokio::test]
    async fn wp_update_state() {
        let db = make_adapter();
        let fid = StoragePort::create_feature(&db, &Feature::new("f", "F", [0u8; 32], None))
            .await
            .unwrap();
        let wp_id = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "T", 1, "c"))
            .await
            .unwrap();
        StoragePort::update_wp_state(&db, wp_id, WpState::Doing)
            .await
            .unwrap();
        let got = StoragePort::get_work_package(&db, wp_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got.state, WpState::Doing);
    }

    #[tokio::test]
    async fn wp_list_by_feature_ordered_by_sequence() {
        let db = make_adapter();
        let fid = StoragePort::create_feature(&db, &Feature::new("f2", "F2", [0u8; 32], None))
            .await
            .unwrap();
        StoragePort::create_work_package(&db, &WorkPackage::new(fid, "B", 2, "c"))
            .await
            .unwrap();
        StoragePort::create_work_package(&db, &WorkPackage::new(fid, "A", 1, "c"))
            .await
            .unwrap();

        let wps = StoragePort::list_wps_by_feature(&db, fid).await.unwrap();
        assert_eq!(wps.len(), 2);
        assert_eq!(wps[0].sequence, 1);
        assert_eq!(wps[1].sequence, 2);
    }

    #[tokio::test]
    async fn wp_dependencies_and_ready_wps() {
        let db = make_adapter();
        let fid = StoragePort::create_feature(&db, &Feature::new("f3", "F3", [0u8; 32], None))
            .await
            .unwrap();
        let wp1 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP1", 1, "c"))
            .await
            .unwrap();
        let wp2 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP2", 2, "c"))
            .await
            .unwrap();

        // wp2 depends on wp1
        StoragePort::add_wp_dependency(
            &db,
            &WpDependency {
                wp_id: wp2,
                depends_on: wp1,
                dep_type: DependencyType::Explicit,
            },
        )
        .await
        .unwrap();

        // Both planned but wp2 has unsatisfied dep -> only wp1 is ready
        let ready = StoragePort::get_ready_wps(&db, fid).await.unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, wp1);

        // Complete wp1
        StoragePort::update_wp_state(&db, wp1, WpState::Doing)
            .await
            .unwrap();
        StoragePort::update_wp_state(&db, wp1, WpState::Review)
            .await
            .unwrap();
        StoragePort::update_wp_state(&db, wp1, WpState::Done)
            .await
            .unwrap();

        // Now wp2 should be ready
        let ready = StoragePort::get_ready_wps(&db, fid).await.unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, wp2);
    }

    #[tokio::test]
    async fn wp_get_dependencies() {
        let db = make_adapter();
        let fid = StoragePort::create_feature(&db, &Feature::new("fd", "FD", [0u8; 32], None))
            .await
            .unwrap();
        let w1 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "W1", 1, "c"))
            .await
            .unwrap();
        let w2 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "W2", 2, "c"))
            .await
            .unwrap();
        StoragePort::add_wp_dependency(
            &db,
            &WpDependency {
                wp_id: w2,
                depends_on: w1,
                dep_type: DependencyType::Data,
            },
        )
        .await
        .unwrap();
        let deps = StoragePort::get_wp_dependencies(&db, w2).await.unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].depends_on, w1);
    }

    // -- Audit tests --

    fn make_audit_entry(feature_id: i64, prev_hash: [u8; 32]) -> AuditEntry {
        let mut entry = AuditEntry {
            id: 0,
            feature_id,
            wp_id: None,
            timestamp: chrono::Utc::now(),
            actor: "agent".into(),
            transition: "created->specified".into(),
            evidence_refs: vec![],
            prev_hash,
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        };
        entry.hash = hash_entry(&entry);
        entry
    }

    #[tokio::test]
    async fn audit_append_and_trail() {
        let db = make_adapter();
        let fid = StoragePort::create_feature(&db, &Feature::new("af", "AF", [0u8; 32], None))
            .await
            .unwrap();

        let e1 = make_audit_entry(fid, [0u8; 32]);
        let _id1 = StoragePort::append_audit_entry(&db, &e1).await.unwrap();

        let e2 = make_audit_entry(fid, e1.hash);
        let _id2 = StoragePort::append_audit_entry(&db, &e2).await.unwrap();

        let e3 = make_audit_entry(fid, e2.hash);
        StoragePort::append_audit_entry(&db, &e3).await.unwrap();

        let trail = StoragePort::get_audit_trail(&db, fid).await.unwrap();
        assert_eq!(trail.len(), 3);
        // Ordered chronologically
        assert!(trail[0].id <= trail[1].id);
        assert!(trail[1].id <= trail[2].id);
    }

    #[tokio::test]
    async fn audit_wrong_prev_hash_rejected() {
        let db = make_adapter();
        let fid = StoragePort::create_feature(&db, &Feature::new("afc", "AFC", [0u8; 32], None))
            .await
            .unwrap();

        let e1 = make_audit_entry(fid, [0u8; 32]);
        StoragePort::append_audit_entry(&db, &e1).await.unwrap();

        // Entry with wrong prev_hash
        let e_bad = make_audit_entry(fid, [0xFFu8; 32]);
        let result = StoragePort::append_audit_entry(&db, &e_bad).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn audit_get_latest() {
        let db = make_adapter();
        let fid = StoragePort::create_feature(&db, &Feature::new("al", "AL", [0u8; 32], None))
            .await
            .unwrap();

        assert!(
            StoragePort::get_latest_audit_entry(&db, fid)
                .await
                .unwrap()
                .is_none()
        );

        let e1 = make_audit_entry(fid, [0u8; 32]);
        StoragePort::append_audit_entry(&db, &e1).await.unwrap();

        let e2 = make_audit_entry(fid, e1.hash);
        StoragePort::append_audit_entry(&db, &e2).await.unwrap();

        let latest = StoragePort::get_latest_audit_entry(&db, fid)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(latest.hash, e2.hash);
    }

    // -- Evidence tests --

    #[tokio::test]
    async fn evidence_create_and_get_by_wp() {
        let db = make_adapter();
        let fid = StoragePort::create_feature(&db, &Feature::new("ef", "EF", [0u8; 32], None))
            .await
            .unwrap();
        let wp_id = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP", 1, "c"))
            .await
            .unwrap();

        let ev = Evidence {
            id: 0,
            wp_id,
            fr_id: "FR-001".into(),
            evidence_type: EvidenceType::TestResult,
            artifact_path: "results/test.xml".into(),
            metadata: Some(serde_json::json!({"pass": 42})),
            created_at: chrono::Utc::now(),
        };
        let ev_id = StoragePort::create_evidence(&db, &ev).await.unwrap();
        assert!(ev_id > 0);

        let evs = StoragePort::get_evidence_by_wp(&db, wp_id).await.unwrap();
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].fr_id, "FR-001");
        assert_eq!(evs[0].evidence_type, EvidenceType::TestResult);
        assert!(evs[0].metadata.is_some());
    }

    #[tokio::test]
    async fn evidence_get_by_fr() {
        let db = make_adapter();
        let fid = StoragePort::create_feature(&db, &Feature::new("efr", "EFR", [0u8; 32], None))
            .await
            .unwrap();
        let wp_id = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP", 1, "c"))
            .await
            .unwrap();

        let ev1 = Evidence {
            id: 0,
            wp_id,
            fr_id: "FR-001".into(),
            evidence_type: EvidenceType::CiOutput,
            artifact_path: "ci.log".into(),
            metadata: None,
            created_at: chrono::Utc::now(),
        };
        let ev2 = Evidence {
            id: 0,
            wp_id,
            fr_id: "FR-002".into(),
            evidence_type: EvidenceType::LintResult,
            artifact_path: "lint.log".into(),
            metadata: None,
            created_at: chrono::Utc::now(),
        };
        StoragePort::create_evidence(&db, &ev1).await.unwrap();
        StoragePort::create_evidence(&db, &ev2).await.unwrap();

        let fr1 = StoragePort::get_evidence_by_fr(&db, "FR-001")
            .await
            .unwrap();
        assert_eq!(fr1.len(), 1);
        assert_eq!(fr1[0].fr_id, "FR-001");
    }

    // -- Policy tests --

    #[tokio::test]
    async fn policy_create_and_list_active() {
        let db = make_adapter();
        let rule = PolicyRule {
            id: 0,
            domain: PolicyDomain::Quality,
            rule: PolicyDefinition {
                description: "All tests must pass".into(),
                check: PolicyCheck::ManualApproval,
            },
            active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let id = StoragePort::create_policy_rule(&db, &rule).await.unwrap();
        assert!(id > 0);

        let active = StoragePort::list_active_policies(&db).await.unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].domain, PolicyDomain::Quality);
    }

    // -- Governance Contract tests --

    #[tokio::test]
    async fn governance_contract_create_and_get() {
        let db = make_adapter();
        let fid = StoragePort::create_feature(&db, &Feature::new("gc", "GC", [0u8; 32], None))
            .await
            .unwrap();

        let contract = GovernanceContract {
            id: 0,
            feature_id: fid,
            version: 1,
            rules: vec![GovernanceRule {
                transition: "created->specified".into(),
                required_evidence: vec![],
                policy_refs: vec![],
            }],
            bound_at: chrono::Utc::now(),
        };
        let cid = StoragePort::create_governance_contract(&db, &contract)
            .await
            .unwrap();
        assert!(cid > 0);

        let got = StoragePort::get_governance_contract(&db, fid, 1)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got.feature_id, fid);
        assert_eq!(got.version, 1);
        assert_eq!(got.rules.len(), 1);

        let latest = StoragePort::get_latest_governance_contract(&db, fid)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(latest.version, 1);
    }

    #[tokio::test]
    async fn governance_contract_versioning() {
        let db = make_adapter();
        let fid = StoragePort::create_feature(&db, &Feature::new("gcv", "GCV", [0u8; 32], None))
            .await
            .unwrap();

        let c1 = GovernanceContract {
            id: 0,
            feature_id: fid,
            version: 1,
            rules: vec![],
            bound_at: chrono::Utc::now(),
        };
        let c2 = GovernanceContract {
            id: 0,
            feature_id: fid,
            version: 2,
            rules: vec![],
            bound_at: chrono::Utc::now(),
        };
        StoragePort::create_governance_contract(&db, &c1)
            .await
            .unwrap();
        StoragePort::create_governance_contract(&db, &c2)
            .await
            .unwrap();

        let latest = StoragePort::get_latest_governance_contract(&db, fid)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(latest.version, 2);

        let v1 = StoragePort::get_governance_contract(&db, fid, 1)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(v1.version, 1);
    }

    // -- Metrics tests --

    #[tokio::test]
    async fn metric_record_and_get() {
        let db = make_adapter();
        let fid = StoragePort::create_feature(&db, &Feature::new("mf", "MF", [0u8; 32], None))
            .await
            .unwrap();

        let m = Metric {
            id: 0,
            feature_id: Some(fid),
            command: "spec-kitty implement".into(),
            duration_ms: 1234,
            agent_runs: 3,
            review_cycles: 1,
            metadata: Some(serde_json::json!({"model": "claude"})),
            timestamp: chrono::Utc::now(),
        };
        let mid = StoragePort::record_metric(&db, &m).await.unwrap();
        assert!(mid > 0);

        let ms = StoragePort::get_metrics_by_feature(&db, fid).await.unwrap();
        assert_eq!(ms.len(), 1);
        assert_eq!(ms[0].command, "spec-kitty implement");
        assert_eq!(ms[0].duration_ms, 1234);
        assert!(ms[0].metadata.is_some());
    }

    // -- Module tests --

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
        let got = StoragePort::get_module_by_slug(&db, "billing")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got.id, id);
    }

    #[tokio::test]
    async fn module_not_found_returns_none() {
        let db = make_adapter();
        assert!(StoragePort::get_module(&db, 9999).await.unwrap().is_none());
        assert!(
            StoragePort::get_module_by_slug(&db, "no-such")
                .await
                .unwrap()
                .is_none()
        );
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
        // Create feature and link it via tag
        let f = Feature::new("feat", "Feat", [0u8; 32], None);
        let fid = StoragePort::create_feature(&db, &f).await.unwrap();
        // Manually set module_id via tag so the module owns this feature
        // (we use the raw feature.module_id path by updating directly)
        {
            let conn = db.conn_for_bench().unwrap();
            conn.execute(
                "UPDATE features SET module_id = ?1 WHERE id = ?2",
                rusqlite::params![mid, fid],
            )
            .unwrap();
        }

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
        let r1 = StoragePort::create_module(&db, &Module::new("Root1", None))
            .await
            .unwrap();
        let r2 = StoragePort::create_module(&db, &Module::new("Root2", None))
            .await
            .unwrap();
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
        let mid = StoragePort::create_module(&db, &Module::new("M", None))
            .await
            .unwrap();
        let fid = StoragePort::create_feature(&db, &Feature::new("f-tag", "FTag", [0u8; 32], None))
            .await
            .unwrap();

        let tag = ModuleFeatureTag::new(mid, fid);
        StoragePort::tag_feature_to_module(&db, &tag).await.unwrap();
        // Idempotent -- should not fail
        StoragePort::tag_feature_to_module(&db, &tag).await.unwrap();

        let mwf = StoragePort::get_module_with_features(&db, mid)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(mwf.tagged_features.len(), 1);
        assert_eq!(mwf.tagged_features[0].id, fid);

        StoragePort::untag_feature_from_module(&db, mid, fid)
            .await
            .unwrap();
        let mwf2 = StoragePort::get_module_with_features(&db, mid)
            .await
            .unwrap()
            .unwrap();
        assert!(mwf2.tagged_features.is_empty());
    }

    #[tokio::test]
    async fn module_get_with_features_none_for_missing() {
        let db = make_adapter();
        assert!(
            StoragePort::get_module_with_features(&db, 9999)
                .await
                .unwrap()
                .is_none()
        );
    }

    // -- Cycle tests --

    use agileplus_domain::domain::cycle::{Cycle, CycleFeature, CycleState};
    use chrono::NaiveDate;

    fn make_date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
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
        StoragePort::update_cycle_state(&db, id, CycleState::Active)
            .await
            .unwrap();
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

        let drafts = StoragePort::list_cycles_by_state(&db, CycleState::Draft)
            .await
            .unwrap();
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].id, id2);

        let actives = StoragePort::list_cycles_by_state(&db, CycleState::Active)
            .await
            .unwrap();
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
        let fid =
            StoragePort::create_feature(&db, &Feature::new("cyc-feat", "CycFeat", [0u8; 32], None))
                .await
                .unwrap();

        let entry = CycleFeature::new(cid, fid);
        StoragePort::add_feature_to_cycle(&db, &entry)
            .await
            .unwrap();
        // Idempotent
        StoragePort::add_feature_to_cycle(&db, &entry)
            .await
            .unwrap();

        let cwf = StoragePort::get_cycle_with_features(&db, cid)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(cwf.features.len(), 1);
        assert_eq!(cwf.features[0].id, fid);

        StoragePort::remove_feature_from_cycle(&db, cid, fid)
            .await
            .unwrap();
        let cwf2 = StoragePort::get_cycle_with_features(&db, cid)
            .await
            .unwrap()
            .unwrap();
        assert!(cwf2.features.is_empty());
    }

    #[tokio::test]
    async fn cycle_with_features_none_for_missing() {
        let db = make_adapter();
        assert!(
            StoragePort::get_cycle_with_features(&db, 9999)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn cycle_module_scope_enforcement() {
        let db = make_adapter();
        let mid = StoragePort::create_module(&db, &Module::new("Scope", None))
            .await
            .unwrap();
        // Cycle scoped to this module
        let c = Cycle::new(
            "Scoped-Cycle",
            make_date(2026, 1, 1),
            make_date(2026, 2, 1),
            Some(mid),
        )
        .unwrap();
        let cid = StoragePort::create_cycle(&db, &c).await.unwrap();

        // Feature NOT in module scope
        let fid =
            StoragePort::create_feature(&db, &Feature::new("out-of-scope", "OOS", [0u8; 32], None))
                .await
                .unwrap();
        let entry = CycleFeature::new(cid, fid);
        let result = StoragePort::add_feature_to_cycle(&db, &entry).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            agileplus_domain::error::DomainError::FeatureNotInModuleScope { .. }
        ));

        // Tag feature to module -- now it should work
        StoragePort::tag_feature_to_module(&db, &ModuleFeatureTag::new(mid, fid))
            .await
            .unwrap();
        StoragePort::add_feature_to_cycle(&db, &CycleFeature::new(cid, fid))
            .await
            .unwrap();
        let cwf = StoragePort::get_cycle_with_features(&db, cid)
            .await
            .unwrap()
            .unwrap();
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
        let fid =
            StoragePort::create_feature(&db, &Feature::new("prog-feat", "Prog", [0u8; 32], None))
                .await
                .unwrap();
        StoragePort::add_feature_to_cycle(&db, &CycleFeature::new(cid, fid))
            .await
            .unwrap();

        // Create 2 WPs
        let _wp1 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP1", 1, "c"))
            .await
            .unwrap();
        let wp2 = StoragePort::create_work_package(&db, &WorkPackage::new(fid, "WP2", 2, "c"))
            .await
            .unwrap();
        StoragePort::update_wp_state(
            &db,
            wp2,
            agileplus_domain::domain::work_package::WpState::Done,
        )
        .await
        .unwrap();

        let cwf = StoragePort::get_cycle_with_features(&db, cid)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(cwf.wp_progress.total, 2);
        assert_eq!(cwf.wp_progress.planned, 1);
        assert_eq!(cwf.wp_progress.done, 1);
    }
}
