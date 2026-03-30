use std::collections::HashMap;
use std::sync::Arc;

use askama::Template;
use tokio::sync::RwLock;

use crate::app_state::{DashboardStore, SharedState, default_health};
use crate::templates::{
    AgentActivityPartial, AgentView, EventTimelinePartial, FeatureView, HealthPanelPartial,
    KanbanPartial, WpListPartial, all_feature_states,
};

<<<<<<< HEAD
use super::{plane_settings_page, root};
=======
<<<<<<< HEAD
use super::{plane_settings_page, root};
=======
use super::pages::{plane_settings_page, root};
>>>>>>> origin/main
>>>>>>> origin/main

fn make_state() -> SharedState {
    let mut store = DashboardStore::default();
    store.health = default_health();
    Arc::new(RwLock::new(store))
}

#[tokio::test]
async fn health_panel_renders() {
    let state = make_state();
    let store = state.read().await;
    let tpl = HealthPanelPartial {
        services: store.health.clone(),
    };
    let html = tpl.render().expect("template renders");
    assert!(html.contains("NATS"));
}

#[tokio::test]
async fn plane_settings_page_renders() {
    let state = make_state();
<<<<<<< HEAD
=======
<<<<<<< HEAD
>>>>>>> origin/main
    let response = plane_settings_page(axum::extract::State(state)).await;
    let body = response.into_body();
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("Plane Native Surface"));
<<<<<<< HEAD
=======
=======
    let response: axum::response::Response = plane_settings_page(axum::extract::State(state)).await;
    let body = response.into_body();
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(!html.is_empty());
>>>>>>> origin/main
>>>>>>> origin/main
    assert!(html.contains("Not configured"));
}

#[tokio::test]
async fn kanban_partial_renders_empty() {
    let states = all_feature_states();
    let cards: HashMap<String, Vec<FeatureView>> =
        states.iter().map(|s| (s.clone(), vec![])).collect();
    let tpl = KanbanPartial { cards };
    let html = tpl.render().expect("template renders");
    assert!(html.contains("kanban-board"));
}

#[tokio::test]
async fn wp_list_renders_empty() {
    let tpl = WpListPartial {
        feature_id: 1,
        workpackages: vec![],
    };
    let html = tpl.render().expect("template renders");
    assert!(html.contains("Title"));
}

#[tokio::test]
async fn event_timeline_renders_empty() {
    let tpl = EventTimelinePartial {
        feature_id: 0,
        events: vec![],
    };
    let html = tpl.render().expect("template renders");
    assert!(html.contains("event-timeline"));
}

#[tokio::test]
async fn agent_activity_renders_empty() {
    let tpl = AgentActivityPartial { agents: vec![] };
    let html = tpl.render().expect("template renders");
    assert!(html.contains("agent-activity"));
}

#[tokio::test]
async fn agent_activity_renders_agents() {
    let tpl = AgentActivityPartial {
        agents: vec![AgentView {
            name: "test-agent".into(),
            status: "running".into(),
            current_task: "doing work".into(),
            last_action: "1m ago".into(),
<<<<<<< HEAD
=======
<<<<<<< HEAD
=======
            pid: Some(12345),
            started_at: Some("2024-01-15 10:30:00 UTC".into()),
            worktree: "/path/to/worktree".into(),
            worktree_label: "worktree".into(),
            is_live: true,
>>>>>>> origin/main
>>>>>>> origin/main
        }],
    };
    let html = tpl.render().expect("template renders");
    assert!(html.contains("test-agent"));
    assert!(html.contains("running"));
}

#[tokio::test]
async fn root_renders_home_page() {
    let state = make_state();
<<<<<<< HEAD
    let response = root(axum::extract::State(state)).await;
=======
<<<<<<< HEAD
    let response = root(axum::extract::State(state)).await;
=======
    let response: axum::response::Response = root(axum::extract::State(state)).await;
>>>>>>> origin/main
>>>>>>> origin/main
    let body = response.into_body();
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("AgilePlus"));
}
