//! GitHub Issues sync logic with conflict detection.
//!
//! Syncs bugs to GitHub Issues with structured markdown bodies.
//! Detects body conflicts via SHA-256 hashing to prevent overwriting.
//!
//! Traceability: WP19-T110, T111, T112

use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use agileplus_triage::BacklogItem;
use crate::client::{GitHubClient, GitHubIssuePayload};

/// Sync state for GitHub Issues tracking.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitHubSyncState {
    /// Maps backlog item ID → GitHub issue number
    pub issue_mappings: HashMap<i64, i64>,
    /// Content hashes for conflict detection
    pub content_hashes: HashMap<i64, String>,
    pub last_synced_at: Option<DateTime<Utc>>,
}

/// GitHub sync adapter.
#[derive(Debug)]
pub struct GitHubSyncAdapter {
    client: GitHubClient,
}

/// Outcome of a sync operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncOutcome {
    Created(i64),
    Updated(i64),
    Skipped,
    Conflict { issue_number: i64, reason: String },
}

impl GitHubSyncAdapter {
    pub fn new(client: GitHubClient) -> Self {
        Self { client }
    }

    /// Sync a backlog bug item to GitHub Issues.
    pub async fn sync_bug(
        &self,
        state: &mut GitHubSyncState,
        item: &BacklogItem,
    ) -> Result<SyncOutcome> {
        let item_id = item.id.context("backlog item must have an ID")?;

        let body = format_bug_body(item);
        let body_hash = hash_content(&body);

        // Check if unchanged
        if let Some(existing_hash) = state.content_hashes.get(&item_id) {
            if *existing_hash == body_hash {
                return Ok(SyncOutcome::Skipped);
            }
        }

        let labels = vec![
            "bug".to_string(),
            "agileplus".to_string(),
            format!("priority:{}", item.priority),
        ];

        let payload = GitHubIssuePayload {
            title: format!("[Bug] {}", item.title),
            body: body.clone(),
            labels,
        };

        let outcome = if let Some(&issue_number) = state.issue_mappings.get(&item_id) {
            // Conflict check: fetch remote and compare hashes
            if let Ok(remote) = self.client.get_issue(issue_number).await {
                if let Some(ref remote_body) = remote.body {
                    let remote_hash = hash_content(remote_body);
                    if let Some(our_hash) = state.content_hashes.get(&item_id) {
                        if remote_hash != *our_hash && body_hash != remote_hash {
                            return Ok(SyncOutcome::Conflict {
                                issue_number,
                                reason: "Remote issue body was modified externally".to_string(),
                            });
                        }
                    }
                }
            }

            let resp = self.client.update_issue(issue_number, &payload).await?;
            SyncOutcome::Updated(resp.number)
        } else {
            let resp = self.client.create_issue(&payload).await?;
            state.issue_mappings.insert(item_id, resp.number);
            SyncOutcome::Created(resp.number)
        };

        state.content_hashes.insert(item_id, body_hash);
        state.last_synced_at = Some(Utc::now());

        Ok(outcome)
    }

    /// Poll GitHub for status changes and return items that changed.
    pub async fn poll_status_changes(
        &self,
        state: &GitHubSyncState,
    ) -> Result<Vec<(i64, String)>> {
        let mut changes = Vec::new();

        for (&item_id, &issue_number) in &state.issue_mappings {
            match self.client.get_issue(issue_number).await {
                Ok(issue) => {
                    changes.push((item_id, issue.state));
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to poll GitHub issue #{}: {}",
                        issue_number,
                        e
                    );
                }
            }
        }

        Ok(changes)
    }
}

/// Format a backlog item as a structured GitHub issue body.
fn format_bug_body(item: &BacklogItem) -> String {
    let mut body = String::new();
    body.push_str("## Description\n\n");
    body.push_str(&item.description);
    body.push_str("\n\n");
    body.push_str("## Metadata\n\n");
    body.push_str(&format!("- **Priority**: {}\n", item.priority));
    body.push_str(&format!("- **Status**: {}\n", item.status));
    body.push_str(&format!("- **Source**: {}\n", item.source));
    if let Some(ref slug) = item.feature_slug {
        body.push_str(&format!("- **Feature**: {slug}\n"));
    }
    body.push_str(&format!("- **Created**: {}\n", item.created_at.to_rfc3339()));
    body.push_str("\n---\n*Synced by AgilePlus*\n");
    body
}

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_triage::{BacklogPriority, BacklogStatus, Intent};

    fn sample_bug() -> BacklogItem {
        BacklogItem {
            id: Some(1),
            title: "Login crash".to_string(),
            description: "App crashes when clicking login".to_string(),
            intent: Intent::Bug,
            priority: BacklogPriority::High,
            status: BacklogStatus::New,
            source: "user-report".to_string(),
            feature_slug: Some("auth".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn bug_body_format() {
        let item = sample_bug();
        let body = format_bug_body(&item);
        assert!(body.contains("## Description"));
        assert!(body.contains("Login crash") || body.contains("App crashes"));
        assert!(body.contains("Priority"));
        assert!(body.contains("high"));
        assert!(body.contains("Feature**: auth"));
        assert!(body.contains("Synced by AgilePlus"));
    }

    #[test]
    fn hash_deterministic() {
        assert_eq!(hash_content("abc"), hash_content("abc"));
        assert_ne!(hash_content("abc"), hash_content("def"));
    }

    #[test]
    fn sync_state_roundtrip() {
        let mut state = GitHubSyncState::default();
        state.issue_mappings.insert(1, 42);
        state.content_hashes.insert(1, "abc123".to_string());

        let json = serde_json::to_string(&state).unwrap();
        let restored: GitHubSyncState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.issue_mappings[&1], 42);
    }

    #[test]
    fn skipped_when_unchanged() {
        // Test that content hash matching would signal skip
        let body = format_bug_body(&sample_bug());
        let h1 = hash_content(&body);
        let h2 = hash_content(&body);
        assert_eq!(h1, h2); // Same content → same hash → skip
    }
}
