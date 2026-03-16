//! Axum route handlers for the dashboard.  (T077)
//!
//! Pattern: if the request carries `HX-Request: true`, return only the
//! relevant partial; otherwise return the full page layout.

use std::collections::HashMap;

use askama::Template;
use axum::{
    Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};

use crate::app_state::SharedState;
use crate::templates::{
    AgentActivityPartial, AgentSettingsPage, AgentView, DashboardPage, EventTimelinePartial,
    EventsPage, FeatureDetailPage, FeatureView, FeaturesPage, HealthPanelPartial, KanbanPartial,
    PlaneSettingsPage, ProjectSwitcherPartial, ProjectView, ServicesSettingsPage, SettingsPage,
    WpListPartial, WpView, all_feature_states,
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
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template error: {e}"),
        )
            .into_response(),
    }
}

/// Build the project list and active project from the store.
fn load_projects(store: &crate::app_state::DashboardStore) -> (Vec<ProjectView>, Option<ProjectView>) {
    let projects: Vec<ProjectView> = store
        .projects
        .iter()
        .map(|p| ProjectView {
            id: p.id,
            slug: p.slug.clone(),
            name: p.name.clone(),
            description: p.description.clone(),
        })
        .collect();
    let active_project = store.active_project().map(|p| ProjectView {
        id: p.id,
        slug: p.slug.clone(),
        name: p.name.clone(),
        description: p.description.clone(),
    });
    (projects, active_project)
}

fn build_kanban_cards(
    store: &crate::app_state::DashboardStore,
) -> HashMap<String, Vec<FeatureView>> {
    let states = all_feature_states();
    let active_features = store.features_for_active_project();
    let mut cards: HashMap<String, Vec<FeatureView>> = HashMap::new();
    for s in &states {
        cards.insert(s.clone(), vec![]);
    }
    // Group active features by state
    for feature in active_features {
        let state_key = feature.state.to_string();
        let view = FeatureView::from_feature(feature);
        cards.entry(state_key).or_insert_with(Vec::new).push(view);
    }
    cards
}

fn sample_events() -> Vec<crate::templates::EventView> {
    vec![
        crate::templates::EventView {
            id: "evt-1".into(),
            kind: "system".into(),
            description: "Dashboard booted with native Plane surface".into(),
            timestamp: "just now".into(),
        },
        crate::templates::EventView {
            id: "evt-2".into(),
            kind: "agent_action".into(),
            description: "Planner synced feature ownership metadata".into(),
            timestamp: "2m ago".into(),
        },
        crate::templates::EventView {
            id: "evt-3".into(),
            kind: "state_change".into(),
            description: "Feature moved from researched to planned".into(),
            timestamp: "9m ago".into(),
        },
    ]
}

pub async fn root() -> Redirect {
    Redirect::to("/dashboard")
}

// ── /dashboard ───────────────────────────────────────────────────────────

pub async fn dashboard_page(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let cards = build_kanban_cards(&store);
    let (projects, active_project) = load_projects(&store);
    render(DashboardPage {
        kanban_cards: cards,
        health: store.health.clone(),
        projects,
        active_project,
    })
}

// ── /api/dashboard/kanban ────────────────────────────────────────────────

