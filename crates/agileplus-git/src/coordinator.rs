use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeClaim {
    pub path: PathBuf,
    pub agent_id: String,
    pub session_id: String,
    pub wp_id: String,
    pub branch: String,
    pub heartbeat: DateTime<Utc>,
    pub status: ClaimStatus,
    pub file_scope: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClaimStatus {
    Active,
    Stale,
    Released,
}

#[derive(Debug)]
pub enum ClaimError {
    AlreadyClaimed {
        existing_agent: String,
        wp_id: String,
    },
    FileScopeOverlap {
        overlapping_files: Vec<String>,
        other_agent: String,
    },
    WorktreeNotFound,
}

impl std::fmt::Display for ClaimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClaimError::AlreadyClaimed {
                existing_agent,
                wp_id,
            } => {
                write!(
                    f,
                    "Worktree already claimed by agent '{}' for WP '{}'",
                    existing_agent, wp_id
                )
            }
            ClaimError::FileScopeOverlap {
                overlapping_files,
                other_agent,
            } => {
                write!(
                    f,
                    "File scope overlap with agent '{}': {:?}",
                    other_agent, overlapping_files
                )
            }
            ClaimError::WorktreeNotFound => write!(f, "Worktree not found"),
        }
    }
}

impl std::error::Error for ClaimError {}

/// In-memory coordinator. Production would back this with SQLite + Dragonfly.
pub struct WorktreeCoordinator {
    claims: Vec<WorktreeClaim>,
    stale_threshold: chrono::Duration,
}

impl WorktreeCoordinator {
    pub fn new() -> Self {
        Self {
            claims: Vec::new(),
            stale_threshold: chrono::Duration::minutes(5),
        }
    }

    pub fn with_stale_threshold(mut self, threshold: chrono::Duration) -> Self {
        self.stale_threshold = threshold;
        self
    }

    /// Claim a worktree for an agent/WP
    pub fn claim(
        &mut self,
        path: PathBuf,
        agent_id: String,
        session_id: String,
        wp_id: String,
        branch: String,
        file_scope: Vec<String>,
    ) -> Result<&WorktreeClaim, ClaimError> {
        // Check for existing active claim on same path
        if let Some(existing) = self
            .claims
            .iter()
            .find(|c| c.path == path && c.status == ClaimStatus::Active)
        {
            return Err(ClaimError::AlreadyClaimed {
                existing_agent: existing.agent_id.clone(),
                wp_id: existing.wp_id.clone(),
            });
        }

        // Check for file scope overlap with other active claims
        for claim in self
            .claims
            .iter()
            .filter(|c| c.status == ClaimStatus::Active)
        {
            let overlapping: Vec<String> = file_scope
                .iter()
                .filter(|f| claim.file_scope.contains(f))
                .cloned()
                .collect();

            if !overlapping.is_empty() {
                return Err(ClaimError::FileScopeOverlap {
                    overlapping_files: overlapping,
                    other_agent: claim.agent_id.clone(),
                });
            }
        }

        let claim = WorktreeClaim {
            path,
            agent_id,
            session_id,
            wp_id,
            branch,
            heartbeat: Utc::now(),
            status: ClaimStatus::Active,
            file_scope,
        };

        self.claims.push(claim);
        Ok(self.claims.last().unwrap())
    }

    /// Update heartbeat for an active claim
    pub fn heartbeat(&mut self, path: &std::path::Path, agent_id: &str) -> bool {
        if let Some(claim) = self
            .claims
            .iter_mut()
            .find(|c| c.path == path && c.agent_id == agent_id && c.status == ClaimStatus::Active)
        {
            claim.heartbeat = Utc::now();
            true
        } else {
            false
        }
    }

    /// Release a claim
    pub fn release(&mut self, path: &std::path::Path, agent_id: &str) -> bool {
        if let Some(claim) = self
            .claims
            .iter_mut()
            .find(|c| c.path == path && c.agent_id == agent_id && c.status == ClaimStatus::Active)
        {
            claim.status = ClaimStatus::Released;
            true
        } else {
            false
        }
    }

