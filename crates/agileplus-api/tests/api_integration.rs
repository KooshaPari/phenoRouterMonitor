//! Integration tests for the AgilePlus HTTP API.
//!
//! These tests spin up a real axum test server backed by in-memory mock
//! implementations of all ports. No external dependencies are required.
//!
//! Run with: `cargo test -p agileplus-api`
//!
//! Traceability: WP15-T090

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use agileplus_api::{AppState, create_router};
use agileplus_domain::config::AppConfig;
use agileplus_domain::credentials::CredentialStore;
use agileplus_domain::credentials::InMemoryCredentialStore;
use agileplus_domain::credentials::keys as cred_keys;
use agileplus_domain::domain::audit::{AuditEntry, hash_entry};
use agileplus_domain::domain::backlog::{
    BacklogFilters, BacklogItem, BacklogPriority, BacklogStatus,
};
use agileplus_domain::domain::cycle::{Cycle, CycleFeature, CycleState, CycleWithFeatures};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::governance::{Evidence, GovernanceContract, PolicyRule};
use agileplus_domain::domain::metric::Metric;
use agileplus_domain::domain::module::{Module, ModuleFeatureTag, ModuleWithFeatures};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::sync_mapping::SyncMapping;
use agileplus_domain::domain::work_package::{WorkPackage, WpDependency, WpState};
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::ContentStoragePort;
use agileplus_domain::ports::observability::{LogEntry, ObservabilityPort, SpanContext};
use agileplus_domain::ports::storage::StoragePort;
use agileplus_domain::ports::vcs::{
    ConflictInfo, FeatureArtifacts, MergeResult, VcsPort, WorktreeInfo,
};
use axum::http::StatusCode;
use axum_test::TestServer;
use chrono::Utc;

// ── Mock Storage ─────────────────────────────────────────────────────────────

#[derive(Clone)]
struct MockStorage {
    features: Arc<std::sync::Mutex<Vec<Feature>>>,
    work_packages: Arc<std::sync::Mutex<Vec<WorkPackage>>>,
    governance: Arc<std::sync::Mutex<Vec<GovernanceContract>>>,
    projects: Arc<std::sync::Mutex<Vec<Project>>>
    audit: Arc<std::sync::Mutex<Vec<AuditEntry>>>,
}

