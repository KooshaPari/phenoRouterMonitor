use std::collections::HashMap;
use std::path::{Path, PathBuf};

use agileplus_domain::{
    domain::audit::hash_entry,
    error::DomainError,
    ports::{
        ContentStoragePort, StoragePort, VcsPort,
        vcs::{BranchInfo, ConflictInfo, FeatureArtifacts, MergeResult, WorktreeInfo},
    },
};

use super::*;
use crate::SqliteStorageAdapter;

/// A simple in-memory VcsPort mock.
struct MockVcs {
    artifacts: HashMap<String, String>,
}

impl MockVcs {
    fn new() -> Self {
        Self {
            artifacts: HashMap::new(),
        }
    }

    fn add(&mut self, slug: &str, path: &str, content: &str) {
        self.artifacts
            .insert(format!("{slug}/{path}"), content.to_string());
    }
}

impl VcsPort for MockVcs {
    async fn create_worktree(
        &self,
        _feature_slug: &str,
        _wp_id: &str,
    ) -> Result<PathBuf, DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>, DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn cleanup_worktree(&self, _worktree_path: &Path) -> Result<(), DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn create_branch(&self, _branch_name: &str, _base: &str) -> Result<(), DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn list_branches(
        &self,
        _pattern: Option<&str>,
        _remote: bool,
    ) -> Result<Vec<BranchInfo>, DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn delete_branch(
        &self,
        _branch_name: &str,
        _force: bool,
        _remote: Option<&str>,
    ) -> Result<(), DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn checkout_branch(&self, _branch_name: &str) -> Result<(), DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn merge_to_target(
        &self,
        _source: &str,
        _target: &str,
    ) -> Result<MergeResult, DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn detect_conflicts(
        &self,
        _source: &str,
        _target: &str,
    ) -> Result<Vec<ConflictInfo>, DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn read_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> Result<String, DomainError> {
        let key = format!("{feature_slug}/{relative_path}");
        self.artifacts
            .get(&key)
            .cloned()
            .ok_or_else(|| DomainError::NotFound(key))
    }

    async fn write_artifact(
        &self,
        _feature_slug: &str,
        _relative_path: &str,
        _content: &str,
    ) -> Result<(), DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn artifact_exists(
        &self,
        _feature_slug: &str,
        _relative_path: &str,
    ) -> Result<bool, DomainError> {
        Err(DomainError::NotImplemented)
    }

    async fn scan_feature_artifacts(
        &self,
        _feature_slug: &str,
    ) -> Result<FeatureArtifacts, DomainError> {
        Err(DomainError::NotImplemented)
    }
}

fn make_chain_jsonl(feature_id: i64) -> String {
    let now = chrono::Utc::now();

    let mut e1 = AuditEntry {
        id: 0,
        feature_id,
        wp_id: None,
        timestamp: now,
        actor: "system".into(),
        transition: "created".into(),
        evidence_refs: vec![],
        prev_hash: [0u8; 32],
        hash: [0u8; 32],
        event_id: None,
        archived_to: None,
    };
    e1.hash = hash_entry(&e1);

    let mut e2 = AuditEntry {
        id: 0,
        feature_id,
        wp_id: None,
        timestamp: now,
        actor: "agent".into(),
        transition: "created->specified".into(),
        evidence_refs: vec![],
        prev_hash: e1.hash,
        hash: [0u8; 32],
        event_id: None,
        archived_to: None,
    };
    e2.hash = hash_entry(&e2);

    let hex = |b: [u8; 32]| b.iter().map(|x| format!("{x:02x}")).collect::<String>();

    let line1 = serde_json::json!({
        "feature_id": 1i64,
        "wp_id": null,
        "timestamp": now.to_rfc3339(),
        "actor": "system",
        "transition": "created",
        "evidence_refs": [],
        "prev_hash": hex([0u8; 32]),
        "hash": hex(e1.hash),
    });
    let line2 = serde_json::json!({
        "feature_id": 1i64,
        "wp_id": null,
        "timestamp": now.to_rfc3339(),
        "actor": "agent",
        "transition": "created->specified",
        "evidence_refs": [],
        "prev_hash": hex(e1.hash),
        "hash": hex(e2.hash),
    });

    format!("{}\n{}\n", line1, line2)
}

#[tokio::test]
async fn rebuild_from_fixtures() {
    let now = chrono::Utc::now();
    let slug = "test-feature";

    let meta = serde_json::json!({
        "slug": slug,
        "friendly_name": "Test Feature",
        "state": "specified",
        "spec_hash": "a".repeat(64),
        "target_branch": "main",
        "created_at": now.to_rfc3339(),
        "updated_at": now.to_rfc3339(),
    });

    let mut mock = MockVcs::new();
    mock.add(slug, "meta.json", &meta.to_string());
    mock.add(slug, "audit/chain.jsonl", &make_chain_jsonl(1));

    let db = SqliteStorageAdapter::in_memory().unwrap();
    let report = db.rebuild_from_git(&mock, &[slug]).await.unwrap();

    assert_eq!(report.features_restored, 1);
    assert_eq!(report.audit_entries_restored, 2);

    let feat = StoragePort::get_feature_by_slug(&db, slug).await.unwrap().unwrap();
    assert_eq!(feat.slug, slug);
    assert_eq!(
        feat.state,
        agileplus_domain::domain::state_machine::FeatureState::Specified
    );
}

#[tokio::test]
async fn rebuild_skips_missing_meta() {
    let mock = MockVcs::new();
    let db = SqliteStorageAdapter::in_memory().unwrap();
    let report = db
        .rebuild_from_git(&mock, &["no-such-feature"])
        .await
        .unwrap();
    assert_eq!(report.features_restored, 0);
}
