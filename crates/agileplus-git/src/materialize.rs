//! Document materialization: renders domain entities as git-tracked files.
//! One-way sync: SSOT -> git. Direct edits to materialized files are overwritten.
//!
//! Writes the following layout under `kitty-specs/<slug>/`:
//!
//! ```text
//! kitty-specs/<slug>/
//!   meta.json        -- Feature metadata (slug, name, state, git provenance)
//!   status.md        -- Human-readable status with WP table
//!   audit.jsonl      -- Append-only materialization audit log
//!   wp/
//!     <wp_id>.json   -- One file per WorkPackage
//! ```
//!
//! Traceability: L4 SSOT materialization layer

use std::{
    io::Write as IoWrite,
    path::{Path, PathBuf},
};

use agileplus_domain::{
    domain::{feature::Feature, work_package::WorkPackage},
    error::DomainError,
};
use chrono::Utc;
use serde_json::{Value, json};

use crate::GitVcsAdapter;

// ---------------------------------------------------------------------------
// Pure rendering functions (no I/O, easily testable)
// ---------------------------------------------------------------------------

/// Render `meta.json` content for a feature.
pub fn render_meta_json(feature: &Feature) -> Value {
    json!({
        "slug": feature.slug,
        "friendly_name": feature.friendly_name,
        "state": feature.state.to_string(),
        "target_branch": feature.target_branch,
        "spec_hash": feature.spec_hash.iter().map(|b| format!("{b:02x}")).collect::<String>(),
        "created_at": feature.created_at.to_rfc3339(),
        "updated_at": feature.updated_at.to_rfc3339(),
        "created_at_commit": feature.created_at_commit,
        "last_modified_commit": feature.last_modified_commit,
        "labels": feature.labels,
        "plane_issue_id": feature.plane_issue_id,
        "materialized_at": Utc::now().to_rfc3339(),
    })
}

/// Render `status.md` content for a feature with its work packages.
pub fn render_status_md(feature: &Feature, work_packages: &[WorkPackage]) -> String {
    let mut out = String::new();

    out.push_str(&format!("# {}\n\n", feature.friendly_name));
    out.push_str(&format!("**Slug**: `{}`  \n", feature.slug));
    out.push_str(&format!("**State**: {}  \n", feature.state));
    out.push_str(&format!(
        "**Target branch**: `{}`  \n",
        feature.target_branch
    ));
    out.push_str(&format!(
        "**Updated**: {}  \n",
        feature.updated_at.to_rfc3339()
    ));

    if !feature.labels.is_empty() {
        out.push_str(&format!("**Labels**: {}  \n", feature.labels.join(", ")));
    }

    out.push('\n');
    out.push_str("---\n\n");

    if work_packages.is_empty() {
        out.push_str("*No work packages.*\n");
    } else {
        out.push_str("## Work Packages\n\n");
        out.push_str("| ID | Seq | Title | State | Agent | Branch |\n");
        out.push_str("|----|-----|-------|-------|-------|--------|\n");

        for wp in work_packages {
            let agent = wp.agent_id.as_deref().unwrap_or("-");
            let branch = wp.worktree_path.as_deref().unwrap_or("-");
            let state_str = format!("{:?}", wp.state).to_lowercase();
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                wp.id, wp.sequence, wp.title, state_str, agent, branch,
            ));
        }
    }

    out.push('\n');
    out.push_str(&format!(
        "> *Materialized at {}. Do not edit directly -- changes will be overwritten.*\n",
        Utc::now().to_rfc3339(),
    ));

    out
}

/// Render a single audit JSONL line (one JSON object, no newline at start).
pub fn render_audit_line(feature: &Feature, commit_oid: Option<&str>) -> String {
    let entry = json!({
        "timestamp": Utc::now().to_rfc3339(),
        "action": "materialized",
        "slug": feature.slug,
        "state": feature.state.to_string(),
        "commit": commit_oid,
    });
    // serde_json::to_string never fails for a well-formed Value.
    serde_json::to_string(&entry).expect("audit line serialization failed")
}