impl Default for MockStorage {
    fn default() -> Self {
        Self {
            features: Arc::new(std::sync::Mutex::new(Vec::new())),
            work_packages: Arc::new(std::sync::Mutex::new(Vec::new())),
            governance: Arc::new(std::sync::Mutex::new(Vec::new())),
            audit: Arc::new(std::sync::Mutex::new(Vec::new())),
            projects: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
}

impl MockStorage {
    fn with_test_data() -> Self {
        let s = MockStorage::default();
        let now = Utc::now();
        s.projects.lock().unwrap().push(Project {
            id: 1,
            slug: "test-project".to_string(),
            name: "Test Project".to_string(),
            description: "A test project".to_string(),
            created_at: now,
            updated_at: now,
        });
        let now = Utc::now();
        s.features.lock().unwrap().push(Feature {
            id: 1,
            slug: "test-feature".to_string(),
            friendly_name: "Test Feature".to_string(),
            state: FeatureState::Implementing,
            spec_hash: [0u8; 32],
            target_branch: "main".to_string(),
            plane_issue_id: None,
            plane_state_id: None,
            labels: vec![],
            module_id: None,
            project_id: None,
            created_at_commit: None,
            last_modified_commit: None,
            created_at: now,
            updated_at: now,
        });
        s.work_packages.lock().unwrap().push(WorkPackage {
            id: 1,
            feature_id: 1,
            title: "WP01".to_string(),
            state: WpState::Done,
            sequence: 1,
            file_scope: vec![],
            acceptance_criteria: "All tests pass".to_string(),
            agent_id: None,
            pr_url: Some("https://github.com/org/repo/pull/1".to_string()),
            pr_state: None,
            worktree_path: None,
            plane_sub_issue_id: None,
            base_commit: None,
            head_commit: None,
            created_at: now,
            updated_at: now,
        });

        // Build a valid 2-entry audit chain.
        let genesis = AuditEntry {
            id: 1,
            feature_id: 1,
            wp_id: None,
            timestamp: now,
            actor: "system".to_string(),
            transition: "created".to_string(),
            evidence_refs: vec![],
            prev_hash: [0u8; 32],
            hash: [0u8; 32], // will be fixed below
            event_id: None,
            archived_to: None,
        };
        let genesis_hash = hash_entry(&genesis);
        let genesis = AuditEntry {
            hash: genesis_hash,
            ..genesis
        };
        let second = AuditEntry {
            id: 2,
            feature_id: 1,
            wp_id: Some(1),
            timestamp: now,
            actor: "agent".to_string(),
            transition: "specified".to_string(),
            evidence_refs: vec![],
            prev_hash: genesis_hash,
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        };
        let second_hash = hash_entry(&second);
        let second = AuditEntry {
            hash: second_hash,
            ..second
        };
        s.audit.lock().unwrap().extend([genesis, second]);

        s.governance.lock().unwrap().push(GovernanceContract {
            id: 1,
            feature_id: 1,
            version: 1,
            rules: vec![],
            bound_at: now,
        });
        s
    }
}

impl StoragePort for MockStorage {
    async fn create_feature(&self, _f: &Feature) -> Result<i64, DomainError> {
        let id = (self.features.lock().unwrap().len() + 1) as i64;
        Ok(id)
    }

    async fn get_feature_by_slug(&self, slug: &str) -> Result<Option<Feature>, DomainError> {
        let found = self
            .features
            .lock()
            .unwrap()
            .iter()
            .find(|f| f.slug == slug)
            .cloned();
        Ok(found)
    }

    async fn get_feature_by_id(&self, id: i64) -> Result<Option<Feature>, DomainError> {
        let found = self
            .features
            .lock()
            .unwrap()
            .iter()
            .find(|f| f.id == id)
            .cloned();
        Ok(found)
    }

    async fn update_feature_state(
        &self,
        _id: i64,
        _state: FeatureState,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn list_features_by_state(
        &self,
        state: FeatureState,
    ) -> Result<Vec<Feature>, DomainError> {
        let features: Vec<Feature> = self
            .features
            .lock()
            .unwrap()
            .iter()
            .filter(|f| f.state == state)
            .cloned()
            .collect();
        Ok(features)
    }

    async fn list_all_features(&self) -> Result<Vec<Feature>, DomainError> {
        let features = self.features.lock().unwrap().clone();
        Ok(features)
    }

    async fn create_work_package(&self, _wp: &WorkPackage) -> Result<i64, DomainError> {
        Ok(99)
    }

    async fn get_work_package(&self, id: i64) -> Result<Option<WorkPackage>, DomainError> {
        let found = self
            .work_packages
            .lock()
            .unwrap()
            .iter()
            .find(|w| w.id == id)
            .cloned();
        Ok(found)
    }

    async fn update_wp_state(&self, _id: i64, _state: WpState) -> Result<(), DomainError> {
        Ok(())
    }

    async fn list_wps_by_feature(&self, feature_id: i64) -> Result<Vec<WorkPackage>, DomainError> {
        let wps: Vec<WorkPackage> = self
            .work_packages
            .lock()
            .unwrap()
            .iter()
            .filter(|w| w.feature_id == feature_id)
            .cloned()
            .collect();
        Ok(wps)
    }

    async fn add_wp_dependency(&self, _dep: &WpDependency) -> Result<(), DomainError> {
        Ok(())
    }

    async fn get_wp_dependencies(&self, _wp_id: i64) -> Result<Vec<WpDependency>, DomainError> {
        Ok(vec![])
    }

    async fn get_ready_wps(&self, feature_id: i64) -> Result<Vec<WorkPackage>, DomainError> {
        let wps: Vec<WorkPackage> = self
            .work_packages
            .lock()
            .unwrap()
            .iter()
            .filter(|w| w.feature_id == feature_id)
            .cloned()
            .collect();
        Ok(wps)
    }

    async fn append_audit_entry(&self, entry: &AuditEntry) -> Result<i64, DomainError> {
        let id = (self.audit.lock().unwrap().len() + 1) as i64;
        let _ = entry;
        Ok(id)
    }

    async fn get_audit_trail(&self, feature_id: i64) -> Result<Vec<AuditEntry>, DomainError> {
        let trail: Vec<AuditEntry> = self
            .audit
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.feature_id == feature_id)
            .cloned()
            .collect();
        Ok(trail)
    }

    async fn get_latest_audit_entry(
        &self,
        feature_id: i64,
    ) -> Result<Option<AuditEntry>, DomainError> {
        let entry = self
            .audit
            .lock()
            .unwrap()
            .iter()
            .rfind(|e| e.feature_id == feature_id)
            .cloned();
        Ok(entry)
    }

    async fn create_evidence(&self, _e: &Evidence) -> Result<i64, DomainError> {
        Ok(1)
    }

    async fn get_evidence_by_wp(&self, _wp_id: i64) -> Result<Vec<Evidence>, DomainError> {
        Ok(vec![])
    }

    async fn get_evidence_by_fr(&self, _fr_id: &str) -> Result<Vec<Evidence>, DomainError> {
        Ok(vec![])
    }

    async fn create_policy_rule(&self, _r: &PolicyRule) -> Result<i64, DomainError> {
        Ok(1)
    }

    async fn list_active_policies(&self) -> Result<Vec<PolicyRule>, DomainError> {
        Ok(vec![])
    }

    async fn record_metric(&self, _m: &Metric) -> Result<i64, DomainError> {
        Ok(1)
    }

    async fn get_metrics_by_feature(&self, _feature_id: i64) -> Result<Vec<Metric>, DomainError> {
        Ok(vec![])
    }

    async fn create_governance_contract(
        &self,
        _c: &GovernanceContract,
    ) -> Result<i64, DomainError> {
        Ok(1)
    }

    async fn get_governance_contract(
        &self,
        feature_id: i64,
        version: i32,
    ) -> Result<Option<GovernanceContract>, DomainError> {
        let found = self
            .governance
            .lock()
            .unwrap()
            .iter()
            .find(|c| c.feature_id == feature_id && c.version == version)
            .cloned();
        Ok(found)
    }

    async fn get_latest_governance_contract(
        &self,
        feature_id: i64,
    ) -> Result<Option<GovernanceContract>, DomainError> {
        let found = self
            .governance
            .lock()
            .unwrap()
            .iter()
            .filter(|c| c.feature_id == feature_id)
            .max_by_key(|c| c.version)
            .cloned();
        Ok(found)
    }

    // -- Module stubs (WP02/WP04) --

    async fn create_module(&self, _module: &Module) -> Result<i64, DomainError> {
        Ok(1)
    }

    async fn get_module(&self, _id: i64) -> Result<Option<Module>, DomainError> {
        Ok(None)
    }

    async fn get_module_by_slug(&self, _slug: &str) -> Result<Option<Module>, DomainError> {
        Ok(None)
    }

    async fn update_module(
        &self,
        _id: i64,
        _friendly_name: &str,
        _description: Option<&str>,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn delete_module(&self, _id: i64) -> Result<(), DomainError> {
        Ok(())
    }

    async fn list_root_modules(&self) -> Result<Vec<Module>, DomainError> {
        Ok(vec![])
    }

    async fn list_child_modules(&self, _parent_id: i64) -> Result<Vec<Module>, DomainError> {
        Ok(vec![])
    }

    async fn get_module_with_features(
        &self,
        _id: i64,
    ) -> Result<Option<ModuleWithFeatures>, DomainError> {
        Ok(None)
    }

    // -- Cycle stubs (WP02/WP04) --

    async fn create_cycle(&self, _cycle: &Cycle) -> Result<i64, DomainError> {
        Ok(1)
    }

    async fn get_cycle(&self, _id: i64) -> Result<Option<Cycle>, DomainError> {
        Ok(None)
    }

    async fn update_cycle_state(&self, _id: i64, _state: CycleState) -> Result<(), DomainError> {
        Ok(())
    }

    async fn list_cycles_by_state(&self, _state: CycleState) -> Result<Vec<Cycle>, DomainError> {
        Ok(vec![])
    }

    async fn list_cycles_by_module(&self, _module_id: i64) -> Result<Vec<Cycle>, DomainError> {
        Ok(vec![])
    }

    async fn list_all_cycles(&self) -> Result<Vec<Cycle>, DomainError> {
        Ok(vec![])
    }

    async fn get_cycle_with_features(
        &self,
        _id: i64,
    ) -> Result<Option<CycleWithFeatures>, DomainError> {
        Ok(None)
    }

    // -- Join table stubs (WP02/WP04) --

    async fn tag_feature_to_module(&self, _tag: &ModuleFeatureTag) -> Result<(), DomainError> {
        Ok(())
    }

    async fn untag_feature_from_module(
        &self,
        _module_id: i64,
        _feature_id: i64,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn add_feature_to_cycle(&self, _entry: &CycleFeature) -> Result<(), DomainError> {
        Ok(())
    }

    async fn remove_feature_from_cycle(
        &self,
        _cycle_id: i64,
        _feature_id: i64,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    // -- Sync Mapping stubs (WP06) --

    async fn get_sync_mapping(
        &self,
        _entity_type: &str,
        _entity_id: i64,
    ) -> Result<Option<SyncMapping>, DomainError> {
        Ok(None)
    }

    async fn upsert_sync_mapping(&self, _mapping: &SyncMapping) -> Result<(), DomainError> {
        Ok(())
    }

    async fn get_sync_mapping_by_plane_id(
        &self,
        _entity_type: &str,
        _plane_issue_id: &str,
    ) -> Result<Option<SyncMapping>, DomainError> {
        Ok(None)
    }

    async fn delete_sync_mapping(
        &self,
        _entity_type: &str,
        _entity_id: i64,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    // -- Project CRUD (StoragePort requirement) --

    async fn create_project(&self, _project: &Project) -> Result<i64, DomainError> {
        let id = (self.projects.lock().unwrap().len() + 1) as i64;
        Ok(id)
    }

    async fn get_project_by_slug(&self, slug: &str) -> Result<Option<Project>, DomainError> {
        let found = self.projects.lock().unwrap().iter().find(|p| p.slug == slug).cloned();
        Ok(found)
    }
}

// ── ContentStoragePort for MockStorage ───────────────────────────────────────

impl ContentStoragePort for MockStorage {
    async fn create_feature(
        &self,
        f: &agileplus_domain::domain::feature::Feature,
    ) -> Result<i64, DomainError> {
        let id = (self.features.lock().unwrap().len() + 1) as i64;
        let _ = f;
        Ok(id)
    }

    async fn get_feature_by_slug(
        &self,
        slug: &str,
    ) -> Result<Option<agileplus_domain::domain::feature::Feature>, DomainError> {
        let feats = self.features.lock().unwrap();
        let found = feats.iter().find(|f| f.slug == slug).cloned();
        Ok(found)
    }

    async fn get_feature_by_id(
        &self,
        id: i64,
    ) -> Result<Option<agileplus_domain::domain::feature::Feature>, DomainError> {
        let feats = self.features.lock().unwrap();
        let found = feats.iter().find(|f| f.id == id).cloned();
        Ok(found)
    }

    async fn update_feature_state(
        &self,
        _id: i64,
        _state: agileplus_domain::domain::state_machine::FeatureState,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn update_feature(
        &self,
        _feature: &agileplus_domain::domain::feature::Feature,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn list_features_by_state(
        &self,
        state: agileplus_domain::domain::state_machine::FeatureState,
    ) -> Result<Vec<agileplus_domain::domain::feature::Feature>, DomainError> {
        let feats: Vec<_> = self
            .features
            .lock()
            .unwrap()
            .iter()
            .filter(|f| f.state == state)
            .cloned()
            .collect();
        Ok(feats)
    }

    async fn list_all_features(
        &self,
    ) -> Result<Vec<agileplus_domain::domain::feature::Feature>, DomainError> {
        let feats: Vec<_> = self.features.lock().unwrap().clone();
        Ok(feats)
    }

    async fn create_backlog_item(&self, _item: &BacklogItem) -> Result<i64, DomainError> {
        Ok(1)
    }

    async fn get_backlog_item(&self, _id: i64) -> Result<Option<BacklogItem>, DomainError> {
        Ok(None)
    }

    async fn list_backlog_items(
        &self,
        _filters: &BacklogFilters,
    ) -> Result<Vec<BacklogItem>, DomainError> {
        Ok(vec![])
    }

    async fn update_backlog_status(
        &self,
        _id: i64,
        _status: BacklogStatus,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn update_backlog_priority(
        &self,
        _id: i64,
        _priority: BacklogPriority,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn pop_next_backlog_item(&self) -> Result<Option<BacklogItem>, DomainError> {
        Ok(None)
    }

    async fn create_work_package(
        &self,
        _wp: &agileplus_domain::domain::work_package::WorkPackage,
    ) -> Result<i64, DomainError> {
        Ok(1)
    }

    async fn get_work_package(
        &self,
        _id: i64,
    ) -> Result<Option<agileplus_domain::domain::work_package::WorkPackage>, DomainError> {
        Ok(None)
    }

    async fn update_wp_state(
        &self,
        _id: i64,
        _state: agileplus_domain::domain::work_package::WpState,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn update_work_package(
        &self,
        _wp: &agileplus_domain::domain::work_package::WorkPackage,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn list_wps_by_feature(
        &self,
        _feature_id: i64,
    ) -> Result<Vec<agileplus_domain::domain::work_package::WorkPackage>, DomainError> {
        Ok(vec![])
    }

    async fn add_wp_dependency(
        &self,
        _dep: &agileplus_domain::domain::work_package::WpDependency,
    ) -> Result<(), DomainError> {
        Ok(())
    }

    async fn get_wp_dependencies(
        &self,
        _wp_id: i64,
    ) -> Result<Vec<agileplus_domain::domain::work_package::WpDependency>, DomainError> {
        Ok(vec![])
    }

    async fn get_ready_wps(
        &self,
        _feature_id: i64,
    ) -> Result<Vec<agileplus_domain::domain::work_package::WorkPackage>, DomainError> {
        Ok(vec![])
    }
}

// ── Mock VCS ─────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct MockVcs;

impl VcsPort for MockVcs {
    async fn create_worktree(&self, _fs: &str, _wp: &str) -> Result<PathBuf, DomainError> {
        Ok(PathBuf::from("/tmp/worktree"))
    }
    async fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>, DomainError> {
        Ok(vec![])
    }
    async fn cleanup_worktree(&self, _p: &Path) -> Result<(), DomainError> {
        Ok(())
    }
    async fn create_branch(&self, _b: &str, _base: &str) -> Result<(), DomainError> {
        Ok(())
    }
    async fn checkout_branch(&self, _b: &str) -> Result<(), DomainError> {
        Ok(())
    }
    async fn merge_to_target(&self, _s: &str, _t: &str) -> Result<MergeResult, DomainError> {
        Ok(MergeResult {
            success: true,
            conflicts: vec![],
            merged_commit: None,
        })
    }
    async fn detect_conflicts(&self, _s: &str, _t: &str) -> Result<Vec<ConflictInfo>, DomainError> {
        Ok(vec![])
    }
    async fn read_artifact(&self, _fs: &str, _p: &str) -> Result<String, DomainError> {
        Ok(String::new())
    }
    async fn write_artifact(&self, _fs: &str, _p: &str, _c: &str) -> Result<(), DomainError> {
        Ok(())
    }
    async fn artifact_exists(&self, _fs: &str, _p: &str) -> Result<bool, DomainError> {
        Ok(false)
    }
    async fn scan_feature_artifacts(&self, _fs: &str) -> Result<FeatureArtifacts, DomainError> {
        Ok(FeatureArtifacts {
            meta_json: None,
            audit_chain: None,
            evidence_paths: vec![],
        })
    }
}

// ── Mock Observability ────────────────────────────────────────────────────────

#[derive(Clone)]
struct MockObs;

impl ObservabilityPort for MockObs {
    fn start_span(&self, _n: &str, _p: Option<&SpanContext>) -> SpanContext {
        SpanContext {
            trace_id: "t".to_string(),
            span_id: "s".to_string(),
            parent_span_id: None,
        }
    }
    fn end_span(&self, _c: &SpanContext) {}
    fn add_span_event(&self, _c: &SpanContext, _n: &str, _a: &[(&str, &str)]) {}
    fn set_span_error(&self, _c: &SpanContext, _e: &str) {}
    fn record_counter(&self, _n: &str, _v: u64, _l: &[(&str, &str)]) {}
    fn record_histogram(&self, _n: &str, _v: f64, _l: &[(&str, &str)]) {}
    fn record_gauge(&self, _n: &str, _v: f64, _l: &[(&str, &str)]) {}
    fn log(&self, _e: &LogEntry) {}
    fn log_info(&self, _m: &str) {}
    fn log_warn(&self, _m: &str) {}
    fn log_error(&self, _m: &str) {}
}

// ── Test harness ─────────────────────────────────────────────────────────────

const TEST_API_KEY: &str = "test-api-key-12345";

async fn setup_test_server() -> TestServer {
    let storage = Arc::new(MockStorage::with_test_data());
    let vcs = Arc::new(MockVcs);
    let telemetry = Arc::new(MockObs);
    let config = Arc::new(AppConfig::default());

    let creds_inner = InMemoryCredentialStore::new();
    creds_inner
        .set("agileplus", cred_keys::API_KEYS, TEST_API_KEY)
        .unwrap();
    let creds: Arc<dyn agileplus_domain::credentials::CredentialStore> = Arc::new(creds_inner);

    let state = AppState::new(storage, vcs, telemetry, config, creds);
    let app = create_router(state);
    TestServer::new(app)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn health_no_auth_required() {
    let server = setup_test_server().await;
    let resp = server.get("/health").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    // Health endpoint returns "healthy" or "degraded" (not "ok") as of WP11-T070.
    let status = body["status"].as_str().expect("status field present");
    assert!(
        status == "healthy" || status == "degraded",
        "unexpected health status: {status}"
    );
    // Timestamp and services must be present.
    assert!(body["timestamp"].is_string());
    assert!(body["services"].is_object());
}

#[tokio::test]
async fn info_no_auth_required() {
    let server = setup_test_server().await;
    let resp = server.get("/info").await;
    resp.assert_status_ok();
}

#[tokio::test]
async fn list_features_requires_auth() {
    let server = setup_test_server().await;
    let resp = server.get("/api/v1/features").await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_features_with_valid_key() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    let arr = body.as_array().unwrap();
    assert!(!arr.is_empty());
    assert_eq!(arr[0]["slug"], "test-feature");
}

#[tokio::test]
async fn list_features_invalid_key_returns_401() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features")
        .add_header("X-API-Key", "wrong-key")
        .await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_feature_found() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features/test-feature")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["slug"], "test-feature");
    assert_eq!(body["name"], "Test Feature");
}

#[tokio::test]
async fn get_feature_not_found() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features/nonexistent")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_work_package_found() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/work-packages/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["id"], 1);
    assert_eq!(body["title"], "WP01");
}

#[tokio::test]
async fn get_work_package_not_found() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/work-packages/999")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_audit_trail() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features/test-feature/audit")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["actor"], "system");
}

#[tokio::test]
async fn verify_audit_chain_valid() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/features/test-feature/audit/verify")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["chain_valid"], true);
    assert_eq!(body["entries_verified"], 2);
}

#[tokio::test]
async fn get_governance() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features/test-feature/governance")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["version"], 1);
    assert_eq!(body["feature_id"], 1);
}

#[tokio::test]
async fn trigger_validate() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/features/test-feature/validate")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["feature_slug"], "test-feature");
    assert_eq!(body["compliant"], true); // no rules → all satisfied
}

#[tokio::test]
async fn response_content_type_is_json() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(
        ct.contains("application/json"),
        "Expected application/json, got: {ct}"
    );
}
