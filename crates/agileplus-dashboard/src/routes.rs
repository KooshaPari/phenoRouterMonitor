//! Axum route handlers for the dashboard.  (T077)
//!
//! Pattern: if the request carries `HX-Request: true`, return only the
//! relevant partial; otherwise return the full page layout.

use std::collections::HashMap;

use askama::Template;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};

use crate::app_state::SharedState;
use crate::templates::{
    all_feature_states, DashboardPage, EventTimelinePartial, FeatureDetailPage,
    FeatureView, HealthPanelPartial, KanbanPartial, SettingsPage, WpListPartial, WpView,
};

/// Returns `true` if the `HX-Request` header is present and truthy.
fn is_htmx(headers: &HeaderMap) -> bool {
    headers
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "true")
        .unwrap_or(false)
}

fn render<T: Template>(tpl: T) -> Response {
    match tpl.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Template error: {e}")).into_response(),
    }
}

fn build_kanban_cards(store: &crate::app_state::DashboardStore) -> HashMap<String, Vec<FeatureView>> {
    let states = all_feature_states();
    let by_state = store.features_by_state();
    let mut cards: HashMap<String, Vec<FeatureView>> = HashMap::new();
    for s in &states {
        cards.insert(s.clone(), vec![]);
    }
    for (state_key, features) in &by_state {
        let views: Vec<FeatureView> = features.iter().map(|f| FeatureView::from_feature(f)).collect();
        cards.insert(state_key.to_string(), views);
    }
    cards
}

// ── /dashboard ───────────────────────────────────────────────────────────

pub async fn dashboard_page(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let cards = build_kanban_cards(&store);
    render(DashboardPage {
        kanban_cards: cards,
        health: store.health.clone(),
    })
}

// ── /api/dashboard/kanban ────────────────────────────────────────────────

pub async fn kanban_board(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Response {
    let store = state.read().await;
    let cards = build_kanban_cards(&store);

    if is_htmx(&headers) {
        render(KanbanPartial { cards })
    } else {
        render(DashboardPage {
            kanban_cards: cards,
            health: store.health.clone(),
        })
    }
}

// ── /api/dashboard/features/:id ─────────────────────────────────────────

pub async fn feature_detail(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
    _headers: HeaderMap,
) -> Response {
    let store = state.read().await;
    let feature = match store.features.iter().find(|f| f.id == id) {
        Some(f) => FeatureView::from_feature(f),
        None => return (StatusCode::NOT_FOUND, "Feature not found").into_response(),
    };
    let fid = feature.id;
    let wps: Vec<WpView> = store
        .work_packages
        .get(&id)
        .map(|v| v.iter().map(WpView::from_wp).collect())
        .unwrap_or_default();

    render(FeatureDetailPage { feature, feature_id: fid, workpackages: wps, events: vec![] })
}

// ── /api/dashboard/features/:id/work-packages ────────────────────────────

pub async fn wp_list(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Response {
    let store = state.read().await;
    let wps: Vec<WpView> = store
        .work_packages
        .get(&id)
        .map(|v| v.iter().map(WpView::from_wp).collect())
        .unwrap_or_default();
    render(WpListPartial { feature_id: id, workpackages: wps })
}

// ── /api/dashboard/health ────────────────────────────────────────────────

pub async fn health_panel(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    render(HealthPanelPartial { services: store.health.clone() })
}

// ── /api/dashboard/events ────────────────────────────────────────────────

pub async fn event_timeline(State(state): State<SharedState>) -> Response {
    let _ = state.read().await;
    render(EventTimelinePartial {
        feature_id: 0,
        events: vec![],
    })
}

// ── /settings ────────────────────────────────────────────────────────────

pub async fn settings_page() -> Response {
    render(SettingsPage)
}

// ── Router builder ───────────────────────────────────────────────────────

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/dashboard", get(dashboard_page))
        .route("/settings", get(settings_page))
        .route("/api/dashboard/kanban", get(kanban_board))
        .route("/api/dashboard/features/{id}", get(feature_detail))
        .route("/api/dashboard/features/{id}/work-packages", get(wp_list))
        .route("/api/dashboard/health", get(health_panel))
        .route("/api/dashboard/events", get(event_timeline))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use crate::app_state::{DashboardStore, default_health};
    use crate::templates::EventTimelinePartial;

    fn make_state() -> SharedState {
        let mut store = DashboardStore::default();
        store.health = default_health();
        Arc::new(RwLock::new(store))
    }

    #[tokio::test]
    async fn health_panel_renders() {
        let state = make_state();
        let store = state.read().await;
        let tpl = HealthPanelPartial { services: store.health.clone() };
        let html = tpl.render().expect("template renders");
        assert!(html.contains("NATS"));
    }

    #[tokio::test]
    async fn kanban_partial_renders_empty() {
        let states = all_feature_states();
        let cards: HashMap<String, Vec<FeatureView>> = states.iter().map(|s| (s.clone(), vec![])).collect();
        let tpl = KanbanPartial { cards };
        let html = tpl.render().expect("template renders");
        assert!(html.contains("kanban-board"));
    }

    #[tokio::test]
    async fn wp_list_renders_empty() {
        let tpl = WpListPartial { feature_id: 1, workpackages: vec![] };
        let html = tpl.render().expect("template renders");
        assert!(html.contains("Title"));
    }

    #[tokio::test]
    async fn event_timeline_renders_empty() {
        let tpl = EventTimelinePartial { feature_id: 0, events: vec![] };
        let html = tpl.render().expect("template renders");
        assert!(html.contains("event-timeline"));
    }
}