/// Render `wp/<id>.json` content for a work package.
pub fn render_wp_json(wp: &WorkPackage) -> Value {
    json!({
        "id": wp.id,
        "feature_id": wp.feature_id,
        "title": wp.title,
        "state": format!("{:?}", wp.state).to_lowercase(),
        "sequence": wp.sequence,
        "acceptance_criteria": wp.acceptance_criteria,
        "file_scope": wp.file_scope,
        "agent_id": wp.agent_id,
        "pr_url": wp.pr_url,
        "pr_state": wp.pr_state.map(|s| format!("{s:?}").to_lowercase()),
        "worktree_path": wp.worktree_path,
        "base_commit": wp.base_commit,
        "head_commit": wp.head_commit,
        "created_at": wp.created_at.to_rfc3339(),
        "updated_at": wp.updated_at.to_rfc3339(),
        "materialized_at": Utc::now().to_rfc3339(),
    })
}

// ---------------------------------------------------------------------------
// Git staging helpers
// ---------------------------------------------------------------------------

/// Write `content` to `full_path`, creating parent dirs, and stage the file.
fn write_and_stage(
    repo: &git2::Repository,
    repo_root: &Path,
    full_path: &Path,
    content: &[u8],
) -> Result<(), DomainError> {
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            DomainError::Vcs(format!("create dirs for {}: {e}", full_path.display()))
        })?;
    }

    std::fs::write(full_path, content)
        .map_err(|e| DomainError::Vcs(format!("write {}: {e}", full_path.display())))?;

    let relative = full_path
        .strip_prefix(repo_root)
        .map_err(|_| DomainError::Vcs("materialized file outside repo root".into()))?;

    let mut index = repo.index().map_err(crate::git_err)?;
    index.add_path(relative).map_err(crate::git_err)?;
    index.write().map_err(crate::git_err)?;

    Ok(())
}

/// Append `line` to an append-only JSONL file and stage it.
///
/// Creates the file if it does not exist. Each call appends exactly one line.
fn append_and_stage(
    repo: &git2::Repository,
    repo_root: &Path,
    full_path: &Path,
    line: &str,
) -> Result<(), DomainError> {
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            DomainError::Vcs(format!("create dirs for {}: {e}", full_path.display()))
        })?;
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(full_path)
        .map_err(|e| DomainError::Vcs(format!("open audit file: {e}")))?;

    writeln!(file, "{line}").map_err(|e| DomainError::Vcs(format!("append audit line: {e}")))?;

    let relative = full_path
        .strip_prefix(repo_root)
        .map_err(|_| DomainError::Vcs("audit file outside repo root".into()))?;

    let mut index = repo.index().map_err(crate::git_err)?;
    index.add_path(relative).map_err(crate::git_err)?;
    index.write().map_err(crate::git_err)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Materialize a feature and its work packages as git-tracked files.
///
/// Writes `meta.json`, `status.md`, and appends a line to `audit.jsonl` under
/// `kitty-specs/<slug>/`, staging all modified files in the git index.
///
/// Returns the feature directory path (`kitty-specs/<slug>/`).
pub fn materialize_feature(
    adapter: &GitVcsAdapter,
    feature: &Feature,
    work_packages: &[WorkPackage],
) -> Result<PathBuf, DomainError> {
    let repo = adapter.open_repo()?;
    let repo_root = adapter.repo_path();
    let feature_dir = repo_root.join("kitty-specs").join(&feature.slug);

    // meta.json (overwrite every time -- SSOT wins)
    let meta_content = serde_json::to_string_pretty(&render_meta_json(feature))
        .map_err(|e| DomainError::Vcs(format!("serialize meta.json: {e}")))?;
    write_and_stage(
        &repo,
        repo_root,
        &feature_dir.join("meta.json"),
        meta_content.as_bytes(),
    )?;

    // status.md (overwrite every time)
    let status_content = render_status_md(feature, work_packages);
    write_and_stage(
        &repo,
        repo_root,
        &feature_dir.join("status.md"),
        status_content.as_bytes(),
    )?;

    // audit.jsonl (append-only)
    let audit_line = render_audit_line(feature, feature.last_modified_commit.as_deref());
    append_and_stage(
        &repo,
        repo_root,
        &feature_dir.join("audit.jsonl"),
        &audit_line,
    )?;

    Ok(feature_dir)
}

