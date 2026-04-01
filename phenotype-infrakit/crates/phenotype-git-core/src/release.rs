//! Canonical release tracking.
//!
//! Detects which release branch a repository is on and extracts release metadata.

use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{GitError, Result};

/// Release information extracted from a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub branch: String,
    pub tag: Option<String>,
    pub commit: String,
    pub date: DateTime<Utc>,
}

/// Track the current release state of a repository.
///
/// Returns `Ok(None)` if the repository is not on a release branch.
pub fn track_release(repo_path: impl AsRef<Path>) -> Result<Option<ReleaseInfo>> {
    let repo_path = repo_path.as_ref();
    let repo = gix::open(repo_path).map_err(|e| GitError::Gix(e.to_string()))?;

    let head = repo.head().map_err(|e| GitError::Gix(e.to_string()))?;

    if head.is_detached() {
        return Ok(None);
    }

    let branch_name = head
        .referent_name()
        .map(|n| n.shorten().to_string())
        .ok_or_else(|| GitError::Release("no referent name".into()))?;

    if !is_release_branch(&branch_name) {
        return Ok(None);
    }

    let head_commit = repo
        .head_commit()
        .map_err(|e| GitError::Gix(e.to_string()))?;
    let commit = head_commit.id.to_string();

    let date = extract_commit_date(&repo, &head_commit);
    let tag = find_tag_at_commit(&repo, &commit);

    Ok(Some(ReleaseInfo {
        branch: branch_name,
        tag,
        commit,
        date,
    }))
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn is_release_branch(name: &str) -> bool {
    name.starts_with("release/")
        || name.starts_with("releases/")
        || name.starts_with('v') && name.contains('.')
}

fn extract_commit_date(repo: &gix::Repository, head_commit: &gix::Commit<'_>) -> DateTime<Utc> {
    let id = head_commit.id;
    if let Ok(obj) = repo.find_object(id) {
        if let Ok(commit) = obj.try_into_commit() {
            if let Ok(platform) = commit.decode() {
                let committer_result = platform.committer();
                if let Ok(sig) = committer_result {
                    let secs = sig.time.seconds;
                    let offset = sig.time.offset;
                    let dt = DateTime::from_timestamp(secs as i64, 0).unwrap_or_default();
                    let offset_hours = offset / 60;
                    let offset_minutes = (offset % 60).abs();
                    if let Some(fixed) =
                        chrono::FixedOffset::east_opt(offset_hours * 3600 + offset_minutes * 60)
                    {
                        return dt.with_timezone(&fixed).into();
                    }
                    return dt;
                }
            }
        }
    }
    Utc::now()
}

fn find_tag_at_commit(repo: &gix::Repository, commit: &str) -> Option<String> {
    let refs = repo.references().ok()?;
    let all = refs.all().ok()?;
    for ref_result in all {
        let r = ref_result.ok()?;
        let name = r.name().shorten().to_string();
        if !name.starts_with('v') && !name.contains('.') {
            continue;
        }

        let target = r.target();
        if let Some(id) = target.try_id() {
            if id.to_string() == commit {
                return Some(name);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-GIT-011
    #[test]
    fn release_info_serialization() {
        let info = ReleaseInfo {
            branch: "release/v1.0.0".into(),
            tag: Some("v1.0.0".into()),
            commit: "abc123".into(),
            date: Utc::now(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("release/v1.0.0"));
        assert!(json.contains("v1.0.0"));

        let roundtrip: ReleaseInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.branch, "release/v1.0.0");
        assert_eq!(roundtrip.tag, Some("v1.0.0".into()));
    }

    // Traces to: FR-GIT-012
    #[test]
    fn is_release_branch_detection() {
        assert!(is_release_branch("release/v1.0.0"));
        assert!(is_release_branch("releases/2024.01"));
        assert!(is_release_branch("v1.2.3"));
        assert!(!is_release_branch("main"));
        assert!(!is_release_branch("feature/auth"));
        assert!(!is_release_branch("develop"));
    }

    // Traces to: FR-GIT-013
    #[test]
    fn track_release_non_release_branch() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path();

        std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["commit", "--allow-empty", "-m", "init"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let result = track_release(repo_path).unwrap();
        assert!(result.is_none());
    }

    // Traces to: FR-GIT-014
    #[test]
    fn track_release_nonexistent_repo() {
        let result = track_release("/nonexistent/path");
        assert!(result.is_err());
    }

    // Traces to: FR-GIT-015
    #[test]
    fn release_info_no_tag() {
        let info = ReleaseInfo {
            branch: "release/v2.0.0".into(),
            tag: None,
            commit: "def789".into(),
            date: Utc::now(),
        };
        assert!(info.tag.is_none());
        assert_eq!(info.commit, "def789");
    }
}
