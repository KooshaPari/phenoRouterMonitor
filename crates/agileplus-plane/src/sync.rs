//! Plane.so sync logic with idempotency and conflict detection.
//!
//! Traceability: WP18-T105, T106, T107

use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::client::{PlaneClient, PlaneIssue};

/// Sync state for tracking idempotent operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub feature_slug: String,
    pub plane_issue_id: Option<String>,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub content_hash: Option<String>,
    /// Maps WP ID → Plane sub-issue ID
    pub wp_mappings: HashMap<String, String>,
}

impl SyncState {
    pub fn new(feature_slug: String) -> Self {
        Self {
            feature_slug,
            plane_issue_id: None,
            last_synced_at: None,
            content_hash: None,
            wp_mappings: HashMap::new(),
        }
    }
}

/// Plane.so sync adapter.
#[derive(Debug)]
pub struct PlaneSyncAdapter {
    client: PlaneClient,
}

/// Outcome of a sync operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncOutcome {
    Created(String),
    Updated(String),
    Skipped,
    Conflict(String),
}

impl PlaneSyncAdapter {
    pub fn new(client: PlaneClient) -> Self {
        Self { client }
    }

    /// Sync a feature to Plane.so as an issue.
    pub async fn sync_feature(
        &self,
        state: &mut SyncState,
        title: &str,
        description: &str,
    ) -> Result<SyncOutcome> {
        let content_hash = hash_content(&format!("{title}\n{description}"));

        // Check if already synced and unchanged
        if let Some(ref existing_hash) = state.content_hash {
            if *existing_hash == content_hash {
                tracing::debug!("Feature {} unchanged, skipping sync", state.feature_slug);
                return Ok(SyncOutcome::Skipped);
            }
        }

        let issue = PlaneIssue {
            id: None,
            name: title.to_string(),
            description_html: Some(format!("<p>{description}</p>")),
            state: None,
            priority: Some(2),
            parent: None,
            labels: vec!["agileplus".to_string(), "feature".to_string()],
        };

        let outcome = if let Some(ref issue_id) = state.plane_issue_id {
            // Check for conflicts before update
            if let Ok(remote) = self.client.get_issue(issue_id).await {
                if let Some(ref remote_desc) = remote.description_html {
                    let remote_hash = hash_content(&format!("{}\n{}", remote.name, remote_desc));
                    if let Some(ref our_hash) = state.content_hash {
                        if remote_hash != *our_hash && content_hash != remote_hash {
                            tracing::warn!(
                                "Conflict detected on Plane issue {}: remote was modified",
                                issue_id
                            );
                            return Ok(SyncOutcome::Conflict(issue_id.clone()));
                        }
                    }
                }
            }

            let resp = self.client.update_issue(issue_id, &issue).await?;
            SyncOutcome::Updated(resp.id)
        } else {
            let resp = self.client.create_issue(&issue).await?;
            state.plane_issue_id = Some(resp.id.clone());
            SyncOutcome::Created(resp.id)
        };

        state.content_hash = Some(content_hash);
        state.last_synced_at = Some(Utc::now());

        Ok(outcome)
    }

    /// Sync a work package as a sub-issue under the feature issue.
    pub async fn sync_work_package(
        &self,
        state: &mut SyncState,
        wp_id: &str,
        title: &str,
        description: &str,
    ) -> Result<SyncOutcome> {
        let parent_id = state.plane_issue_id.as_deref().context(
            "cannot sync WP before parent feature is synced",
        )?;

        let issue = PlaneIssue {
            id: None,
            name: format!("[{wp_id}] {title}"),
            description_html: Some(format!("<p>{description}</p>")),
            state: None,
            priority: Some(3),
            parent: Some(parent_id.to_string()),
            labels: vec!["agileplus".to_string(), "work-package".to_string()],
        };

        if let Some(existing_id) = state.wp_mappings.get(wp_id) {
            let resp = self.client.update_issue(existing_id, &issue).await?;
            Ok(SyncOutcome::Updated(resp.id))
        } else {
            let resp = self.client.create_issue(&issue).await?;
            state.wp_mappings.insert(wp_id.to_string(), resp.id.clone());
            Ok(SyncOutcome::Created(resp.id))
        }
    }
}

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_state_new() {
        let state = SyncState::new("test-feature".to_string());
        assert!(state.plane_issue_id.is_none());
        assert!(state.wp_mappings.is_empty());
    }

    #[test]
    fn hash_deterministic() {
        let h1 = hash_content("hello world");
        let h2 = hash_content("hello world");
        assert_eq!(h1, h2);
        assert_ne!(h1, hash_content("different"));
    }

    #[test]
    fn sync_state_serialization() {
        let mut state = SyncState::new("feat".to_string());
        state.plane_issue_id = Some("issue-123".to_string());
        state.wp_mappings.insert("WP01".to_string(), "sub-456".to_string());

        let json = serde_json::to_string(&state).unwrap();
        let restored: SyncState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.plane_issue_id.unwrap(), "issue-123");
        assert_eq!(restored.wp_mappings["WP01"], "sub-456");
    }
}