/// Materialize a single work package as a git-tracked JSON file.
///
/// Writes `kitty-specs/<feature_slug>/wp/<wp_id>.json` and stages it.
pub fn materialize_work_package(
    adapter: &GitVcsAdapter,
    feature_slug: &str,
    wp: &WorkPackage,
) -> Result<(), DomainError> {
    let repo = adapter.open_repo()?;
    let repo_root = adapter.repo_path();
    let wp_path = repo_root
        .join("kitty-specs")
        .join(feature_slug)
        .join("wp")
        .join(format!("{}.json", wp.id));

    let content = serde_json::to_string_pretty(&render_wp_json(wp))
        .map_err(|e| DomainError::Vcs(format!("serialize wp {}: {e}", wp.id)))?;

    write_and_stage(&repo, repo_root, &wp_path, content.as_bytes())?;

    Ok(())
}

/// Commit all currently staged materialization files.
///
/// Uses the repository's default signature when available, falling back to
/// "agileplus-bot <agileplus-bot@localhost>".
///
/// Returns the new commit OID as a hex string.
pub fn commit_materialization(
    adapter: &GitVcsAdapter,
    feature_slug: &str,
    message: Option<&str>,
) -> Result<String, DomainError> {
    let repo = adapter.open_repo()?;

    let default_msg = format!("chore(specs): materialize {feature_slug} artifacts");
    let commit_msg = message.unwrap_or(&default_msg);

    // Resolve signature: try repo config, fall back to bot identity.
    let sig = repo.signature().unwrap_or_else(|_| {
        git2::Signature::now("agileplus-bot", "agileplus-bot@localhost")
            .expect("static signature is always valid")
    });

    // Write the current index to a tree.
    let mut index = repo.index().map_err(crate::git_err)?;
    let tree_oid = index.write_tree().map_err(crate::git_err)?;
    let tree = repo.find_tree(tree_oid).map_err(crate::git_err)?;

    // Determine parent commit (HEAD), if any.
    let parent_commit = match repo.head() {
        Ok(head_ref) => {
            let head_oid = head_ref
                .target()
                .ok_or_else(|| DomainError::Vcs("HEAD reference has no target OID".into()))?;
            Some(repo.find_commit(head_oid).map_err(crate::git_err)?)
        }
        Err(e) if e.code() == git2::ErrorCode::UnbornBranch => None,
        Err(e) => return Err(crate::git_err(e)),
    };

    let parents: Vec<&git2::Commit<'_>> = parent_commit.iter().collect();

    let commit_oid = repo
        .commit(Some("HEAD"), &sig, &sig, commit_msg, &tree, &parents)
        .map_err(crate::git_err)?;

    Ok(commit_oid.to_string())
}

