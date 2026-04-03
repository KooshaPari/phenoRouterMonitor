use agileplus_domain::domain::state_machine::FeatureState;
use serde::{Deserialize, Serialize};

use crate::webhook::PlaneWebhookIssue;

/// Outcome of processing an inbound webhook event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InboundOutcome {
    /// A new work item was auto-imported; contains the Plane work item ID.
    AutoImported { issue_id: String, title: String },
    /// A local entity was updated because the content hash changed.
    Updated {
        issue_id: String,
        new_hash: String,
        new_state: FeatureState,
    },
    /// Content hash unchanged; no action taken.
    Unchanged { issue_id: String },
    /// The remote work item was deleted; entity should be archived.
    Archived { issue_id: String },
    /// The work item is not tracked locally; no action taken.
    NotTracked { issue_id: String },
}

/// Callback trait that inbound sync uses to query/update local state.
///
/// Implementations interact with the local database or in-memory store.
pub trait LocalEntityStore: Send + Sync {
    /// Look up the local content hash for a Plane work item ID.
    /// Returns `None` if the entity is not tracked.
    fn get_content_hash(&self, plane_issue_id: &str) -> Option<String>;

    /// Update the local entity state and content hash.
    fn apply_update(
        &mut self,
        plane_issue_id: &str,
        new_state: FeatureState,
        new_hash: String,
    ) -> anyhow::Result<()>;

    /// Mark the entity as archived/deleted.
    fn mark_archived(&mut self, plane_issue_id: &str) -> anyhow::Result<()>;

    /// Record a new auto-imported entity.
    fn auto_import(
        &mut self,
        webhook_issue: &PlaneWebhookIssue,
        state: FeatureState,
    ) -> anyhow::Result<()>;
}
