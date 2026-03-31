use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchPolicy {
    /// The main integration branch (usually "main")
    pub main_branch: String,
    /// Release branches pattern
    pub release_pattern: String,
    /// Feature branch prefix
    pub feature_prefix: String,
    /// WP branch format: feature/{slug}/WP{id}
    pub wp_branch_format: String,
    /// Whether direct commits to main are blocked
    pub block_direct_main_commits: bool,
}

impl Default for BranchPolicy {
    fn default() -> Self {
        Self {
            main_branch: "main".to_string(),
            release_pattern: "release/*".to_string(),
            feature_prefix: "feature/".to_string(),
            wp_branch_format: "feature/{slug}/WP{id}".to_string(),
            block_direct_main_commits: true,
        }
    }
}

#[derive(Debug)]
pub enum TopologyError {
    DirectMainCommit,
    InvalidBranchName { name: String, reason: String },
    PolicyViolation { message: String },
}

impl std::fmt::Display for TopologyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopologyError::DirectMainCommit => write!(f, "Direct commits to main are blocked"),
            TopologyError::InvalidBranchName { name, reason } => {
                write!(f, "Invalid branch name '{}': {}", name, reason)
            }
            TopologyError::PolicyViolation { message } => {
                write!(f, "Policy violation: {}", message)
            }
        }
    }
}

impl std::error::Error for TopologyError {}

pub struct BranchTopology {
    policy: BranchPolicy,
}

impl BranchTopology {
    pub fn new(policy: BranchPolicy) -> Self {
        Self { policy }
    }

    pub fn from_config_file(path: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let content = std::fs::read_to_string(path)?;
        let policy: BranchPolicy = toml::from_str(&content)?;
        Ok(Self::new(policy))
    }

    /// Generate the expected branch name for a feature
    pub fn feature_branch(&self, slug: &str) -> String {
        format!("{}{}", self.policy.feature_prefix, slug)
    }

    /// Generate the expected branch name for a WP within a feature
    pub fn wp_branch(&self, slug: &str, wp_id: &str) -> String {
        self.policy
            .wp_branch_format
            .replace("{slug}", slug)
            .replace("{id}", wp_id)
    }

    /// Validate that a branch name conforms to policy
    pub fn validate_branch(&self, branch: &str) -> Result<(), TopologyError> {
        if branch == self.policy.main_branch {
            return Ok(()); // main itself is valid
        }

        if branch.starts_with(&self.policy.feature_prefix) {
            return Ok(());
        }

        // Check release pattern (simple glob)
        let release_prefix = self.policy.release_pattern.trim_end_matches('*');
        if branch.starts_with(release_prefix) {
            return Ok(());
        }

        Err(TopologyError::InvalidBranchName {
            name: branch.to_string(),
            reason: format!(
                "Branch must start with '{}' for features, '{}' for releases, or be '{}'",
                self.policy.feature_prefix, release_prefix, self.policy.main_branch
            ),
        })
    }

    /// Check if a commit to a branch is allowed
    pub fn check_commit(&self, branch: &str) -> Result<(), TopologyError> {
        if branch == self.policy.main_branch && self.policy.block_direct_main_commits {
            return Err(TopologyError::DirectMainCommit);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_topology() -> BranchTopology {
        BranchTopology::new(BranchPolicy::default())
    }

    #[test]
    fn feature_branch_format() {
        let t = default_topology();
        assert_eq!(t.feature_branch("my-feature"), "feature/my-feature");
    }

    #[test]
    fn wp_branch_format() {
        let t = default_topology();
        assert_eq!(t.wp_branch("my-feature", "01"), "feature/my-feature/WP01");
    }

    #[test]
    fn validate_main_branch() {
        let t = default_topology();
        assert!(t.validate_branch("main").is_ok());
    }

    #[test]
    fn validate_feature_branch() {
        let t = default_topology();
        assert!(t.validate_branch("feature/foo").is_ok());
    }

    #[test]
    fn validate_release_branch() {
        let t = default_topology();
        assert!(t.validate_branch("release/1.0").is_ok());
    }

    #[test]
    fn validate_invalid_branch() {
        let t = default_topology();
        assert!(t.validate_branch("hotfix/oops").is_err());
    }

    #[test]
    fn commit_to_main_blocked() {
        let t = default_topology();
        assert!(t.check_commit("main").is_err());
    }

    #[test]
    fn commit_to_feature_allowed() {
        let t = default_topology();
        assert!(t.check_commit("feature/foo").is_ok());
    }

    #[test]
    fn commit_to_main_allowed_when_unblocked() {
        let policy = BranchPolicy {
            block_direct_main_commits: false,
            ..BranchPolicy::default()
        };
        let t = BranchTopology::new(policy);
        assert!(t.check_commit("main").is_ok());
    }
}
