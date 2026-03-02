//! Integration tests for the AgilePlus HTTP API.
//!
//! These tests spin up a real axum test server backed by in-memory mock
//! implementations of all ports. No external dependencies are required.
//!
//! Run with: `cargo test -p agileplus-api`
//!
//! Traceability: WP15-T090

#![allow(dead_code)]

use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use agileplus_api::{create_router, AppState};
use agileplus_domain::credentials::CredentialStore;
use agileplus_domain::config::AppConfig;
use agileplus_domain::credentials::InMemoryCredentialStore;
use agileplus_domain::credentials::keys as cred_keys;
use agileplus_domain::domain::audit::{AuditEntry, hash_entry};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::governance::{Evidence, GovernanceContract, PolicyRule};
use agileplus_domain::domain::metric::Metric;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{WorkPackage, WpDependency, WpState};
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::observability::{
    LogEntry, ObservabilityPort, SpanContext,
};
use agileplus_domain::ports::storage::StoragePort;
use agileplus_domain::ports::vcs::{
    ConflictInfo, FeatureArtifacts, MergeResult, VcsPort, WorktreeInfo,
};
use axum::http::StatusCode;
use axum_test::TestServer;
use chrono::Utc;

// ── Mock Storage ─────────────────────────────────────────────────────────────

#[derive(Default, Clone)]
struct MockStorage {
    features: Arc<std::sync::Mutex<Vec<Feature>>>,
    work_packages: Arc<std::sync::Mutex<Vec<WorkPackage>>>,
    governance: Arc<std::sync::Mutex<Vec<GovernanceContract>>>,
    audit: Arc<std::sync::Mutex<Vec<AuditEntry>>>,
}

