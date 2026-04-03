use std::collections::HashMap;

use agileplus_domain::domain::state_machine::FeatureState;

use super::{InboundOutcome, InboundSync, LocalEntityStore};
use crate::{
    state_mapper::PlaneStateMapper,
    webhook::{PlaneInboundEvent, PlaneWebhookCycle, PlaneWebhookIssue, PlaneWebhookModule},
};

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

    fn auto_import(
        &mut self,
        webhook_issue: &PlaneWebhookIssue,
        _state: FeatureState,
    ) -> anyhow::Result<()> {
        self.imported.push(webhook_issue.id.clone());
        Ok(())
    }
}

fn make_webhook_issue(id: &str, name: &str, state: Option<&str>) -> PlaneWebhookIssue {
    PlaneWebhookIssue {
        id: id.to_string(),
        name: name.to_string(),
        description_html: None,
        state: state.map(|value| value.to_string()),
        labels: vec![],
        project: None,
    }
}

#[test]
fn auto_import_new_webhook_issue() {
    let processor = InboundSync::new(PlaneStateMapper::new(), true);
    let mut store = MockStore::new();

    let event =
        PlaneInboundEvent::IssueCreated(make_webhook_issue("id1", "New Issue", Some("backlog")));
    let outcome = processor.process(event, &mut store).unwrap();

    assert!(matches!(outcome, InboundOutcome::AutoImported { .. }));
    assert!(store.imported.contains(&"id1".to_string()));
}

#[test]
fn not_tracked_when_auto_import_disabled() {
    let processor = InboundSync::new(PlaneStateMapper::new(), false);
    let mut store = MockStore::new();

    let event =
        PlaneInboundEvent::IssueCreated(make_webhook_issue("id2", "Issue", Some("started")));
    let outcome = processor.process(event, &mut store).unwrap();

    assert!(matches!(outcome, InboundOutcome::NotTracked { .. }));
}

#[test]
fn not_tracked_for_module_events() {
    let processor = InboundSync::new(PlaneStateMapper::new(), false);
    let mut store = MockStore::new();

    let event = PlaneInboundEvent::ModuleUpdated(PlaneWebhookModule {
        id: "mod-1".to_string(),
        name: "Module".to_string(),
        description: None,
    });
    let outcome = processor.process(event, &mut store).unwrap();

    match outcome {
        InboundOutcome::NotTracked { issue_id } => assert_eq!(issue_id, "mod-1"),
        _ => panic!("expected NotTracked"),
    }
}

#[test]
fn not_tracked_for_cycle_events() {
    let processor = InboundSync::new(PlaneStateMapper::new(), false);
    let mut store = MockStore::new();

    let event = PlaneInboundEvent::CycleUpdated(PlaneWebhookCycle {
        id: "cycle-1".to_string(),
        name: "Cycle".to_string(),
        start_date: None,
        end_date: None,
    });
    let outcome = processor.process(event, &mut store).unwrap();

    match outcome {
        InboundOutcome::NotTracked { issue_id } => assert_eq!(issue_id, "cycle-1"),
        _ => panic!("expected NotTracked"),
    }
}

#[test]
fn update_tracked_webhook_issue() {
    let processor = InboundSync::new(PlaneStateMapper::new(), true);
    let mut store = MockStore::new();
    store
        .hashes
        .insert("id3".to_string(), "oldhash".to_string());

    let event =
        PlaneInboundEvent::IssueUpdated(make_webhook_issue("id3", "Updated", Some("started")));
    let outcome = processor.process(event, &mut store).unwrap();

    assert!(matches!(outcome, InboundOutcome::Updated { .. }));
}

#[test]
fn delete_archives_tracked_webhook_issue() {
    let processor = InboundSync::new(PlaneStateMapper::new(), true);
    let mut store = MockStore::new();
    store.hashes.insert("id4".to_string(), "hash".to_string());

    let event = PlaneInboundEvent::IssueDeleted {
        issue_id: "id4".to_string(),
    };
    let outcome = processor.process(event, &mut store).unwrap();

    assert!(matches!(outcome, InboundOutcome::Archived { .. }));
    assert!(store.archived.contains(&"id4".to_string()));
}
