//! T049: Inbound Sync — process Plane.so webhook events.
//!
//! Traceability: WP08-T049

use agileplus_domain::domain::state_machine::FeatureState;
use serde::{Deserialize, Serialize};

use crate::{
    content_hash::compute_content_hash,
    state_mapper::PlaneStateMapper,
    webhook::{PlaneInboundEvent, PlaneWebhookIssue},
};

/// Outcome of processing an inbound webhook event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InboundOutcome {
    /// A new issue was auto-imported; contains the Plane issue ID.
    AutoImported { issue_id: String, title: String },
    /// A local entity was updated because the content hash changed.
    Updated {
        issue_id: String,
        new_hash: String,
        new_state: FeatureState,
    },
    /// Content hash unchanged; no action taken.
    Unchanged { issue_id: String },
    /// The remote issue was deleted; entity should be archived.
    Archived { issue_id: String },
    /// The issue is not tracked locally; no action taken.
    NotTracked { issue_id: String },
}

/// Callback trait that inbound sync uses to query/update local state.
///
/// Implementations interact with the local database or in-memory store.
pub trait LocalEntityStore: Send + Sync {
    /// Look up the local content hash for a Plane issue ID.
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
    fn auto_import(&mut self, issue: &PlaneWebhookIssue, state: FeatureState) -> anyhow::Result<()>;
}

/// Inbound sync processor.
pub struct InboundSync {
    mapper: PlaneStateMapper,
    auto_import_enabled: bool,
}

impl InboundSync {
    pub fn new(mapper: PlaneStateMapper, auto_import_enabled: bool) -> Self {
        Self {
            mapper,
            auto_import_enabled,
        }
    }

    /// Process a webhook event and update local state via the store.
    pub fn process<S: LocalEntityStore>(
        &self,
        event: PlaneInboundEvent,
        store: &mut S,
    ) -> anyhow::Result<InboundOutcome> {
        match event {
            PlaneInboundEvent::IssueCreated(issue) => self.handle_create(issue, store),
            PlaneInboundEvent::IssueUpdated(issue) => self.handle_update(issue, store),
            PlaneInboundEvent::IssueDeleted { issue_id } => self.handle_delete(issue_id, store),
        }
    }

    fn handle_create<S: LocalEntityStore>(
        &self,
        issue: PlaneWebhookIssue,
        store: &mut S,
    ) -> anyhow::Result<InboundOutcome> {
        if store.get_content_hash(&issue.id).is_some() {
            // Already tracked; treat as update.
            return self.handle_update(issue, store);
        }

        if self.auto_import_enabled {
            let state = self.mapper.from_plane(
                issue.state.as_deref().unwrap_or("backlog"),
                issue.state.as_deref().unwrap_or(""),
            );
            store.auto_import(&issue, state)?;
            tracing::info!(
                plane_issue_id = issue.id,
                title = issue.name,
                "auto-imported new Plane.so issue"
            );
            Ok(InboundOutcome::AutoImported {
                issue_id: issue.id,
                title: issue.name,
            })
        } else {
            tracing::debug!(
                plane_issue_id = issue.id,
                "issue not tracked; auto-import disabled"
            );
            Ok(InboundOutcome::NotTracked { issue_id: issue.id })
        }
    }

    fn handle_update<S: LocalEntityStore>(
        &self,
        issue: PlaneWebhookIssue,
        store: &mut S,
    ) -> anyhow::Result<InboundOutcome> {
        let existing_hash = match store.get_content_hash(&issue.id) {
            Some(h) => h,
            None => {
                return Ok(InboundOutcome::NotTracked { issue_id: issue.id });
            }
        };

        let state_group = issue.state.as_deref().unwrap_or("backlog");
        let new_state = self.mapper.from_plane(state_group, state_group);
        let new_hash = compute_content_hash(
            &issue.name,
            issue.state.as_deref().unwrap_or(""),
            &new_state.to_string(),
            &issue.labels,
        );

        if new_hash == existing_hash {
            tracing::debug!(plane_issue_id = issue.id, "content hash unchanged; skipping");
            return Ok(InboundOutcome::Unchanged { issue_id: issue.id });
        }

        store.apply_update(&issue.id, new_state, new_hash.clone())?;
        tracing::info!(
            plane_issue_id = issue.id,
            new_state = ?new_state,
            "applied inbound state update from Plane.so"
        );
        Ok(InboundOutcome::Updated {
            issue_id: issue.id,
            new_hash,
            new_state,
        })
    }

