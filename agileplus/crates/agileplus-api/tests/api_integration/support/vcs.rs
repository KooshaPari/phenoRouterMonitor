use std::future::Future;
use std::path::{Path, PathBuf};

use agileplus_domain::error::DomainError;
use agileplus_domain::ports::vcs::{
    BranchInfo, ConflictInfo, FeatureArtifacts, MergeResult, VcsPort, WorktreeInfo,
};

#[derive(Clone)]
pub(crate) struct MockVcs;

impl VcsPort for MockVcs {
    fn create_worktree(
        &self,
        _fs: &str,
        _wp: &str,
    ) -> impl Future<Output = Result<PathBuf, DomainError>> + Send {
        async move { Ok(PathBuf::from("/tmp/worktree")) }
    }

    fn list_worktrees(
        &self,
    ) -> impl Future<Output = Result<Vec<WorktreeInfo>, DomainError>> + Send {
        async move { Ok(vec![]) }
    }

    fn cleanup_worktree(&self, _p: &Path) -> impl Future<Output = Result<(), DomainError>> + Send {
        async move { Ok(()) }
    }

    fn create_branch(
        &self,
        _b: &str,
        _base: &str,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        async move { Ok(()) }
    }

    fn list_branches(
        &self,
        _pattern: Option<&str>,
        _remote: bool,
    ) -> impl Future<Output = Result<Vec<BranchInfo>, DomainError>> + Send {
        async move { Ok(vec![]) }
    }

    fn delete_branch(
        &self,
        _branch_name: &str,
        _force: bool,
        _remote: Option<&str>,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        async move { Ok(()) }
    }

    fn checkout_branch(&self, _b: &str) -> impl Future<Output = Result<(), DomainError>> + Send {
        async move { Ok(()) }
    }

    fn merge_to_target(
        &self,
        _s: &str,
        _t: &str,
    ) -> impl Future<Output = Result<MergeResult, DomainError>> + Send {
        async move {
            Ok(MergeResult {
                success: true,
                conflicts: vec![],
                merged_commit: None,
            })
        }
    }

    fn detect_conflicts(
        &self,
        _s: &str,
        _t: &str,
    ) -> impl Future<Output = Result<Vec<ConflictInfo>, DomainError>> + Send {
        async move { Ok(vec![]) }
    }

    fn read_artifact(
        &self,
        _fs: &str,
        _p: &str,
    ) -> impl Future<Output = Result<String, DomainError>> + Send {
        async move { Ok(String::new()) }
    }

    fn write_artifact(
        &self,
        _fs: &str,
        _p: &str,
        _c: &str,
    ) -> impl Future<Output = Result<(), DomainError>> + Send {
        async move { Ok(()) }
    }

    fn artifact_exists(
        &self,
        _fs: &str,
        _p: &str,
    ) -> impl Future<Output = Result<bool, DomainError>> + Send {
        async move { Ok(false) }
    }

    fn scan_feature_artifacts(
        &self,
        _fs: &str,
    ) -> impl Future<Output = Result<FeatureArtifacts, DomainError>> + Send {
        async move {
            Ok(FeatureArtifacts {
                meta_json: None,
                audit_chain: None,
                evidence_paths: vec![],
            })
        }
    }
}