impl MockStorage {
    fn with_test_data() -> Self {
        let s = MockStorage::default();
        let now = Utc::now();
        s.features.lock().unwrap().push(Feature {
            id: 1,
            slug: "test-feature".to_string(),
            friendly_name: "Test Feature".to_string(),
            state: FeatureState::Implementing,
            spec_hash: [0u8; 32],
            target_branch: "main".to_string(),
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
        };
        let genesis_hash = hash_entry(&genesis);
        let genesis = AuditEntry { hash: genesis_hash, ..genesis };
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
        };
        let second_hash = hash_entry(&second);
        let second = AuditEntry { hash: second_hash, ..second };
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
    fn create_feature(&self, f: &Feature) -> impl Future<Output = Result<i64, DomainError>> + Send {
        let id = (self.features.lock().unwrap().len() + 1) as i64;
        async move { Ok(id) }
    }

    fn get_feature_by_slug(&self, slug: &str) -> impl Future<Output = Result<Option<Feature>, DomainError>> + Send {
        let found = self.features.lock().unwrap().iter().find(|f| f.slug == slug).cloned();
        async move { Ok(found) }
    }

    fn get_feature_by_id(&self, id: i64) -> impl Future<Output = Result<Option<Feature>, DomainError>> + Send {
        let found = self.features.lock().unwrap().iter().find(|f| f.id == id).cloned();
        async move { Ok(found) }
    }

    fn update_feature_state(&self, _id: i64, _state: FeatureState) -> impl Future<Output = Result<(), DomainError>> + Send {
        async move { Ok(()) }
    }

    fn list_features_by_state(&self, state: FeatureState) -> impl Future<Output = Result<Vec<Feature>, DomainError>> + Send {
        let features: Vec<Feature> = self.features.lock().unwrap().iter().filter(|f| f.state == state).cloned().collect();
        async move { Ok(features) }
    }

    fn list_all_features(&self) -> impl Future<Output = Result<Vec<Feature>, DomainError>> + Send {
        let features = self.features.lock().unwrap().clone();
        async move { Ok(features) }
    }

    fn create_work_package(&self, _wp: &WorkPackage) -> impl Future<Output = Result<i64, DomainError>> + Send {
        async move { Ok(99) }
    }

    fn get_work_package(&self, id: i64) -> impl Future<Output = Result<Option<WorkPackage>, DomainError>> + Send {
        let found = self.work_packages.lock().unwrap().iter().find(|w| w.id == id).cloned();
        async move { Ok(found) }
    }

    fn update_wp_state(&self, _id: i64, _state: WpState) -> impl Future<Output = Result<(), DomainError>> + Send {
        async move { Ok(()) }
    }

    fn list_wps_by_feature(&self, feature_id: i64) -> impl Future<Output = Result<Vec<WorkPackage>, DomainError>> + Send {
        let wps: Vec<WorkPackage> = self.work_packages.lock().unwrap().iter().filter(|w| w.feature_id == feature_id).cloned().collect();
        async move { Ok(wps) }
    }

    fn add_wp_dependency(&self, _dep: &WpDependency) -> impl Future<Output = Result<(), DomainError>> + Send {
        async move { Ok(()) }
    }

    fn get_wp_dependencies(&self, _wp_id: i64) -> impl Future<Output = Result<Vec<WpDependency>, DomainError>> + Send {
        async move { Ok(vec![]) }
    }

    fn get_ready_wps(&self, feature_id: i64) -> impl Future<Output = Result<Vec<WorkPackage>, DomainError>> + Send {
        let wps: Vec<WorkPackage> = self.work_packages.lock().unwrap().iter().filter(|w| w.feature_id == feature_id).cloned().collect();
        async move { Ok(wps) }
    }

    fn append_audit_entry(&self, entry: &AuditEntry) -> impl Future<Output = Result<i64, DomainError>> + Send {
        let id = (self.audit.lock().unwrap().len() + 1) as i64;
        let _ = entry;
        async move { Ok(id) }
    }

    fn get_audit_trail(&self, feature_id: i64) -> impl Future<Output = Result<Vec<AuditEntry>, DomainError>> + Send {
        let trail: Vec<AuditEntry> = self.audit.lock().unwrap().iter().filter(|e| e.feature_id == feature_id).cloned().collect();
        async move { Ok(trail) }
    }

    fn get_latest_audit_entry(&self, feature_id: i64) -> impl Future<Output = Result<Option<AuditEntry>, DomainError>> + Send {
        let entry = self.audit.lock().unwrap().iter().filter(|e| e.feature_id == feature_id).last().cloned();
        async move { Ok(entry) }
    }

    fn create_evidence(&self, _e: &Evidence) -> impl Future<Output = Result<i64, DomainError>> + Send {
        async move { Ok(1) }
    }

    fn get_evidence_by_wp(&self, _wp_id: i64) -> impl Future<Output = Result<Vec<Evidence>, DomainError>> + Send {
        async move { Ok(vec![]) }
    }

    fn get_evidence_by_fr(&self, _fr_id: &str) -> impl Future<Output = Result<Vec<Evidence>, DomainError>> + Send {
        async move { Ok(vec![]) }
    }

    fn create_policy_rule(&self, _r: &PolicyRule) -> impl Future<Output = Result<i64, DomainError>> + Send {
        async move { Ok(1) }
    }

    fn list_active_policies(&self) -> impl Future<Output = Result<Vec<PolicyRule>, DomainError>> + Send {
        async move { Ok(vec![]) }
    }

    fn record_metric(&self, _m: &Metric) -> impl Future<Output = Result<i64, DomainError>> + Send {
        async move { Ok(1) }
    }

    fn get_metrics_by_feature(&self, _feature_id: i64) -> impl Future<Output = Result<Vec<Metric>, DomainError>> + Send {
        async move { Ok(vec![]) }
    }

    fn create_governance_contract(&self, _c: &GovernanceContract) -> impl Future<Output = Result<i64, DomainError>> + Send {
        async move { Ok(1) }
    }

    fn get_governance_contract(&self, feature_id: i64, version: i32) -> impl Future<Output = Result<Option<GovernanceContract>, DomainError>> + Send {
        let found = self.governance.lock().unwrap().iter().find(|c| c.feature_id == feature_id && c.version == version).cloned();
        async move { Ok(found) }
    }

    fn get_latest_governance_contract(&self, feature_id: i64) -> impl Future<Output = Result<Option<GovernanceContract>, DomainError>> + Send {
        let found = self.governance.lock().unwrap().iter().filter(|c| c.feature_id == feature_id).max_by_key(|c| c.version).cloned();
        async move { Ok(found) }
    }
}

// ── Mock VCS ─────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct MockVcs;

impl VcsPort for MockVcs {
    fn create_worktree(&self, _fs: &str, _wp: &str) -> impl Future<Output = Result<PathBuf, DomainError>> + Send {
        async move { Ok(PathBuf::from("/tmp/worktree")) }
    }
    fn list_worktrees(&self) -> impl Future<Output = Result<Vec<WorktreeInfo>, DomainError>> + Send {
        async move { Ok(vec![]) }
    }
    fn cleanup_worktree(&self, _p: &Path) -> impl Future<Output = Result<(), DomainError>> + Send {
        async move { Ok(()) }
    }
    fn create_branch(&self, _b: &str, _base: &str) -> impl Future<Output = Result<(), DomainError>> + Send {
        async move { Ok(()) }
    }
    fn checkout_branch(&self, _b: &str) -> impl Future<Output = Result<(), DomainError>> + Send {
        async move { Ok(()) }
    }
    fn merge_to_target(&self, _s: &str, _t: &str) -> impl Future<Output = Result<MergeResult, DomainError>> + Send {
        async move { Ok(MergeResult { success: true, conflicts: vec![], merged_commit: None }) }
    }
    fn detect_conflicts(&self, _s: &str, _t: &str) -> impl Future<Output = Result<Vec<ConflictInfo>, DomainError>> + Send {
        async move { Ok(vec![]) }
    }
    fn read_artifact(&self, _fs: &str, _p: &str) -> impl Future<Output = Result<String, DomainError>> + Send {
        async move { Ok(String::new()) }
    }
    fn write_artifact(&self, _fs: &str, _p: &str, _c: &str) -> impl Future<Output = Result<(), DomainError>> + Send {
        async move { Ok(()) }
    }
    fn artifact_exists(&self, _fs: &str, _p: &str) -> impl Future<Output = Result<bool, DomainError>> + Send {
        async move { Ok(false) }
    }
    fn scan_feature_artifacts(&self, _fs: &str) -> impl Future<Output = Result<FeatureArtifacts, DomainError>> + Send {
        async move { Ok(FeatureArtifacts { meta_json: None, audit_chain: None, evidence_paths: vec![] }) }
    }
}

// ── Mock Observability ────────────────────────────────────────────────────────

#[derive(Clone)]
struct MockObs;

impl ObservabilityPort for MockObs {
    fn start_span(&self, _n: &str, _p: Option<&SpanContext>) -> SpanContext {
        SpanContext { trace_id: "t".to_string(), span_id: "s".to_string(), parent_span_id: None }
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
        .await
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
    assert_eq!(body["status"], "ok");
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
    let ct = resp.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(ct.contains("application/json"), "Expected application/json, got: {ct}");
}
