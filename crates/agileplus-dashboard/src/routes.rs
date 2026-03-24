//! Axum route handlers for the dashboard.  (T077)
//!
//! Pattern: if the request carries `HX-Request: true`, return only the
//! relevant partial; otherwise return the full page layout.

use std::collections::HashMap;
use std::env;

use askama::Template;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use chrono::Utc;

use agileplus_domain::domain::{
    feature::Feature, state_machine::FeatureState, work_package::WpState,
};

use crate::app_state::SharedState;
use crate::templates::{
    AgentActivityPartial, AgentSettingsPage, AgentView, DashboardPage, EventTimelinePartial,
    EventsPage, EvidenceBundleView, FeatureDetailPage, FeatureView, FeaturesPage,
    HealthPanelPartial, HomePage, KanbanPartial, MediaAssetView, PlaneHealthEndpointView,
    PlaneSettingsPage, ProjectSummaryView, ProjectSwitcherPartial, ProjectView, ReportArtifactView,
    ServicesSettingsPage, SettingsPage, WpListPartial, WpView, all_feature_states,
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
fn load_projects(
    store: &crate::app_state::DashboardStore,
) -> (Vec<ProjectView>, Option<ProjectView>) {
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

fn build_project_summaries(store: &crate::app_state::DashboardStore) -> Vec<ProjectSummaryView> {
    store
        .projects
        .iter()
        .map(|project| {
            let (feature_count, active_count, shipped_count) =
                store.feature_counts_for_project(project.id);
            ProjectSummaryView {
                project: ProjectView {
                    id: project.id,
                    slug: project.slug.clone(),
                    name: project.name.clone(),
                    description: project.description.clone(),
                },
                feature_count,
                active_count,
                shipped_count,
            }
        })
        .collect()
}

const DEFAULT_PLANE_API_URL: &str = "https://app.plane.so";
const DEFAULT_PLANE_WEB_URL: &str = "https://app.plane.so";

fn env_or_none(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn parse_bool_env(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(default)
}

fn plane_api_key_hint(api_key: &Option<String>) -> String {
    match api_key {
        Some(key) => match (key.chars().next(), key.chars().rev().next()) {
            (Some(first), Some(last)) => format!("{first}••••••{last}"),
            _ => "Configured".to_string(),
        },
        None => "Not configured".to_string(),
    }
}

fn plane_health_endpoints(
    services: &[crate::app_state::ServiceHealth],
) -> Vec<PlaneHealthEndpointView> {
    services
        .iter()
        .filter(|service| service.name.contains("Plane") || service.name.starts_with("API"))
        .map(|service| PlaneHealthEndpointView {
            name: service.name.clone(),
            healthy: service.healthy,
            degraded: service.degraded,
            latency_ms: service.latency_ms,
            last_check_utc: service
                .last_check
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
        })
        .collect()
}

fn build_feature_events(
    feature: &FeatureView,
    workpackages: &[WpView],
) -> Vec<crate::templates::EventView> {
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    let mut events = vec![crate::templates::EventView {
        id: format!("evt-feature-{}-created", feature.id),
        kind: "system".into(),
        description: format!("Feature '{}' opened in dashboard", feature.slug),
        timestamp: now.clone(),
    }];

    if !workpackages.is_empty() {
        events.push(crate::templates::EventView {
            id: format!("evt-feature-{}-sync", feature.id),
            kind: "agent_action".into(),
            description: format!("{} work package entries synced", workpackages.len()),
            timestamp: now.clone(),
        });

        for wp in workpackages {
            events.push(crate::templates::EventView {
                id: format!("evt-feature-{}-wp-{}", feature.id, wp.id),
                kind: "state_change".into(),
                description: format!("Work-package {} is in state '{}'", wp.title, wp.state),
                timestamp: now.clone(),
            });
        }
    } else {
        events.push(crate::templates::EventView {
            id: format!("evt-feature-{}-no-wp", feature.id),
            kind: "system".into(),
            description: "No work packages linked yet".into(),
            timestamp: now.clone(),
        });
    }

    events
}

fn build_feature_evidence_bundles(
    feature: &FeatureView,
    workpackages: &[WpView],
) -> Vec<EvidenceBundleView> {
    let mut bundles = vec![EvidenceBundleView {
        id: format!("bundle-{id}-summary", id = feature.id),
        fr_id: format!("FR-{id}", id = feature.id),
        evidence_type: "feature_summary".into(),
        wp_id: "dashboard".into(),
        wp_title: feature.title.clone(),
        artifact_path: format!("/artifacts/features/{}.md", feature.slug),
        created_at: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        artifact_ext: "md".into(),
        status: "available".into(),
    }];

    for wp in workpackages {
        bundles.push(EvidenceBundleView {
            id: format!("bundle-{fid}-wp-{wid}", fid = feature.id, wid = wp.id),
            fr_id: format!("FR-{fid}", fid = feature.id),
            evidence_type: "workpackage_artifact".into(),
            wp_id: wp.id.to_string(),
            wp_title: wp.title.clone(),
            artifact_path: format!(
                "/artifacts/wp/{wid}/{slug}.json",
                wid = wp.id,
                slug = feature.slug
            ),
            created_at: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            artifact_ext: "json".into(),
            status: if wp.progress > 0 {
                "accepted"
            } else {
                "generated"
            }
            .into(),
        });
    }

    bundles
}

fn build_feature_media_assets(
    feature: &FeatureView,
    workpackages: &[WpView],
) -> Vec<MediaAssetView> {
    let mut media = vec![MediaAssetView {
        id: format!("media-{id}-cover", id = feature.id),
        source: "dashboard".into(),
        name: format!("{slug}-hero.png", slug = feature.slug),
        kind: "image".into(),
        mime: "image/png".into(),
        url_or_path: format!("/assets/{slug}/cover.png", slug = feature.slug),
        size_bytes: 128_512,
        uploaded_at: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    }];

    for wp in workpackages {
        media.push(MediaAssetView {
            id: format!("media-{fid}-wp-{wid}", fid = feature.id, wid = wp.id),
            source: "agent-work-package".into(),
            name: format!("{slug}-wp-{wid}.png", slug = feature.slug, wid = wp.id),
            kind: "screenshot".into(),
            mime: "image/png".into(),
            url_or_path: format!("/assets/wp/{wid}/coverage.png", wid = wp.id),
            size_bytes: 84_320 + (wp.id as usize * 3_000),
            uploaded_at: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        });
    }

    media
}

fn build_feature_reports(
    feature: &FeatureView,
    workpackages: &[WpView],
) -> Vec<ReportArtifactView> {
    vec![ReportArtifactView {
        id: format!("report-{id}-coverage", id = feature.id),
        name: format!("Feature Coverage Report — {name}", name = feature.title),
        source: "coverage-engine".into(),
        status: "completed".into(),
        generated_at: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        rule_count: 5,
        satisfied_count: if feature.labels.is_empty() {
            2
        } else {
            feature.labels.len() + 2
        },
        compliant: workpackages.len() >= 1,
    }]
}

fn plane_sync_mode() -> String {
    if parse_bool_env("PLANE_SYNC_BIDIRECTIONAL", false) {
        "Bidirectional".to_string()
    } else {
        "One-way".to_string()
    }
}

fn plane_connection_checks(
    api_key: &Option<String>,
    workspace: &Option<String>,
) -> (bool, String, Vec<String>) {
    let mut warnings = Vec::new();
    if api_key.is_none() {
        warnings.push("Missing PLANE_API_KEY; configure a valid Plane API key".to_string());
    }
    if workspace.is_none() {
        warnings.push("Missing PLANE_WORKSPACE; set workspace slug for Plane sync".to_string());
    }

    if warnings.is_empty() {
        (true, "Connected via PLANE_API_KEY".to_string(), warnings)
    } else if warnings.len() == 1 {
        let status = warnings[0].clone();
        (false, status, warnings)
    } else {
        (false, "Plane settings incomplete".to_string(), warnings)
    }
}

fn percentage_coverage(hit: usize, total: usize) -> String {
    if total == 0 {
        return "0/0 (0%)".to_string();
    }
    let ratio = (hit.saturating_mul(100)).saturating_div(total);
    format!("{hit}/{total} ({ratio}%)")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DashboardFilter {
    All,
    Active,
    Blocked,
    Shipped,
}

fn dashboard_filter_from_query(query: &HashMap<String, String>) -> DashboardFilter {
    match query.get("filter").map(|value| value.as_str()) {
        Some("active") => DashboardFilter::Active,
        Some("blocked") => DashboardFilter::Blocked,
        Some("shipped") => DashboardFilter::Shipped,
        _ => DashboardFilter::All,
    }
}

fn feature_matches_filter(
    store: &crate::app_state::DashboardStore,
    feature: &Feature,
    filter: DashboardFilter,
) -> bool {
    let is_blocked = store
        .work_packages
        .get(&feature.id)
        .map(|workpackages| workpackages.iter().any(|wp| wp.state == WpState::Blocked))
        .unwrap_or(false);

    match filter {
        DashboardFilter::All => true,
        DashboardFilter::Active => !matches!(
            feature.state,
            FeatureState::Shipped | FeatureState::Retrospected
        ),
        DashboardFilter::Blocked => is_blocked,
        DashboardFilter::Shipped => matches!(
            feature.state,
            FeatureState::Shipped | FeatureState::Retrospected
        ),
    }
}

fn build_kanban_cards(
    store: &crate::app_state::DashboardStore,
    filter: DashboardFilter,
) -> HashMap<String, Vec<FeatureView>> {
    let states = all_feature_states();
    let mut cards: HashMap<String, Vec<FeatureView>> = HashMap::new();
    for s in &states {
        cards.insert(s.clone(), vec![]);
    }
    // Group active features by state after applying project and sidebar filters.
    for feature in store.features_for_active_project() {
        if !feature_matches_filter(store, feature, filter) {
            continue;
        }
        let state_key = feature.state.to_string();
        let view = FeatureView::from_feature(feature);
        cards.entry(state_key).or_default().push(view);
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

pub async fn root(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let total_features = store.features.len();
    let active_features = store
        .features
        .iter()
        .filter(|feature| {
            !matches!(
                feature.state,
                FeatureState::Shipped | FeatureState::Retrospected
            )
        })
        .count();
    let shipped_features = store
        .features
        .iter()
        .filter(|feature| {
            matches!(
                feature.state,
                FeatureState::Shipped | FeatureState::Retrospected
            )
        })
        .count();
    let projects = build_project_summaries(&store);

    render(HomePage {
        total_features,
        active_features,
        shipped_features,
        projects,
    })
}

pub async fn home(State(state): State<SharedState>) -> Response {
    root(State(state)).await
}

// ── /dashboard ───────────────────────────────────────────────────────────

pub async fn dashboard_page(
    State(state): State<SharedState>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let store = state.read().await;
    let filter = dashboard_filter_from_query(&query);
    let cards = build_kanban_cards(&store, filter);
    let (projects, active_project) = load_projects(&store);
    let active_filter = query.get("filter").cloned().unwrap_or_else(|| "all".into());
    render(DashboardPage {
        kanban_cards: cards,
        health: store.health.clone(),
        projects,
        active_project,
        active_filter,
    })
}

// ── /api/dashboard/kanban ────────────────────────────────────────────────

pub async fn kanban_board(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let store = state.read().await;
    let filter = dashboard_filter_from_query(&query);
    let cards = build_kanban_cards(&store, filter);
    let active_filter = query.get("filter").cloned().unwrap_or_else(|| "all".into());

    if is_htmx(&headers) {
        render(KanbanPartial { cards })
    } else {
        let (projects, active_project) = load_projects(&store);
        render(DashboardPage {
            kanban_cards: cards,
            health: store.health.clone(),
            projects,
            active_project,
            active_filter,
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
    let events = build_feature_events(&feature, &wps);
    let evidence_bundles = build_feature_evidence_bundles(&feature, &wps);
    let media_assets = build_feature_media_assets(&feature, &wps);
    let reports = build_feature_reports(&feature, &wps);

    render(FeatureDetailPage {
        feature,
        feature_id: fid,
        workpackages: wps,
        events,
        evidence_bundles,
        media_assets,
        reports,
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

pub async fn switch_project(State(state): State<SharedState>, Path(id): Path<i64>) -> Response {
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
    let cards = build_kanban_cards(&store, DashboardFilter::All);
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
    let plane_workspace = env_or_none("PLANE_WORKSPACE");
    let project_slug = env_or_none("PLANE_PROJECT").unwrap_or_else(|| "not configured".to_string());
    let plane_api_key = env_or_none("PLANE_API_KEY");
    let plane_api_url =
        env_or_none("PLANE_API_URL").unwrap_or_else(|| DEFAULT_PLANE_API_URL.to_string());
    let plane_web_url =
        env_or_none("PLANE_WEB_URL").unwrap_or_else(|| DEFAULT_PLANE_WEB_URL.to_string());
    let (connected, connection_status, mut config_warnings) =
        plane_connection_checks(&plane_api_key, &plane_workspace);

    let plane_health_endpoints = plane_health_endpoints(&store.health);
    let plane_health_healthy = plane_health_endpoints
        .iter()
        .all(|endpoint| endpoint.healthy && !endpoint.degraded);
    let plane_api_latency_ms = plane_health_endpoints
        .iter()
        .find(|endpoint| endpoint.name == "Plane API")
        .and_then(|endpoint| endpoint.latency_ms);

    if !connected {
        config_warnings
            .push("Plane sync disabled until required settings are provided".to_string());
    }

    if !plane_health_healthy {
        config_warnings.push("Plane API health check is not healthy".to_string());
    }

    let mapped_features = store
        .features
        .iter()
        .filter(|feature| feature.plane_issue_id.is_some())
        .count();
    let total_features = store.features.len();
    let mapped_work_packages = store
        .work_packages
        .values()
        .flatten()
        .filter(|wp| wp.plane_sub_issue_id.is_some())
        .count();
    let total_work_packages: usize = store.work_packages.values().map(Vec::len).sum();

    let connection_status_configured = !connection_status.is_empty();

    render(PlaneSettingsPage {
        workspace_name: plane_workspace
            .clone()
            .unwrap_or_else(|| "Not configured".to_string()),
        workspace_slug: plane_workspace.unwrap_or_else(|| "not configured".to_string()),
        project_slug,
        plane_api_url: plane_api_url.trim_end_matches('/').to_string(),
        plane_web_url: plane_web_url.trim_end_matches('/').to_string(),
        plane_api_url_set: !plane_api_url.trim_end_matches('/').is_empty(),
        plane_web_url_set: !plane_web_url.trim_end_matches('/').is_empty(),
        plane_api_key_hint: plane_api_key_hint(&plane_api_key),
        plane_api_key_set: plane_api_key.is_some(),
        sync_enabled: connected,
        sync_mode: plane_sync_mode(),
        connected,
        connection_status: connection_status.clone(),
        connection_status_configured,
        plane_service_healthy: plane_health_healthy,
        plane_api_latency_ms,
        plane_health_endpoints,
        mapped_features_coverage: percentage_coverage(mapped_features, total_features),
        mapped_work_packages_coverage: percentage_coverage(
            mapped_work_packages,
            total_work_packages,
        ),
        mapped_features,
        mapped_work_packages,
        config_warnings,
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
        .route("/home", get(home))
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
        .route(
            "/api/dashboard/projects/{id}/activate",
            post(switch_project),
        )
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
            }],
        };
        let html = tpl.render().expect("template renders");
        assert!(html.contains("test-agent"));
        assert!(html.contains("running"));
    }
}