pub async fn kanban_board(State(state): State<SharedState>, headers: HeaderMap) -> Response {
    let store = state.read().await;
    let cards = build_kanban_cards(&store);

    if is_htmx(&headers) {
        render(KanbanPartial { cards })
    } else {
        let (projects, active_project) = load_projects(&store);
        render(DashboardPage {
            kanban_cards: cards,
            health: store.health.clone(),
            projects,
            active_project,
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

    render(FeatureDetailPage {
        feature,
        feature_id: fid,
        workpackages: wps,
        events: vec![],
    })
}

// ── /api/dashboard/features/:id/work-packages ────────────────────────────

pub async fn wp_list(State(state): State<SharedState>, Path(id): Path<i64>) -> Response {
    let store = state.read().await;
    let wps: Vec<WpView> = store
        .work_packages
        .get(&id)
        .map(|v| v.iter().map(WpView::from_wp).collect())
        .unwrap_or_default();
    render(WpListPartial {
        feature_id: id,
        workpackages: wps,
    })
}

// ── /api/dashboard/health ────────────────────────────────────────────────

pub async fn health_panel(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    render(HealthPanelPartial {
        services: store.health.clone(),
    })
}

// ── /api/dashboard/events ────────────────────────────────────────────────

pub async fn event_timeline(State(state): State<SharedState>) -> Response {
    let _ = state.read().await;
    render(EventTimelinePartial {
        feature_id: 0,
        events: vec![],
    })
}

// ── /api/dashboard/agents ────────────────────────────────────────────────

pub async fn agent_activity(_state: State<SharedState>) -> Response {
    // In production this would query the agent registry / NATS subjects.
    // Return a placeholder list for now.
    let agents: Vec<AgentView> = vec![
        AgentView {
            name: "spec-agent".into(),
            status: "idle".into(),
            current_task: String::new(),
            last_action: "2m ago".into(),
        },
        AgentView {
            name: "impl-agent".into(),
            status: "running".into(),
            current_task: "WP13 implementation".into(),
            last_action: "just now".into(),
        },
    ];
    render(AgentActivityPartial { agents })
}

// ── /api/dashboard/projects ──────────────────────────────────────────

pub async fn project_switcher(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let projects: Vec<ProjectView> = store
        .projects
        .iter()
        .map(|p| ProjectView {
            id: p.id,
            slug: p.slug.clone(),
            name: p.name.clone(),
            description: p.description.clone(),
        })
        .collect();
    render(ProjectSwitcherPartial {
        projects,
        active_id: store.active_project_id,
    })
}

// ── /api/dashboard/projects/:id/activate ─────────────────────────────

pub async fn switch_project(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Response {
    {
        let mut store = state.write().await;
        if id == 0 {
            // id=0 means "All Projects" -- clear the filter.
            store.active_project_id = None;
        } else if store.projects.iter().any(|p| p.id == id) {
            store.active_project_id = Some(id);
        } else {
            return (StatusCode::NOT_FOUND, "Project not found").into_response();
        }
    }

    // Reload the kanban board with the updated project filter.
    let store = state.read().await;
    let cards = build_kanban_cards(&store);
    render(KanbanPartial { cards })
}

// ── /settings ────────────────────────────────────────────────────────────

pub async fn settings_page() -> Response {
    render(SettingsPage)
}

// ── /features ────────────────────────────────────────────────────────────

pub async fn features_page(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let features = store
        .features
        .iter()
        .map(FeatureView::from_feature)
        .collect::<Vec<_>>();
    render(FeaturesPage { features })
}

// ── /events ──────────────────────────────────────────────────────────────

pub async fn events_page() -> Response {
    render(EventsPage {
        events: sample_events(),
    })
}

// ── /settings/* ──────────────────────────────────────────────────────────

pub async fn plane_settings_page(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let mapped_features = store
        .features
        .iter()
        .filter(|feature| feature.plane_issue_id.is_some())
        .count();
    let mapped_work_packages = store
        .work_packages
        .values()
        .flatten()
        .filter(|wp| wp.plane_sub_issue_id.is_some())
        .count();

    render(PlaneSettingsPage {
        workspace_name: "AgilePlus Core Workspace".into(),
        plane_api_url: std::env::var("PLANE_API_URL")
            .unwrap_or_else(|_| "https://app.plane.so".into()),
        sync_enabled: true,
        connected: std::env::var("PLANE_API_KEY").is_ok(),
        mapped_features,
        mapped_work_packages,
    })
}

pub async fn agent_settings_page() -> Response {
    render(AgentSettingsPage {
        agent_pool_size: 6,
        retry_budget: 3,
        dispatch_mode: "balanced".into(),
    })
}

pub async fn services_settings_page(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    render(ServicesSettingsPage {
        services: store.health.clone(),
    })
}

// ── /api/time ────────────────────────────────────────────────────────────

pub async fn time_footer() -> Html<String> {
    Html(
        chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
    )
}

pub async fn stream_placeholder() -> StatusCode {
    StatusCode::NO_CONTENT
}

// ── Router builder ───────────────────────────────────────────────────────

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/dashboard", get(dashboard_page))
        .route("/features", get(features_page))
        .route("/events", get(events_page))
        .route("/settings", get(settings_page))
        .route("/settings/plane", get(plane_settings_page))
        .route("/settings/agents", get(agent_settings_page))
        .route("/settings/services", get(services_settings_page))
        .route("/api/time", get(time_footer))
        .route("/api/stream", get(stream_placeholder))
        .route("/api/dashboard/kanban", get(kanban_board))
        .route("/api/dashboard/features/{id}", get(feature_detail))
        .route("/api/dashboard/features/{id}/work-packages", get(wp_list))
        .route("/api/dashboard/health", get(health_panel))
        .route("/api/dashboard/events", get(event_timeline))
        .route("/api/dashboard/agents", get(agent_activity))
        .route("/api/dashboard/projects", get(project_switcher))
        .route("/api/dashboard/projects/{id}/activate", post(switch_project))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::{DashboardStore, default_health};
    use crate::templates::{AgentActivityPartial, AgentView, EventTimelinePartial};
    use std::sync::Arc;
    use tokio::sync::RwLock;

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
        let response = plane_settings_page(State(state)).await;
        let body = response.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let html = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(html.contains("Plane Native Surface"));
        assert!(html.contains("AgilePlus Core Workspace"));
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
            }],
        };
        let html = tpl.render().expect("template renders");
        assert!(html.contains("test-agent"));
        assert!(html.contains("running"));
    }
}
