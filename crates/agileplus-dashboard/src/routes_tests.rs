use std::collections::HashMap;
use std::sync::Arc;

use askama::Template;
use axum::{body::to_bytes, extract::State};
use tokio::sync::RwLock;

use crate::app_state::{DashboardStore, SharedState, default_health};
use crate::module_cycle::{cycles_page, modules_page};
use crate::routes::{plane_settings_page, root};
use crate::templates::{
    AgentActivityPartial, AgentView, EventTimelinePartial, FeatureView, HealthPanelPartial,
    KanbanPartial, WpListPartial, all_feature_states,
};

fn make_state() -> SharedState {
    let mut store = DashboardStore::seeded();
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
    let response = plane_settings_page(State(state)).await;
    let body = response.into_body();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("Native Plane Views"));
    assert!(html.contains("Browse Synced Features"));
}

#[tokio::test]
async fn home_page_renders() {
    let state = make_state();
    let response = root(State(state)).await;
    let body = response.into_body();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("Multi-Project Control Surface"));
    assert!(html.contains("Projects"));
}

#[tokio::test]
async fn modules_page_renders() {
    let state = make_state();
    let body = modules_page(State(state)).await.into_body();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("Module Ownership"));
    assert!(html.contains("Spec-Driven Development Engine"));
}

#[tokio::test]
async fn cycles_page_renders() {
    let state = make_state();
    let body = cycles_page(State(state)).await.into_body();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains("Delivery Cycles"));
    assert!(html.contains("Foundation Sprint"));
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
            pid: Some(9999),
            started_at: None,
            worktree: String::new(),
            worktree_label: String::new(),
            is_live: true,
        }],
    };
    let html = tpl.render().expect("template renders");
    assert!(html.contains("test-agent"));
    assert!(html.contains("running"));
}
