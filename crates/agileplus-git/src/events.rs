//! NATS event publishing for git observer events.

use crate::observer::GitEvent;
use serde::Serialize;

/// Subject prefix for all git events.
pub const GIT_SUBJECT_PREFIX: &str = "agileplus.git";

#[derive(Debug, Clone, Serialize)]
pub struct GitEventEnvelope {
    pub event_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub repo_root: String,
    pub payload: serde_json::Value,
}

impl GitEventEnvelope {
    pub fn from_git_event(event: &GitEvent, repo_root: &str) -> Self {
        let (event_type, payload) = match event {
            GitEvent::RefChanged {
                ref_name,
                old_oid,
                new_oid,
            } => (
                "ref_changed".to_string(),
                serde_json::json!({ "ref": ref_name, "old": old_oid, "new": new_oid }),
            ),
            GitEvent::Checkout { branch } => (
                "checkout".to_string(),
                serde_json::json!({ "branch": branch }),
            ),
            GitEvent::Merge { source, target } => (
                "merge".to_string(),
                serde_json::json!({ "source": source, "target": target }),
            ),
            GitEvent::Rebase { branch } => (
                "rebase".to_string(),
                serde_json::json!({ "branch": branch }),
            ),
            GitEvent::WorktreeAdded { path } => (
                "worktree_added".to_string(),
                serde_json::json!({ "path": path.display().to_string() }),
            ),
            GitEvent::WorktreeRemoved { path } => (
                "worktree_removed".to_string(),
                serde_json::json!({ "path": path.display().to_string() }),
            ),
        };

        Self {
            event_type,
            timestamp: chrono::Utc::now(),
            repo_root: repo_root.to_string(),
            payload,
        }
    }

    pub fn nats_subject(&self) -> String {
        format!("{}.{}", GIT_SUBJECT_PREFIX, self.event_type)
    }
}

/// Publishes a git event to NATS. Caller provides the NATS client.
pub async fn publish_git_event(
    nats: &async_nats::Client,
    event: &GitEvent,
    repo_root: &str,
) -> Result<(), async_nats::PublishError> {
    let envelope = GitEventEnvelope::from_git_event(event, repo_root);
    let subject = envelope.nats_subject();
    let payload = serde_json::to_vec(&envelope).unwrap_or_default();
    nats.publish(subject, payload.into()).await
}
