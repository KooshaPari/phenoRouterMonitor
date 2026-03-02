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
        feature::Feature,
        governance::{Evidence, GovernanceContract, PolicyRule},
        metric::Metric,
        state_machine::FeatureState,
        work_package::{WpDependency, WpState, WorkPackage},
    },
    error::DomainError,
    ports::StoragePort,
};

use agileplus_domain::domain::event::Event;
use agileplus_events::{EventError, EventStore};

use crate::migrations::MigrationRunner;
use crate::repository::{audit, events, evidence, features, governance, metrics, work_packages};

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
    pub fn conn_for_bench(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, Connection>, DomainError> {
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

    async fn list_features_by_state(&self, state: FeatureState) -> Result<Vec<Feature>, DomainError> {
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
}

#[async_trait::async_trait]
impl EventStore for SqliteStorageAdapter {
    async fn append(&self, event: &Event) -> Result<i64, EventError> {
        let conn = self.lock().map_err(|e| EventError::StorageError(e.to_string()))?;
        events::append_event(&conn, event).map_err(|e| EventError::StorageError(e.to_string()))
    }

    async fn get_events(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<Vec<Event>, EventError> {
        let conn = self.lock().map_err(|e| EventError::StorageError(e.to_string()))?;
        events::get_events(&conn, entity_type, entity_id)
            .map_err(|e| EventError::StorageError(e.to_string()))
    }

    async fn get_events_since(
        &self,
        entity_type: &str,
        entity_id: i64,
        sequence: i64,
    ) -> Result<Vec<Event>, EventError> {
        let conn = self.lock().map_err(|e| EventError::StorageError(e.to_string()))?;
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
        let conn = self.lock().map_err(|e| EventError::StorageError(e.to_string()))?;
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
        let conn = self.lock().map_err(|e| EventError::StorageError(e.to_string()))?;
        events::get_latest_sequence(&conn, entity_type, entity_id)
            .map_err(|e| EventError::StorageError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::domain::{
        audit::{hash_entry, AuditEntry},
        feature::Feature,
        governance::{
            Evidence, EvidenceType, GovernanceContract, GovernanceRule, PolicyCheck,
            PolicyDefinition, PolicyDomain, PolicyRule,
        },
        metric::Metric,
        state_machine::FeatureState,
        work_package::{DependencyType, WpDependency, WpState, WorkPackage},
    };

    fn make_adapter() -> SqliteStorageAdapter {
        SqliteStorageAdapter::in_memory().expect("in-memory adapter")
    }

    // -- Feature tests --

    #[tokio::test]
    async fn feature_create_and_get_by_slug() {
        let db = make_adapter();
        let f = Feature::new("my-feat", "My Feature", [0u8; 32], None);
        let id = db.create_feature(&f).await.unwrap();
        assert!(id > 0);

        let got = db.get_feature_by_slug("my-feat").await.unwrap().unwrap();
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
        let id = db.create_feature(&f).await.unwrap();
        let got = db.get_feature_by_id(id).await.unwrap().unwrap();
        assert_eq!(got.slug, "feat-id");
    }

    #[tokio::test]
    async fn feature_update_state() {
        let db = make_adapter();
        let f = Feature::new("upd-feat", "Upd", [0u8; 32], None);
        let id = db.create_feature(&f).await.unwrap();

        db.update_feature_state(id, FeatureState::Specified).await.unwrap();
        let got = db.get_feature_by_id(id).await.unwrap().unwrap();
        assert_eq!(got.state, FeatureState::Specified);
    }

    #[tokio::test]
    async fn feature_list_by_state() {
        let db = make_adapter();
        let f1 = Feature::new("f1", "F1", [0u8; 32], None);
        let f2 = Feature::new("f2", "F2", [0u8; 32], None);
        let id1 = db.create_feature(&f1).await.unwrap();
        let _id2 = db.create_feature(&f2).await.unwrap();
        db.update_feature_state(id1, FeatureState::Specified).await.unwrap();

        let specified = db.list_features_by_state(FeatureState::Specified).await.unwrap();
        assert_eq!(specified.len(), 1);
        assert_eq!(specified[0].slug, "f1");

        let created = db.list_features_by_state(FeatureState::Created).await.unwrap();
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].slug, "f2");
    }

    #[tokio::test]
    async fn feature_list_all() {
        let db = make_adapter();
        db.create_feature(&Feature::new("aa", "AA", [0u8; 32], None)).await.unwrap();
        db.create_feature(&Feature::new("bb", "BB", [0u8; 32], None)).await.unwrap();
        let all = db.list_all_features().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn feature_duplicate_slug_fails() {
        let db = make_adapter();
        let f = Feature::new("dup", "Dup", [0u8; 32], None);
        db.create_feature(&f).await.unwrap();
        let result = db.create_feature(&f).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn feature_not_found_returns_none() {
        let db = make_adapter();
        let got = db.get_feature_by_slug("no-such-slug").await.unwrap();
        assert!(got.is_none());
        let got2 = db.get_feature_by_id(9999).await.unwrap();
        assert!(got2.is_none());
    }

    // -- Work Package tests --

    #[tokio::test]
    async fn wp_create_and_get() {
        let db = make_adapter();
        let feat = Feature::new("wp-feat", "WP Feat", [0u8; 32], None);
        let fid = db.create_feature(&feat).await.unwrap();

        let mut wp = WorkPackage::new(fid, "Task A", 1, "criteria");
        wp.file_scope = vec!["src/main.rs".into()];
        let wp_id = db.create_work_package(&wp).await.unwrap();
        assert!(wp_id > 0);

        let got = db.get_work_package(wp_id).await.unwrap().unwrap();
        assert_eq!(got.title, "Task A");
        assert_eq!(got.file_scope, vec!["src/main.rs"]);
        assert_eq!(got.state, WpState::Planned);
        assert_eq!(got.feature_id, fid);
    }

    #[tokio::test]
    async fn wp_update_state() {
        let db = make_adapter();
        let fid = db.create_feature(&Feature::new("f", "F", [0u8; 32], None)).await.unwrap();
        let wp_id = db.create_work_package(&WorkPackage::new(fid, "T", 1, "c")).await.unwrap();
        db.update_wp_state(wp_id, WpState::Doing).await.unwrap();
        let got = db.get_work_package(wp_id).await.unwrap().unwrap();
        assert_eq!(got.state, WpState::Doing);
    }

    #[tokio::test]
    async fn wp_list_by_feature_ordered_by_sequence() {
        let db = make_adapter();
        let fid = db.create_feature(&Feature::new("f2", "F2", [0u8; 32], None)).await.unwrap();
        db.create_work_package(&WorkPackage::new(fid, "B", 2, "c")).await.unwrap();
        db.create_work_package(&WorkPackage::new(fid, "A", 1, "c")).await.unwrap();

        let wps = db.list_wps_by_feature(fid).await.unwrap();
        assert_eq!(wps.len(), 2);
        assert_eq!(wps[0].sequence, 1);
        assert_eq!(wps[1].sequence, 2);
    }

    #[tokio::test]
    async fn wp_dependencies_and_ready_wps() {
        let db = make_adapter();
        let fid = db.create_feature(&Feature::new("f3", "F3", [0u8; 32], None)).await.unwrap();
        let wp1 = db.create_work_package(&WorkPackage::new(fid, "WP1", 1, "c")).await.unwrap();
        let wp2 = db.create_work_package(&WorkPackage::new(fid, "WP2", 2, "c")).await.unwrap();

        // wp2 depends on wp1
        db.add_wp_dependency(&WpDependency {
            wp_id: wp2,
            depends_on: wp1,
            dep_type: DependencyType::Explicit,
        })
        .await
        .unwrap();

        // Both planned but wp2 has unsatisfied dep -> only wp1 is ready
        let ready = db.get_ready_wps(fid).await.unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, wp1);

        // Complete wp1
        db.update_wp_state(wp1, WpState::Doing).await.unwrap();
        db.update_wp_state(wp1, WpState::Review).await.unwrap();
        db.update_wp_state(wp1, WpState::Done).await.unwrap();

        // Now wp2 should be ready
        let ready = db.get_ready_wps(fid).await.unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, wp2);
    }

    #[tokio::test]
    async fn wp_get_dependencies() {
        let db = make_adapter();
        let fid = db.create_feature(&Feature::new("fd", "FD", [0u8; 32], None)).await.unwrap();
        let w1 = db.create_work_package(&WorkPackage::new(fid, "W1", 1, "c")).await.unwrap();
        let w2 = db.create_work_package(&WorkPackage::new(fid, "W2", 2, "c")).await.unwrap();
        db.add_wp_dependency(&WpDependency { wp_id: w2, depends_on: w1, dep_type: DependencyType::Data })
            .await.unwrap();
        let deps = db.get_wp_dependencies(w2).await.unwrap();
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
        let fid = db.create_feature(&Feature::new("af", "AF", [0u8; 32], None)).await.unwrap();

        let e1 = make_audit_entry(fid, [0u8; 32]);
        let _id1 = db.append_audit_entry(&e1).await.unwrap();

        let e2 = make_audit_entry(fid, e1.hash);
        let _id2 = db.append_audit_entry(&e2).await.unwrap();

        let e3 = make_audit_entry(fid, e2.hash);
        db.append_audit_entry(&e3).await.unwrap();

        let trail = db.get_audit_trail(fid).await.unwrap();
        assert_eq!(trail.len(), 3);
        // Ordered chronologically
        assert!(trail[0].id <= trail[1].id);
        assert!(trail[1].id <= trail[2].id);
    }

    #[tokio::test]
    async fn audit_wrong_prev_hash_rejected() {
        let db = make_adapter();
        let fid = db.create_feature(&Feature::new("afc", "AFC", [0u8; 32], None)).await.unwrap();

        let e1 = make_audit_entry(fid, [0u8; 32]);
        db.append_audit_entry(&e1).await.unwrap();

        // Entry with wrong prev_hash
        let e_bad = make_audit_entry(fid, [0xFFu8; 32]);
        let result = db.append_audit_entry(&e_bad).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn audit_get_latest() {
        let db = make_adapter();
        let fid = db.create_feature(&Feature::new("al", "AL", [0u8; 32], None)).await.unwrap();

        assert!(db.get_latest_audit_entry(fid).await.unwrap().is_none());

        let e1 = make_audit_entry(fid, [0u8; 32]);
        db.append_audit_entry(&e1).await.unwrap();

        let e2 = make_audit_entry(fid, e1.hash);
        db.append_audit_entry(&e2).await.unwrap();

        let latest = db.get_latest_audit_entry(fid).await.unwrap().unwrap();
        assert_eq!(latest.hash, e2.hash);
    }

    // -- Evidence tests --

    #[tokio::test]
    async fn evidence_create_and_get_by_wp() {
        let db = make_adapter();
        let fid = db.create_feature(&Feature::new("ef", "EF", [0u8; 32], None)).await.unwrap();
        let wp_id = db.create_work_package(&WorkPackage::new(fid, "WP", 1, "c")).await.unwrap();

        let ev = Evidence {
            id: 0,
            wp_id,
            fr_id: "FR-001".into(),
            evidence_type: EvidenceType::TestResult,
            artifact_path: "results/test.xml".into(),
            metadata: Some(serde_json::json!({"pass": 42})),
            created_at: chrono::Utc::now(),
        };
        let ev_id = db.create_evidence(&ev).await.unwrap();
        assert!(ev_id > 0);

        let evs = db.get_evidence_by_wp(wp_id).await.unwrap();
        assert_eq!(evs.len(), 1);
        assert_eq!(evs[0].fr_id, "FR-001");
        assert_eq!(evs[0].evidence_type, EvidenceType::TestResult);
        assert!(evs[0].metadata.is_some());
    }

    #[tokio::test]
    async fn evidence_get_by_fr() {
        let db = make_adapter();
        let fid = db.create_feature(&Feature::new("efr", "EFR", [0u8; 32], None)).await.unwrap();
        let wp_id = db.create_work_package(&WorkPackage::new(fid, "WP", 1, "c")).await.unwrap();

        let ev1 = Evidence { id: 0, wp_id, fr_id: "FR-001".into(), evidence_type: EvidenceType::CiOutput,
            artifact_path: "ci.log".into(), metadata: None, created_at: chrono::Utc::now() };
        let ev2 = Evidence { id: 0, wp_id, fr_id: "FR-002".into(), evidence_type: EvidenceType::LintResult,
            artifact_path: "lint.log".into(), metadata: None, created_at: chrono::Utc::now() };
        db.create_evidence(&ev1).await.unwrap();
        db.create_evidence(&ev2).await.unwrap();

        let fr1 = db.get_evidence_by_fr("FR-001").await.unwrap();
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
        let id = db.create_policy_rule(&rule).await.unwrap();
        assert!(id > 0);

        let active = db.list_active_policies().await.unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].domain, PolicyDomain::Quality);
    }

    // -- Governance Contract tests --

    #[tokio::test]
    async fn governance_contract_create_and_get() {
        let db = make_adapter();
        let fid = db.create_feature(&Feature::new("gc", "GC", [0u8; 32], None)).await.unwrap();

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
        let cid = db.create_governance_contract(&contract).await.unwrap();
        assert!(cid > 0);

        let got = db.get_governance_contract(fid, 1).await.unwrap().unwrap();
        assert_eq!(got.feature_id, fid);
        assert_eq!(got.version, 1);
        assert_eq!(got.rules.len(), 1);

        let latest = db.get_latest_governance_contract(fid).await.unwrap().unwrap();
        assert_eq!(latest.version, 1);
    }

    #[tokio::test]
    async fn governance_contract_versioning() {
        let db = make_adapter();
        let fid = db.create_feature(&Feature::new("gcv", "GCV", [0u8; 32], None)).await.unwrap();

        let c1 = GovernanceContract { id: 0, feature_id: fid, version: 1, rules: vec![], bound_at: chrono::Utc::now() };
        let c2 = GovernanceContract { id: 0, feature_id: fid, version: 2, rules: vec![], bound_at: chrono::Utc::now() };
        db.create_governance_contract(&c1).await.unwrap();
        db.create_governance_contract(&c2).await.unwrap();

        let latest = db.get_latest_governance_contract(fid).await.unwrap().unwrap();
        assert_eq!(latest.version, 2);

        let v1 = db.get_governance_contract(fid, 1).await.unwrap().unwrap();
        assert_eq!(v1.version, 1);
    }

    // -- Metrics tests --

    #[tokio::test]
    async fn metric_record_and_get() {
        let db = make_adapter();
        let fid = db.create_feature(&Feature::new("mf", "MF", [0u8; 32], None)).await.unwrap();

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
        let mid = db.record_metric(&m).await.unwrap();
        assert!(mid > 0);

        let ms = db.get_metrics_by_feature(fid).await.unwrap();
        assert_eq!(ms.len(), 1);
        assert_eq!(ms[0].command, "spec-kitty implement");
        assert_eq!(ms[0].duration_ms, 1234);
        assert!(ms[0].metadata.is_some());
    }
}
