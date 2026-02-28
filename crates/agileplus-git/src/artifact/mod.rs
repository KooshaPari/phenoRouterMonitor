//! Git artifact read/write operations for AgilePlus.
//!
//! Artifacts live under kitty-specs/<feature-slug>/<relative-path>.
//! write_artifact stages the file but does NOT commit.
//!
//! Traceability: WP07-T041, WP07-T042

use std::path::{Path, PathBuf};

use agileplus_domain::{error::DomainError, ports::FeatureArtifacts};

use crate::GitVcsAdapter;

/// Compute full artifact path from repo root.
fn artifact_full_path(repo_root: &Path, feature_slug: &str, relative_path: &str) -> PathBuf {
    repo_root
        .join("kitty-specs")
        .join(feature_slug)
        .join(relative_path)
}

/// Read a text artifact from the working tree.
pub(crate) fn read_artifact(
    adapter: &GitVcsAdapter,
    feature_slug: &str,
    relative_path: &str,
) -> Result<String, DomainError> {
    let path = artifact_full_path(adapter.repo_path(), feature_slug, relative_path);
    if !path.exists() {
        return Err(DomainError::NotFound(format!(
            "artifact not found: kitty-specs/{feature_slug}/{relative_path}"
        )));
    }
    std::fs::read_to_string(&path)
        .map_err(|e| DomainError::Vcs(format!("read artifact: {e}")))
}

/// Write a text artifact and stage it in the git index.
pub(crate) fn write_artifact(
    adapter: &GitVcsAdapter,
    feature_slug: &str,
    relative_path: &str,
    content: &str,
) -> Result<(), DomainError> {
    let path = artifact_full_path(adapter.repo_path(), feature_slug, relative_path);

    // Create parent directories.
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| DomainError::Vcs(format!("create artifact dirs: {e}")))?;
    }

    std::fs::write(&path, content)
        .map_err(|e| DomainError::Vcs(format!("write artifact: {e}")))?;

    // Stage the file in git index.
    let repo = adapter.open_repo()?;
    let mut index = repo.index().map_err(crate::git_err)?;

    // Path relative to repo workdir for index.
    let relative = path
        .strip_prefix(adapter.repo_path())
        .map_err(|_| DomainError::Vcs("artifact path outside repo".into()))?;

    index.add_path(relative).map_err(crate::git_err)?;
    index.write().map_err(crate::git_err)?;

    Ok(())
}

/// Check whether an artifact exists in the working tree.
pub(crate) fn artifact_exists(
    adapter: &GitVcsAdapter,
    feature_slug: &str,
    relative_path: &str,
) -> Result<bool, DomainError> {
    let path = artifact_full_path(adapter.repo_path(), feature_slug, relative_path);
    Ok(path.exists())
}

/// Scan and collect all artifacts for a feature from the working tree.
pub(crate) fn scan_feature_artifacts(
    adapter: &GitVcsAdapter,
    feature_slug: &str,
) -> Result<FeatureArtifacts, DomainError> {
    let base = adapter
        .repo_path()
        .join("kitty-specs")
        .join(feature_slug);

    let meta_json = {
        let p = base.join("meta.json");
        if p.exists() {
            Some(std::fs::read_to_string(&p).map_err(|e| DomainError::Vcs(e.to_string()))?)
        } else {
            None
        }
    };

    let audit_chain = {
        let p = base.join("audit").join("chain.jsonl");
        if p.exists() {
            Some(std::fs::read_to_string(&p).map_err(|e| DomainError::Vcs(e.to_string()))?)
        } else {
            None
        }
    };

    let evidence_paths = collect_evidence(&base.join("evidence"))?;

    Ok(FeatureArtifacts {
        meta_json,
        audit_chain,
        evidence_paths,
    })
}

/// Recursively collect file paths under the evidence directory.
fn collect_evidence(evidence_dir: &Path) -> Result<Vec<String>, DomainError> {
    if !evidence_dir.exists() {
        return Ok(vec![]);
    }
    let mut paths = Vec::new();
    collect_evidence_recursive(evidence_dir, &mut paths)?;
    Ok(paths)
}

fn collect_evidence_recursive(dir: &Path, paths: &mut Vec<String>) -> Result<(), DomainError> {
    for entry in std::fs::read_dir(dir).map_err(|e| DomainError::Vcs(e.to_string()))? {
        let entry = entry.map_err(|e| DomainError::Vcs(e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_evidence_recursive(&path, paths)?;
        } else {
            paths.push(path.to_string_lossy().to_string());
        }
    }
    Ok(())
}
