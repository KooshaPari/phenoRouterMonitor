use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitGuardConfig {
    /// If true, block checkout away from allowed branches in canonical repos
    pub block_checkout: bool,
    /// If true, block rebase in canonical repos
    pub block_rebase: bool,
    /// If true, block force push
    pub block_force_push: bool,
    /// If true, block reset --hard
    pub block_hard_reset: bool,
    /// Branches allowed in canonical repo (usually just "main")
    pub allowed_branches: Vec<String>,
    /// Paths that are canonical repos (not worktrees)
    pub canonical_paths: Vec<PathBuf>,
}

impl Default for GitGuardConfig {
    fn default() -> Self {
        Self {
            block_checkout: true,
            block_rebase: true,
            block_force_push: true,
            block_hard_reset: true,
            allowed_branches: vec!["main".to_string()],
            canonical_paths: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GuardViolation {
    pub operation: String,
    pub reason: String,
    pub repo_path: PathBuf,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct GitGuard {
    config: GitGuardConfig,
}

impl GitGuard {
    pub fn new(config: GitGuardConfig) -> Self {
        Self { config }
    }

    pub fn from_config_file(path: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let content = std::fs::read_to_string(path)?;
        let config: GitGuardConfig = toml::from_str(&content)?;
        Ok(Self::new(config))
    }

    /// Check if a repo path is canonical (not a worktree)
    pub fn is_canonical(&self, repo_path: &Path) -> bool {
        self.config.canonical_paths.iter().any(|p| p == repo_path)
    }

    /// Check if checkout to a branch is allowed
    pub fn check_checkout(
        &self,
        repo_path: &Path,
        target_branch: &str,
    ) -> Result<(), GuardViolation> {
        if !self.config.block_checkout || !self.is_canonical(repo_path) {
            return Ok(());
        }

        if !self
            .config
            .allowed_branches
            .contains(&target_branch.to_string())
        {
            return Err(GuardViolation {
                operation: "checkout".to_string(),
                reason: format!(
                    "Checkout to '{}' blocked in canonical repo. Allowed: {:?}. Use a worktree instead.",
                    target_branch, self.config.allowed_branches
                ),
                repo_path: repo_path.to_path_buf(),
                timestamp: chrono::Utc::now(),
            });
        }

        Ok(())
    }

    /// Check if rebase is allowed
    pub fn check_rebase(&self, repo_path: &Path) -> Result<(), GuardViolation> {
        if !self.config.block_rebase || !self.is_canonical(repo_path) {
            return Ok(());
        }

        Err(GuardViolation {
            operation: "rebase".to_string(),
            reason: "Rebase blocked in canonical repo. Use a worktree for rebasing.".to_string(),
            repo_path: repo_path.to_path_buf(),
            timestamp: chrono::Utc::now(),
        })
    }

    /// Check if force push is allowed
    pub fn check_force_push(&self, repo_path: &Path) -> Result<(), GuardViolation> {
        if !self.config.block_force_push || !self.is_canonical(repo_path) {
            return Ok(());
        }

        Err(GuardViolation {
            operation: "force_push".to_string(),
            reason: "Force push blocked in canonical repo.".to_string(),
            repo_path: repo_path.to_path_buf(),
            timestamp: chrono::Utc::now(),
        })
    }

    /// Check if hard reset is allowed
    pub fn check_hard_reset(&self, repo_path: &Path) -> Result<(), GuardViolation> {
        if !self.config.block_hard_reset || !self.is_canonical(repo_path) {
            return Ok(());
        }

        Err(GuardViolation {
            operation: "hard_reset".to_string(),
            reason: "Hard reset blocked in canonical repo.".to_string(),
            repo_path: repo_path.to_path_buf(),
            timestamp: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical_guard() -> GitGuard {
        let mut config = GitGuardConfig::default();
        config.canonical_paths.push(PathBuf::from("/repo/main"));
        GitGuard::new(config)
    }

    #[test]
    fn checkout_main_allowed_in_canonical() {
        let guard = canonical_guard();
        assert!(
            guard
                .check_checkout(Path::new("/repo/main"), "main")
                .is_ok()
        );
    }

    #[test]
    fn checkout_feature_blocked_in_canonical() {
        let guard = canonical_guard();
        assert!(
            guard
                .check_checkout(Path::new("/repo/main"), "feature/foo")
                .is_err()
        );
    }

    #[test]
    fn checkout_feature_allowed_in_worktree() {
        let guard = canonical_guard();
        // /repo/worktree is NOT in canonical_paths, so allowed
        assert!(
            guard
                .check_checkout(Path::new("/repo/worktree"), "feature/foo")
                .is_ok()
        );
    }

    #[test]
    fn rebase_blocked_in_canonical() {
        let guard = canonical_guard();
        assert!(guard.check_rebase(Path::new("/repo/main")).is_err());
    }

    #[test]
    fn rebase_allowed_in_worktree() {
        let guard = canonical_guard();
        assert!(guard.check_rebase(Path::new("/repo/worktree")).is_ok());
    }

    #[test]
    fn force_push_blocked_in_canonical() {
        let guard = canonical_guard();
        assert!(guard.check_force_push(Path::new("/repo/main")).is_err());
    }

    #[test]
    fn hard_reset_blocked_in_canonical() {
        let guard = canonical_guard();
        assert!(guard.check_hard_reset(Path::new("/repo/main")).is_err());
    }

    #[test]
    fn all_checks_pass_when_disabled() {
        let config = GitGuardConfig {
            block_checkout: false,
            block_rebase: false,
            block_force_push: false,
            block_hard_reset: false,
            allowed_branches: vec!["main".to_string()],
            canonical_paths: vec![PathBuf::from("/repo/main")],
        };
        let guard = GitGuard::new(config);
        assert!(
            guard
                .check_checkout(Path::new("/repo/main"), "feature/x")
                .is_ok()
        );
        assert!(guard.check_rebase(Path::new("/repo/main")).is_ok());
        assert!(guard.check_force_push(Path::new("/repo/main")).is_ok());
        assert!(guard.check_hard_reset(Path::new("/repo/main")).is_ok());
    }
}