    fn handle_delete<S: LocalEntityStore>(
        &self,
        issue_id: String,
        store: &mut S,
    ) -> anyhow::Result<InboundOutcome> {
        if store.get_content_hash(&issue_id).is_none() {
            return Ok(InboundOutcome::NotTracked {
                issue_id: issue_id.clone(),
            });
        }
        store.mark_archived(&issue_id)?;
        tracing::info!(plane_issue_id = issue_id, "archived deleted Plane.so issue");
        Ok(InboundOutcome::Archived { issue_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::webhook::PlaneWebhookIssue;
    use std::collections::HashMap;

    struct MockStore {
        hashes: HashMap<String, String>,
        archived: Vec<String>,
        imported: Vec<String>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                hashes: HashMap::new(),
                archived: Vec::new(),
                imported: Vec::new(),
            }
        }
    }

    impl LocalEntityStore for MockStore {
        fn get_content_hash(&self, id: &str) -> Option<String> {
            self.hashes.get(id).cloned()
        }

        fn apply_update(&mut self, id: &str, _state: FeatureState, hash: String) -> anyhow::Result<()> {
            self.hashes.insert(id.to_string(), hash);
            Ok(())
        }

        fn mark_archived(&mut self, id: &str) -> anyhow::Result<()> {
            self.archived.push(id.to_string());
            Ok(())
        }

        fn auto_import(&mut self, issue: &PlaneWebhookIssue, _state: FeatureState) -> anyhow::Result<()> {
            self.imported.push(issue.id.clone());
            Ok(())
        }
    }

    fn make_issue(id: &str, name: &str, state: Option<&str>) -> PlaneWebhookIssue {
        PlaneWebhookIssue {
            id: id.to_string(),
            name: name.to_string(),
            description_html: None,
            state: state.map(|s| s.to_string()),
            labels: vec![],
            project: None,
        }
    }

    #[test]
    fn auto_import_new_issue() {
        let mapper = PlaneStateMapper::new();
        let processor = InboundSync::new(mapper, true);
        let mut store = MockStore::new();

        let event = PlaneInboundEvent::IssueCreated(make_issue("id1", "New Issue", Some("backlog")));
        let outcome = processor.process(event, &mut store).unwrap();
        assert!(matches!(outcome, InboundOutcome::AutoImported { .. }));
        assert!(store.imported.contains(&"id1".to_string()));
    }

    #[test]
    fn not_tracked_when_auto_import_disabled() {
        let mapper = PlaneStateMapper::new();
        let processor = InboundSync::new(mapper, false);
        let mut store = MockStore::new();

        let event = PlaneInboundEvent::IssueCreated(make_issue("id2", "Issue", Some("started")));
        let outcome = processor.process(event, &mut store).unwrap();
        assert!(matches!(outcome, InboundOutcome::NotTracked { .. }));
    }

    #[test]
    fn update_tracked_issue() {
        let mapper = PlaneStateMapper::new();
        let processor = InboundSync::new(mapper, true);
        let mut store = MockStore::new();
        store.hashes.insert("id3".to_string(), "oldhash".to_string());

        let event = PlaneInboundEvent::IssueUpdated(make_issue("id3", "Updated", Some("started")));
        let outcome = processor.process(event, &mut store).unwrap();
        assert!(matches!(outcome, InboundOutcome::Updated { .. }));
    }

    #[test]
    fn delete_archives_tracked_issue() {
        let mapper = PlaneStateMapper::new();
        let processor = InboundSync::new(mapper, true);
        let mut store = MockStore::new();
        store.hashes.insert("id4".to_string(), "hash".to_string());

        let event = PlaneInboundEvent::IssueDeleted { issue_id: "id4".to_string() };
        let outcome = processor.process(event, &mut store).unwrap();
        assert!(matches!(outcome, InboundOutcome::Archived { .. }));
        assert!(store.archived.contains(&"id4".to_string()));
    }
}