/// Convenience: materialize feature + all work packages, then commit.
///
/// Equivalent to calling `materialize_feature`, `materialize_work_package` for
/// each WP, and finally `commit_materialization`. Returns the commit OID.
pub fn materialize_and_commit(
    adapter: &GitVcsAdapter,
    feature: &Feature,
    work_packages: &[WorkPackage],
) -> Result<String, DomainError> {
    materialize_feature(adapter, feature, work_packages)?;

    for wp in work_packages {
        materialize_work_package(adapter, &feature.slug, wp)?;
    }

    commit_materialization(adapter, &feature.slug, None)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::domain::{
        feature::Feature,
        work_package::{WorkPackage, WpState},
    };

    fn make_feature() -> Feature {
        let mut f = Feature::new("my-feature", "My Feature", [0xabu8; 32], Some("main"));
        f.id = 1;
        f.labels = vec!["alpha".into(), "beta".into()];
        f.last_modified_commit = Some("deadbeef".into());
        f
    }

    fn make_wp(id: i64, title: &str) -> WorkPackage {
        let mut wp = WorkPackage::new(1, title, 1, "It must work.");
        wp.id = id;
        wp.agent_id = Some("agent-007".into());
        wp.worktree_path = Some("worktrees/my-feature-wp1".into());
        wp
    }

    // --- render_meta_json ---

    #[test]
    fn render_meta_json_contains_slug() {
        let f = make_feature();
        let v = render_meta_json(&f);
        assert_eq!(v["slug"], "my-feature");
    }

    #[test]
    fn render_meta_json_state_as_string() {
        let f = make_feature();
        let v = render_meta_json(&f);
        assert_eq!(v["state"], "created");
    }

    #[test]
    fn render_meta_json_spec_hash_hex() {
        let f = make_feature();
        let v = render_meta_json(&f);
        let hex = v["spec_hash"].as_str().unwrap();
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn render_meta_json_last_modified_commit() {
        let f = make_feature();
        let v = render_meta_json(&f);
        assert_eq!(v["last_modified_commit"], "deadbeef");
    }

    // --- render_status_md ---

    #[test]
    fn render_status_md_contains_title() {
        let f = make_feature();
        let md = render_status_md(&f, &[]);
        assert!(md.contains("My Feature"));
    }

    #[test]
    fn render_status_md_no_wps_shows_placeholder() {
        let f = make_feature();
        let md = render_status_md(&f, &[]);
        assert!(md.contains("No work packages"));
    }

    #[test]
    fn render_status_md_wp_table() {
        let f = make_feature();
        let wp = make_wp(42, "Implement login");
        let md = render_status_md(&f, &[wp]);
        assert!(md.contains("Implement login"));
        assert!(md.contains("42"));
        assert!(md.contains("agent-007"));
        assert!(md.contains("planned"));
    }

    #[test]
    fn render_status_md_multiple_wps() {
        let f = make_feature();
        let wp1 = make_wp(1, "First WP");
        let wp2 = make_wp(2, "Second WP");
        let md = render_status_md(&f, &[wp1, wp2]);
        assert!(md.contains("First WP"));
        assert!(md.contains("Second WP"));
    }

    #[test]
    fn render_status_md_labels() {
        let f = make_feature();
        let md = render_status_md(&f, &[]);
        assert!(md.contains("alpha"));
        assert!(md.contains("beta"));
    }

    #[test]
    fn render_status_md_overwrite_warning() {
        let f = make_feature();
        let md = render_status_md(&f, &[]);
        assert!(md.contains("Do not edit directly"));
    }

    // --- render_audit_line ---

    #[test]
    fn render_audit_line_is_valid_json() {
        let f = make_feature();
        let line = render_audit_line(&f, Some("abc123"));
        let v: Value = serde_json::from_str(&line).expect("audit line must be valid JSON");
        assert_eq!(v["action"], "materialized");
        assert_eq!(v["slug"], "my-feature");
        assert_eq!(v["commit"], "abc123");
    }

    #[test]
    fn render_audit_line_no_newline_in_value() {
        let f = make_feature();
        let line = render_audit_line(&f, None);
        // The rendered string itself should not contain embedded newlines (JSONL invariant).
        assert!(!line.contains('\n'));
    }

    #[test]
    fn render_audit_line_null_commit_when_none() {
        let f = make_feature();
        let line = render_audit_line(&f, None);
        let v: Value = serde_json::from_str(&line).unwrap();
        assert!(v["commit"].is_null());
    }

    // --- render_wp_json ---

    #[test]
    fn render_wp_json_contains_id() {
        let wp = make_wp(99, "My WP");
        let v = render_wp_json(&wp);
        assert_eq!(v["id"], 99);
    }

    #[test]
    fn render_wp_json_state_lowercase() {
        let wp = make_wp(1, "WP");
        let v = render_wp_json(&wp);
        assert_eq!(v["state"], "planned");
    }

    #[test]
    fn render_wp_json_doing_state() {
        let mut wp = make_wp(1, "WP");
        wp.state = WpState::Doing;
        let v = render_wp_json(&wp);
        assert_eq!(v["state"], "doing");
    }

    #[test]
    fn render_wp_json_agent_id() {
        let wp = make_wp(1, "WP");
        let v = render_wp_json(&wp);
        assert_eq!(v["agent_id"], "agent-007");
    }

    #[test]
    fn render_wp_json_null_pr_state_when_none() {
        let wp = make_wp(1, "WP");
        let v = render_wp_json(&wp);
        assert!(v["pr_state"].is_null());
    }

    #[test]
    fn render_wp_json_file_scope() {
        let mut wp = make_wp(1, "WP");
        wp.file_scope = vec!["src/main.rs".into(), "src/lib.rs".into()];
        let v = render_wp_json(&wp);
        let scope = v["file_scope"].as_array().unwrap();
        assert_eq!(scope.len(), 2);
        assert_eq!(scope[0], "src/main.rs");
    }

    // --- git integration tests (require tempfile) ---

    /// Create a bare-minimum initial commit in a freshly initialised repo so
    /// HEAD is valid and the index is ready for staging operations.
    fn make_initial_commit(repo: &git2::Repository) {
        let sig = git2::Signature::now("test", "test@test.com").unwrap();
        let tree_oid = {
            let mut idx = repo.index().unwrap();
            idx.write_tree().unwrap()
        };
        // tree is in its own block so it's dropped before the function returns.
        {
            let tree = repo.find_tree(tree_oid).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
                .unwrap();
        }
    }

    #[test]
    fn materialize_feature_creates_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        {
            let repo = git2::Repository::init(tmp.path()).unwrap();
            make_initial_commit(&repo);
        }

        let adapter = GitVcsAdapter::new(tmp.path().to_path_buf()).unwrap();
        let feature = make_feature();
        let wp = make_wp(1, "WP1");

        let dir = materialize_feature(&adapter, &feature, &[wp]).unwrap();
        assert!(dir.join("meta.json").exists());
        assert!(dir.join("status.md").exists());
        assert!(dir.join("audit.jsonl").exists());
    }

    #[test]
    fn materialize_work_package_creates_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        {
            let repo = git2::Repository::init(tmp.path()).unwrap();
            make_initial_commit(&repo);
        }

        let adapter = GitVcsAdapter::new(tmp.path().to_path_buf()).unwrap();
        let wp = make_wp(7, "My WP");
        materialize_work_package(&adapter, "my-feature", &wp).unwrap();

        let wp_path = tmp
            .path()
            .join("kitty-specs")
            .join("my-feature")
            .join("wp")
            .join("7.json");
        assert!(wp_path.exists());

        let content = std::fs::read_to_string(&wp_path).unwrap();
        let v: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(v["id"], 7);
    }

    #[test]
    fn materialize_and_commit_returns_oid() {
        let tmp = tempfile::TempDir::new().unwrap();
        {
            let repo = git2::Repository::init(tmp.path()).unwrap();
            make_initial_commit(&repo);
        }

        let adapter = GitVcsAdapter::new(tmp.path().to_path_buf()).unwrap();
        let feature = make_feature();
        let wp = make_wp(1, "WP1");

        let oid = materialize_and_commit(&adapter, &feature, &[wp]).unwrap();
        // A git OID is a 40-character hex string.
        assert_eq!(oid.len(), 40);
        assert!(oid.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn audit_jsonl_appends_across_calls() {
        let tmp = tempfile::TempDir::new().unwrap();
        {
            let repo = git2::Repository::init(tmp.path()).unwrap();
            make_initial_commit(&repo);
        }

        let adapter = GitVcsAdapter::new(tmp.path().to_path_buf()).unwrap();
        let feature = make_feature();

        materialize_feature(&adapter, &feature, &[]).unwrap();
        materialize_feature(&adapter, &feature, &[]).unwrap();

        let audit_path = tmp
            .path()
            .join("kitty-specs")
            .join("my-feature")
            .join("audit.jsonl");
        let content = std::fs::read_to_string(&audit_path).unwrap();
        let line_count = content.lines().count();
        assert_eq!(line_count, 2, "each materialization appends one audit line");
    }
}