    /// Detect and mark stale claims
    pub fn detect_stale(&mut self) -> Vec<WorktreeClaim> {
        let now = Utc::now();
        let threshold = self.stale_threshold;
        let mut stale = Vec::new();

        for claim in self
            .claims
            .iter_mut()
            .filter(|c| c.status == ClaimStatus::Active)
        {
            if now.signed_duration_since(claim.heartbeat) > threshold {
                claim.status = ClaimStatus::Stale;
                stale.push(claim.clone());
            }
        }

        stale
    }

    /// Force-release a stale claim (for recovery)
    pub fn force_release(&mut self, path: &std::path::Path) -> bool {
        if let Some(claim) = self.claims.iter_mut().find(|c| {
            c.path == path && (c.status == ClaimStatus::Active || c.status == ClaimStatus::Stale)
        }) {
            claim.status = ClaimStatus::Released;
            true
        } else {
            false
        }
    }

    /// List all active claims
    pub fn active_claims(&self) -> Vec<&WorktreeClaim> {
        self.claims
            .iter()
            .filter(|c| c.status == ClaimStatus::Active)
            .collect()
    }
}

impl Default for WorktreeCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_and_release() {
        let mut coord = WorktreeCoordinator::new();
        let path = PathBuf::from("/worktrees/wt1");
        coord
            .claim(
                path.clone(),
                "agent-1".into(),
                "sess-1".into(),
                "WP01".into(),
                "feature/foo/WP01".into(),
                vec!["src/lib.rs".into()],
            )
            .unwrap();
        assert_eq!(coord.active_claims().len(), 1);
        assert!(coord.release(&path, "agent-1"));
        assert_eq!(coord.active_claims().len(), 0);
    }

    #[test]
    fn double_claim_same_path_fails() {
        let mut coord = WorktreeCoordinator::new();
        let path = PathBuf::from("/worktrees/wt1");
        coord
            .claim(
                path.clone(),
                "a1".into(),
                "s1".into(),
                "WP01".into(),
                "b1".into(),
                vec![],
            )
            .unwrap();
        let result = coord.claim(
            path.clone(),
            "a2".into(),
            "s2".into(),
            "WP02".into(),
            "b2".into(),
            vec![],
        );
        assert!(matches!(result, Err(ClaimError::AlreadyClaimed { .. })));
    }

    #[test]
    fn file_scope_overlap_fails() {
        let mut coord = WorktreeCoordinator::new();
        coord
            .claim(
                PathBuf::from("/wt1"),
                "a1".into(),
                "s1".into(),
                "WP01".into(),
                "b1".into(),
                vec!["src/main.rs".into()],
            )
            .unwrap();
        let result = coord.claim(
            PathBuf::from("/wt2"),
            "a2".into(),
            "s2".into(),
            "WP02".into(),
            "b2".into(),
            vec!["src/main.rs".into()],
        );
        assert!(matches!(result, Err(ClaimError::FileScopeOverlap { .. })));
    }

    #[test]
    fn heartbeat_updates_timestamp() {
        let mut coord = WorktreeCoordinator::new();
        let path = PathBuf::from("/wt1");
        coord
            .claim(
                path.clone(),
                "a1".into(),
                "s1".into(),
                "WP01".into(),
                "b1".into(),
                vec![],
            )
            .unwrap();
        assert!(coord.heartbeat(&path, "a1"));
    }

    #[test]
    fn stale_detection() {
        let mut coord =
            WorktreeCoordinator::new().with_stale_threshold(chrono::Duration::seconds(-1)); // immediately stale
        let path = PathBuf::from("/wt1");
        coord
            .claim(
                path.clone(),
                "a1".into(),
                "s1".into(),
                "WP01".into(),
                "b1".into(),
                vec![],
            )
            .unwrap();
        let stale = coord.detect_stale();
        assert_eq!(stale.len(), 1);
        assert_eq!(stale[0].status, ClaimStatus::Stale);
    }

    #[test]
    fn force_release_stale() {
        let mut coord =
            WorktreeCoordinator::new().with_stale_threshold(chrono::Duration::seconds(-1));
        let path = PathBuf::from("/wt1");
        coord
            .claim(
                path.clone(),
                "a1".into(),
                "s1".into(),
                "WP01".into(),
                "b1".into(),
                vec![],
            )
            .unwrap();
        coord.detect_stale();
        assert!(coord.force_release(&path));
        assert_eq!(coord.active_claims().len(), 0);
    }
}
