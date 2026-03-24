//! Git observer - watches .git/ for ref changes and emits typed events.

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use tokio::sync::broadcast;
use tracing::{debug, info};

#[derive(Debug, Clone, serde::Serialize)]
pub enum GitEvent {
    RefChanged {
        ref_name: String,
        old_oid: Option<String>,
        new_oid: String,
    },
    Checkout {
        branch: String,
    },
    Merge {
        source: String,
        target: String,
    },
    Rebase {
        branch: String,
    },
    WorktreeAdded {
        path: PathBuf,
    },
    WorktreeRemoved {
        path: PathBuf,
    },
}

pub struct GitObserver {
    repo_root: PathBuf,
    tx: broadcast::Sender<GitEvent>,
    _watcher: RecommendedWatcher,
}

impl GitObserver {
    pub fn new(repo_root: PathBuf) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, _) = broadcast::channel(256);
        let tx_clone = tx.clone();
        let root = repo_root.clone();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                Self::handle_fs_event(&root, &tx_clone, event);
            }
        })?;

        // Watch .git directory for ref changes
        let git_dir = repo_root.join(".git");
        if git_dir.exists() {
            watcher.watch(&git_dir.join("refs"), RecursiveMode::Recursive)?;
            watcher.watch(&git_dir.join("HEAD"), RecursiveMode::NonRecursive)?;
        }

        info!("Git observer started for {}", repo_root.display());

        Ok(Self {
            repo_root,
            tx,
            _watcher: watcher,
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<GitEvent> {
        self.tx.subscribe()
    }

    pub fn repo_root(&self) -> &std::path::Path {
        &self.repo_root
    }

    fn handle_fs_event(
        repo_root: &std::path::Path,
        tx: &broadcast::Sender<GitEvent>,
        event: Event,
    ) {
        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                for path in &event.paths {
                    if let Some(git_event) = Self::classify_change(repo_root, path) {
                        debug!("Git event: {:?}", git_event);
                        let _ = tx.send(git_event);
                    }
                }
            }
            _ => {}
        }
    }

    fn classify_change(repo_root: &std::path::Path, path: &std::path::Path) -> Option<GitEvent> {
        let rel = path.strip_prefix(repo_root.join(".git")).ok()?;
        let rel_str = rel.to_string_lossy();

        if rel_str == "HEAD" {
            let head_content = std::fs::read_to_string(path).ok()?;
            if head_content.starts_with("ref: refs/heads/") {
                let branch = head_content
                    .trim()
                    .strip_prefix("ref: refs/heads/")?
                    .to_string();
                return Some(GitEvent::Checkout { branch });
            }
        }

        if rel_str.starts_with("refs/heads/") {
            let ref_name = rel_str.to_string();
            let new_oid = std::fs::read_to_string(path).ok()?.trim().to_string();
            return Some(GitEvent::RefChanged {
                ref_name,
                old_oid: None,
                new_oid,
            });
        }

        if rel_str.starts_with("refs/stash") || rel_str.contains("MERGE_HEAD") {
            return Some(GitEvent::Merge {
                source: "unknown".to_string(),
                target: "HEAD".to_string(),
            });
        }

        if rel_str.contains("rebase-merge") || rel_str.contains("rebase-apply") {
            return Some(GitEvent::Rebase {
                branch: "HEAD".to_string(),
            });
        }

        None
    }
}
