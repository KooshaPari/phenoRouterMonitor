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

use agileplus_domain::domain::{
    feature::Feature, state_machine::FeatureState, work_package::WpState,
};

use crate::app_state::{ServiceHealth, SharedState};
use crate::process_detector;
use crate::templates::{
    AgentActivityPartial, AgentSettingsPage, AgentView, CiLinkView, DashboardPage,
    EcosystemProject, EventTimelinePartial, EventsPage, EvidenceBundleView,
    FeatureDetailPage, FeatureEvidencePartial, FeatureView, FeaturesPage, GenerateEvidenceResponse,
    GitCommitView, HealthPanelPartial, HomePage, HubPage, KanbanPartial, MediaAssetView,
    PlaneHealthEndpointView, PlaneSettingsPage, PrLinkView, ProjectSummaryView,
    ProjectSwitcherPartial, ProjectView, ReportArtifactView, ServicesSettingsPage, SettingsPage,
    ToastPartial, WpListPartial, WpView, all_feature_states,
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ── JSON API Response Types ────────────────────────────────────────────────

/// JSON response for GET /api/dashboard/agents (real-time agent detection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub status: String,
    pub current_task: String,
    pub pid: Option<u32>,
    pub started_at: Option<String>,
    pub worktree: String,
    pub uptime: String,
}

/// JSON response for GET /api/dashboard/health (service health status)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub services: Vec<ServiceHealthJson>,
    pub timestamp: String,
    pub all_healthy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthJson {
    pub name: String,
    pub healthy: bool,
    pub degraded: bool,
    pub latency_ms: Option<u64>,
    pub last_check: String,
}

/// JSON response for GET /api/dashboard/features/{id}/evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceGalleryJson {
    pub feature_id: String,
    pub artifacts: Vec<EvidenceArtifactJson>,
    pub generated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceArtifactJson {
    pub id: String,
    pub type_: String,
    pub title: String,
    pub path: String,
    pub url: String,
    pub created_at: String,
}

// ── Configuration Types ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaneConfig {
    pub api_url: String,
    pub api_key: String,
    pub workspace_slug: String,
    pub project_slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub pool_size: usize,
    pub retry_budget: usize,
    pub dispatch_mode: String,
    pub default_provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub endpoint_url: String,
    #[serde(default = "default_service_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub max_retries: Option<u32>,
}

fn default_service_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub theme: String,
    pub log_level: String,
    pub data_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub plane: Option<PlaneConfig>,
    pub agents: Option<AgentConfig>,
    pub services: Option<Vec<ServiceConfig>>,
    pub dashboard: Option<DashboardConfig>,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path();
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config {
                plane: None,
                agents: None,
                services: None,
                dashboard: None,
            })
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        std::env::var("HOME")
            .ok()
            .map(|home| PathBuf::from(home).join(".agileplus/config.toml"))
            .unwrap_or_else(|| PathBuf::from(".agileplus/config.toml"))
    }
}

// ── Form Request Types ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PlaneSettingsForm {
    pub api_url: String,
    pub api_key: String,
    pub workspace_slug: String,
    pub project_slug: String,
}

#[derive(Debug, Deserialize)]
pub struct AgentSettingsForm {
    pub pool_size: usize,
    pub retry_budget: usize,
    pub dispatch_mode: String,
    pub default_provider: String,
}

#[derive(Debug, Deserialize)]
pub struct ServiceSettingsForm {
    pub names: Vec<String>,
    pub endpoint_urls: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DashboardSettingsForm {
    pub theme: String,
    pub log_level: String,
    pub data_directory: String,
}

/// Returns `true` if the `HX-Request` header is present and truthy.
fn is_htmx(headers: &HeaderMap) -> bool {
    headers
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "true")
        .unwrap_or(false)
}

/// Minimal HTML entity escaping for embedding text content in HTML attributes/elements.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Classify a file extension into a broad artifact type for display purposes.
#[allow(dead_code)]
fn artifact_type_for_ext(ext: &str) -> &'static str {
    match ext {
        "lcov" | "coverage" | "cov" => "coverage",
        "xml" | "junit" | "tap" => "test-results",
        "json" | "sarif" => "report",
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => "image",
        "md" | "txt" | "log" => "text",
        _ => "artifact",
    }
}

/// Percent-encode path segments so they are safe to embed in URLs.
///
/// Only encodes characters that are not allowed unencoded in URL path segments:
/// spaces, `#`, `?`, `%`, and `+`.
#[allow(dead_code)]
fn percent_encode_path(path: &str) -> String {
    path.chars()
        .flat_map(|c| match c {
            ' ' => vec!['%', '2', '0'],
            '#' => vec!['%', '2', '3'],
            '?' => vec!['%', '3', 'F'],
            '%' => vec!['%', '2', '5'],
            '+' => vec!['%', '2', 'B'],
            other => vec![other],
        })
        .collect()
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
        Some(key) => match (key.chars().next(), key.chars().next_back()) {
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
        agent_name: None,
        agent_link: None,
        wp_id: None,
        wp_link: None,
        commit_sha: None,
        commit_link: None,
        ci_run_id: None,
        ci_run_link: None,
    }];

    if !workpackages.is_empty() {
        events.push(crate::templates::EventView {
            id: format!("evt-feature-{}-sync", feature.id),
            kind: "agent_action".into(),
            description: format!("{} work package entries synced", workpackages.len()),
            timestamp: now.clone(),
            agent_name: Some("sync-agent".to_string()),
            agent_link: Some("/agents/sync-agent".to_string()),
            wp_id: None,
            wp_link: None,
            commit_sha: Some("7c5b6ef".to_string()),
            commit_link: Some("https://github.com/Phenotype/AgilePlus/commit/7c5b6ef".to_string()),
            ci_run_id: Some("1024".to_string()),
            ci_run_link: Some(
                "https://github.com/Phenotype/AgilePlus/actions/runs/1024".to_string(),
            ),
        });

        for wp in workpackages {
            // agent_link: route to agent detail page when an agent_id is present.
            let agent_link = wp
                .agent_id
                .as_deref()
                .map(|aid| format!("/api/dashboard/agents/{aid}"));

            // wp_link: slug-based URL to the work package detail anchor.
            let wp_link = Some(format!(
                "/features/{}/work-packages/{}",
                feature.slug, wp.id
            ));

            // commit_link: GitHub commit URL when a head commit SHA is present.
            let (commit_sha, commit_link) = match &wp.head_commit {
                Some(sha) => (
                    Some(sha.clone()),
                    Some(format!(
                        "https://github.com/KooshaPari/AgilePlus/commit/{sha}"
                    )),
                ),
                None => (None, None),
            };

            // ci_run_link: derive from pr_url when it is a GitHub PR URL by
            // redirecting to the Actions tab for that repository.
            let ci_run_link = wp.pr_url.as_deref().and_then(|url| {
                // pr_url is typically https://github.com/{owner}/{repo}/pull/{n}
                // Strip the `/pull/{n}` suffix and append `/actions` for the runs view.
                let prefix = url
                    .split("/pull/")
                    .next()
                    .filter(|p| p.starts_with("https://github.com/"))?;
                Some(format!("{prefix}/actions"))
            });

            events.push(crate::templates::EventView {
                id: format!("evt-feature-{}-wp-{}", feature.id, wp.id),
                kind: "state_change".into(),
                description: format!("Work-package {} is in state '{}'", wp.title, wp.state),
                timestamp: now.clone(),
                agent_name: wp.agent_id.clone(),
                agent_link,
                wp_id: Some(wp.id.to_string()),
                wp_link,
                commit_sha,
                commit_link,
                ci_run_id: None,
                ci_run_link,
            });
        }
    } else {
        events.push(crate::templates::EventView {
            id: format!("evt-feature-{}-no-wp", feature.id),
            kind: "system".into(),
            description: "No work packages linked yet".into(),
            timestamp: now.clone(),
            agent_name: None,
            agent_link: None,
            wp_id: None,
            wp_link: None,
            commit_sha: None,
            commit_link: None,
            ci_run_id: None,
            ci_run_link: None,
        });
    }

    events
}

fn build_feature_evidence_bundles(
    feature: &FeatureView,
    workpackages: &[WpView],
) -> Vec<EvidenceBundleView> {
    // Try to load real bundles from disk first.
    let disk_bundles = load_evidence_bundles_from_disk(&feature.id.to_string());
    if !disk_bundles.is_empty() {
        return disk_bundles;
    }

    // Fall back to stub bundles when no disk bundles exist yet.
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
        content_preview: Some("# Feature Summary\n\nThis feature provides...".to_string()),
        is_text_artifact: true,
        is_image_artifact: false,
        download_url: format!("/api/evidence/{}/summary/content", feature.id),
        test_passed: None,
        tests_passed_count: 0,
        tests_failed_count: 0,
        test_summary: None,
        commit_count: 0,
        pr_count: 0,
        ci_links: vec![],
        git_commits: vec![],
        pr_links: vec![],
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
            status: if wp.progress > 0 { "accepted" } else { "generated" }.into(),
            content_preview: Some(r#"{"status":"generated","progress":0}"#.to_string()),
            is_text_artifact: true,
            is_image_artifact: false,
            download_url: format!("/api/evidence/{}/{}/content", feature.id, wp.id),
            test_passed: None,
            tests_passed_count: 0,
            tests_failed_count: 0,
            test_summary: None,
            commit_count: 0,
            pr_count: 0,
            ci_links: vec![],
            git_commits: vec![],
            pr_links: vec![],
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
        compliant: !workpackages.is_empty(),
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
            agent_name: None,
            agent_link: None,
            wp_id: None,
            wp_link: None,
            commit_sha: None,
            commit_link: None,
            ci_run_id: None,
            ci_run_link: None,
        },
        crate::templates::EventView {
            id: "evt-2".into(),
            kind: "agent_action".into(),
            description: "Planner synced feature ownership metadata".into(),
            timestamp: "2m ago".into(),
            agent_name: Some("planner-agent".to_string()),
            agent_link: Some("/agents/planner-agent".to_string()),
            wp_id: None,
            wp_link: None,
            commit_sha: Some("abc1234".to_string()),
            commit_link: Some("https://github.com/example/repo/commit/abc1234".to_string()),
            ci_run_id: None,
            ci_run_link: None,
        },
        crate::templates::EventView {
            id: "evt-3".into(),
            kind: "state_change".into(),
            description: "Feature moved from researched to planned".into(),
            timestamp: "9m ago".into(),
            agent_name: None,
            agent_link: None,
            wp_id: Some("42".to_string()),
            wp_link: Some("/features/1#wp-42".to_string()),
            commit_sha: None,
            commit_link: None,
            ci_run_id: Some("12345678".to_string()),
            ci_run_link: Some("https://github.com/example/repo/actions/runs/12345678".to_string()),
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

// ── /api/dashboard/features/{id}/events ──────────────────────────────────

pub async fn feature_events(
    State(state): State<SharedState>,
    Path(feature_id): Path<i64>,
) -> Response {
    let store = state.read().await;
    let feature = match store.features.iter().find(|f| f.id == feature_id) {
        Some(f) => FeatureView::from_feature(f),
        None => return (StatusCode::NOT_FOUND, "Feature not found").into_response(),
    };
    let wps: Vec<WpView> = store
        .work_packages
        .get(&feature_id)
        .map(|v| v.iter().map(WpView::from_wp).collect())
        .unwrap_or_default();
    let events = build_feature_events(&feature, &wps);

    render(EventTimelinePartial {
        feature_id,
        events,
    })
}

// ── /api/dashboard/features/{id}/media ───────────────────────────────────

pub async fn feature_media(
    State(state): State<SharedState>,
    Path(feature_id): Path<i64>,
) -> Response {
    let store = state.read().await;
    let feature = match store.features.iter().find(|f| f.id == feature_id) {
        Some(f) => FeatureView::from_feature(f),
        None => return (StatusCode::NOT_FOUND, "Feature not found").into_response(),
    };
    let wps: Vec<WpView> = store
        .work_packages
        .get(&feature_id)
        .map(|v| v.iter().map(WpView::from_wp).collect())
        .unwrap_or_default();
    let media = build_feature_media_assets(&feature, &wps);

    // Return media assets as a simple HTML partial
    let html = media
        .iter()
        .map(|m| {
            format!(
                r#"<div class="media-asset border rounded p-3 bg-zinc-800">
                <img src="{}" alt="{}" class="w-full rounded"/>
                <p class="text-xs text-zinc-400 mt-2">{}</p>
              </div>"#,
                m.url_or_path, m.name, m.name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    Html(format!(
        r#"<div class="grid grid-cols-2 gap-3 media-gallery">{}</div>"#,
        html
    ))
    .into_response()
}

// ── /api/dashboard/agents ────────────────────────────────────────────────

pub async fn agent_activity(State(state): State<SharedState>) -> Response {
    let _ = state.read().await;

    // Detect real agent processes
    let detected = process_detector::detect_agents();

    // Convert detected agents to view models
    let agents: Vec<AgentView> = detected
        .into_iter()
        .map(|agent| {
            let uptime = calculate_uptime(&agent.started_at);
            let worktree_label = agent
                .worktree
                .as_deref()
                .and_then(|wt| wt.split('/').next_back())
                .unwrap_or("")
                .to_string();
            let worktree = agent.worktree.unwrap_or_default();
            AgentView {
                name: agent.name,
                status: agent.status.clone(),
                current_task: agent.current_task,
                last_action: uptime,
                pid: Some(agent.pid),
                started_at: agent.started_at,
                worktree,
                worktree_label,
                is_live: agent.status == "running",
            }
        })
        .collect();

    render(AgentActivityPartial { agents })
}

/// Calculate uptime string from the elapsed duration string produced by
/// `process_detector::get_process_start_time` (e.g. "5m", "1h 20m").
fn calculate_uptime(started_at: &Option<String>) -> String {
    match started_at {
        Some(elapsed) => format!("running for {}", elapsed),
        None => "uptime unknown".into(),
    }
}

// ── /api/dashboard/agents (JSON) ─────────────────────────────────────────

/// JSON API: GET /api/dashboard/agents
/// Returns detected agent processes as JSON (polls every 5s from dashboard templates).
pub async fn agents_json(State(_state): State<SharedState>) -> impl IntoResponse {
    // Detect real agent processes
    let detected = process_detector::detect_agents();

    // Convert detected agents to JSON response
    let agents: Vec<AgentInfo> = detected
        .into_iter()
        .map(|agent| {
            let uptime = calculate_uptime(&agent.started_at);
            AgentInfo {
                name: agent.name,
                status: agent.status.clone(),
                current_task: agent.current_task,
                pid: Some(agent.pid),
                started_at: agent.started_at,
                worktree: agent.worktree.unwrap_or_default(),
                uptime,
            }
        })
        .collect();

    axum::Json(serde_json::json!({
        "agents": agents,
        "count": agents.len(),
        "timestamp": Utc::now().to_rfc3339(),
    }))
}

// ── /api/dashboard/health (JSON) ────────────────────────────────────────

/// JSON API: GET /api/dashboard/health
/// Returns service health status as JSON (polls every 10s from dashboard templates).
pub async fn health_json(State(state): State<SharedState>) -> impl IntoResponse {
    let store = state.read().await;

    let services: Vec<ServiceHealthJson> = store
        .health
        .iter()
        .map(|service| ServiceHealthJson {
            name: service.name.clone(),
            healthy: service.healthy,
            degraded: service.degraded,
            latency_ms: service.latency_ms,
            last_check: service.last_check.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        })
        .collect();

    let all_healthy = services.iter().all(|s| s.healthy && !s.degraded);

    axum::Json(HealthStatus {
        services,
        timestamp: Utc::now().to_rfc3339(),
        all_healthy,
    })
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
    let config = Config::load().unwrap_or(Config {
        plane: None,
        agents: None,
        services: None,
        dashboard: None,
    });

    let agent_config = config.agents.unwrap_or_else(|| AgentConfig {
        pool_size: 6,
        retry_budget: 3,
        dispatch_mode: "balanced".to_string(),
        default_provider: "claude".to_string(),
    });

    render(AgentSettingsPage {
        agent_pool_size: agent_config.pool_size,
        retry_budget: agent_config.retry_budget,
        dispatch_mode: agent_config.dispatch_mode,
        default_provider: agent_config.default_provider,
    })
}

pub async fn services_settings_page(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let config = Config::load().unwrap_or(Config {
        plane: None,
        agents: None,
        services: None,
        dashboard: None,
    });

    let configs: Vec<crate::templates::ServiceConfigView> = config
        .services
        .unwrap_or_default()
        .into_iter()
        .map(|s| crate::templates::ServiceConfigView {
            name: s.name,
            endpoint_url: s.endpoint_url,
        })
        .collect();

    render(ServicesSettingsPage {
        services: store.health.clone(),
        configs,
    })
}

// ── /hub ─────────────────────────────────────────────────────────────────

pub async fn hub_page() -> Response {
    let projects = vec![
        EcosystemProject {
            name: "phenodocs",
            tagline: "Ecosystem docs hub",
            stack: "TypeScript · Vue",
            port: Some(4100),
            github: "https://github.com/KooshaPari/phenodocs",
            category: "docs",
        },
        EcosystemProject {
            name: "AgilePlus",
            tagline: "Spec-driven PM platform",
            stack: "Rust · Tauri",
            port: Some(4101),
            github: "https://github.com/KooshaPari/AgilePlus",
            category: "app",
        },
        EcosystemProject {
            name: "heliosApp",
            tagline: "TypeScript runtime app",
            stack: "TypeScript · Bun",
            port: Some(4102),
            github: "https://github.com/KooshaPari/heliosApp",
            category: "app",
        },
        EcosystemProject {
            name: "thegent",
            tagline: "Agent framework",
            stack: "TypeScript · Python",
            port: Some(4103),
            github: "https://github.com/KooshaPari/thegent",
            category: "lib",
        },
        EcosystemProject {
            name: "bifrost-extensions",
            tagline: "LLM gateway extensions",
            stack: "Go",
            port: Some(4104),
            github: "https://github.com/KooshaPari/bifrost-extensions",
            category: "lib",
        },
        EcosystemProject {
            name: "civ",
            tagline: "CI validation",
            stack: "TypeScript",
            port: Some(4105),
            github: "https://github.com/KooshaPari/civ",
            category: "docs",
        },
        EcosystemProject {
            name: "TraceRTM",
            tagline: "Requirements traceability",
            stack: "Python · Go · TS",
            port: Some(4110),
            github: "https://github.com/KooshaPari/trace",
            category: "app",
        },
        EcosystemProject {
            name: "agentapi-plusplus",
            tagline: "Agent HTTP API",
            stack: "Go",
            port: None,
            github: "https://github.com/KooshaPari/agentapi-plusplus",
            category: "api",
        },
        EcosystemProject {
            name: "cliproxyapi-plusplus",
            tagline: "Multi-provider CLI proxy",
            stack: "Go",
            port: None,
            github: "https://github.com/KooshaPari/cliproxyapi-plusplus",
            category: "api",
        },
    ];
    render(HubPage { projects })
}

// ── /api/time ────────────────────────────────────────────────────────────

pub async fn time_footer() -> Html<String> {
    Html(
        chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
    )
}

// ── /api/evidence ────────────────────────────────────────────────────────

/// Load real evidence bundles from `.agileplus/evidence/<feature_id>/bundle.json`.
fn load_evidence_bundles_from_disk(feature_id: &str) -> Vec<EvidenceBundleView> {
    let bundle_path = PathBuf::from(".agileplus")
        .join("evidence")
        .join(feature_id)
        .join("bundle.json");

    let content = match fs::read_to_string(&bundle_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let val: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    let timestamp = val["timestamp"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    // Parse test_results
    let tr = &val["test_results"];
    let test_passed = tr["passed"].as_bool();
    let tests_passed_count = tr["passed_count"].as_u64().unwrap_or(0) as u32;
    let tests_failed_count = tr["failed_count"].as_u64().unwrap_or(0) as u32;
    let test_summary = tr["summary"].as_str().map(str::to_string);
    let test_output = tr["output_snippet"].as_str().map(str::to_string);

    // Parse git commits
    let git_commits: Vec<GitCommitView> = val["git_log"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|c| GitCommitView {
            short_hash: c["short_hash"].as_str().unwrap_or("").to_string(),
            subject: c["subject"].as_str().unwrap_or("").to_string(),
            date: c["date"].as_str().unwrap_or("").to_string(),
            author: c["author"].as_str().unwrap_or("").to_string(),
            url: c["url"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    // Parse PRs
    let pr_links: Vec<PrLinkView> = val["prs"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|p| PrLinkView {
            number: p["number"].as_u64().unwrap_or(0),
            title: p["title"].as_str().unwrap_or("").to_string(),
            url: p["url"].as_str().unwrap_or("").to_string(),
            state: p["state"].as_str().unwrap_or("").to_lowercase(),
            head_ref: p["headRefName"].as_str().unwrap_or("").to_string(),
            created_at: p["createdAt"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    // Parse CI links
    let ci_links: Vec<CiLinkView> = val["ci_links"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|c| CiLinkView {
            id: c["id"].as_i64().unwrap_or(0),
            title: c["title"].as_str().unwrap_or("").to_string(),
            status: c["status"].as_str().unwrap_or("").to_string(),
            conclusion: c["conclusion"].as_str().unwrap_or("pending").to_string(),
            url: c["url"].as_str().unwrap_or("").to_string(),
            created_at: c["created_at"].as_str().unwrap_or("").to_string(),
        })
        .collect();

    let commit_count = git_commits.len();
    let pr_count = pr_links.len();
    let status = if test_passed.unwrap_or(false) {
        "verified"
    } else {
        "generated"
    };

    vec![EvidenceBundleView {
        id: format!("bundle-{feature_id}-disk"),
        fr_id: format!("FR-{feature_id}"),
        evidence_type: "generated_bundle".into(),
        wp_id: "auto".into(),
        wp_title: format!("Evidence Bundle — {feature_id}"),
        artifact_path: bundle_path.display().to_string(),
        created_at: timestamp,
        artifact_ext: "json".into(),
        status: status.into(),
        content_preview: test_output,
        is_text_artifact: true,
        is_image_artifact: false,
        download_url: format!("/api/features/{feature_id}/evidence/bundle.json"),
        test_passed,
        tests_passed_count,
        tests_failed_count,
        test_summary,
        commit_count,
        pr_count,
        ci_links,
        git_commits,
        pr_links,
    }]
}

pub async fn evidence_content(
    State(_state): State<SharedState>,
    Path((feature_id, artifact_id)): Path<(i64, String)>,
) -> Response {
    // Serve from .agileplus/evidence/<feature_id>/<artifact_id>
    let artifact_path = PathBuf::from(".agileplus")
        .join("evidence")
        .join(feature_id.to_string())
        .join(&artifact_id);

    if let Ok(content) = fs::read_to_string(&artifact_path) {
        let escaped = html_escape(&content);
        return Html(format!(
            "<pre class='text-xs font-mono text-zinc-300 whitespace-pre-wrap'>{escaped}</pre>",
        ))
        .into_response();
    }

    Html(format!(
        "# Evidence Bundle {feature_id}\n\n## Artifact ID: {artifact_id}\n\nNo artifact found at expected path."
    ))
    .into_response()
}

pub async fn evidence_preview(
    State(_state): State<SharedState>,
    Path((feature_id, artifact_id)): Path<(i64, String)>,
) -> Response {
    let artifact_path = PathBuf::from(".agileplus")
        .join("evidence")
        .join(feature_id.to_string())
        .join(&artifact_id);

    let text = fs::read_to_string(&artifact_path)
        .unwrap_or_else(|_| format!("No preview — artifact not found: {artifact_id}"));
    let escaped = html_escape(&text);
    let preview = format!(
        "<div class='p-3 rounded bg-zinc-800 border border-zinc-700'>\
         <pre class='text-xs font-mono text-zinc-300 max-h-48 overflow-y-auto'>{escaped}</pre>\
         </div>"
    );
    Html(preview).into_response()
}

// ── /api/features/{id}/evidence ───────────────────────────────────────────

/// `GET /api/features/{id}/evidence`
/// Returns the evidence gallery partial for the feature.
pub async fn feature_evidence_list(
    State(_state): State<SharedState>,
    Path(feature_id): Path<String>,
) -> Response {
    let bundles = load_evidence_bundles_from_disk(&feature_id);
    let tmpl = FeatureEvidencePartial {
        evidence_bundles: bundles,
    };
    match tmpl.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!("Template render error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// `POST /api/features/{id}/evidence/generate`
/// Runs `scripts/generate-evidence.sh <feature-id>` asynchronously and
/// returns a JSON status response.
pub async fn feature_evidence_generate(
    State(_state): State<SharedState>,
    Path(feature_id): Path<String>,
) -> Response {
    // Locate the script relative to the process working directory.
    let script = PathBuf::from("scripts").join("generate-evidence.sh");

    if !script.exists() {
        return axum::Json(GenerateEvidenceResponse {
            feature_id: feature_id.clone(),
            bundle_path: String::new(),
            status: "error".into(),
            message: "generate-evidence.sh not found — ensure the server is started from the repo root".into(),
        })
        .into_response();
    }

    let bundle_path = format!(".agileplus/evidence/{feature_id}/bundle.json");
    let fid = feature_id.clone();

    // Spawn async so the HTTP response returns immediately.
    tokio::spawn(async move {
        let out = tokio::process::Command::new("bash")
            .arg(&script)
            .arg(&fid)
            .output()
            .await;
        match out {
            Ok(o) if o.status.success() => {
                tracing::info!("Evidence bundle generated for feature {fid}");
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                tracing::warn!("Evidence generation failed for {fid}: {stderr}");
            }
            Err(e) => {
                tracing::error!("Failed to run generate-evidence.sh for {fid}: {e}");
            }
        }
    });

    axum::Json(GenerateEvidenceResponse {
        feature_id,
        bundle_path,
        status: "started".into(),
        message: "Evidence generation started — poll GET /api/features/{id}/evidence for results".into(),
    })
    .into_response()
}

/// `GET /api/dashboard/features/{id}/evidence.json`
/// Returns evidence gallery metadata as JSON for lightbox integration.
pub async fn feature_evidence_json(
    State(_state): State<SharedState>,
    Path(feature_id): Path<String>,
) -> impl IntoResponse {
    let bundles = load_evidence_bundles_from_disk(&feature_id);

    // Extract artifacts from bundles for gallery JSON response
    let artifacts: Vec<EvidenceArtifactJson> = bundles
        .iter()
        .map(|bundle| EvidenceArtifactJson {
            id: bundle.id.clone(),
            type_: bundle.evidence_type.clone(),
            title: bundle.wp_title.clone(),
            path: bundle.artifact_path.clone(),
            url: format!("/api/evidence/{}/{}/preview", feature_id, bundle.id),
            created_at: bundle.created_at.clone(),
        })
        .collect();

    let generated_at = bundles
        .first()
        .map(|b| b.created_at.clone());

    axum::Json(EvidenceGalleryJson {
        feature_id,
        artifacts,
        generated_at,
    })
}

pub async fn stream_placeholder() -> StatusCode {
    StatusCode::NO_CONTENT
}

// ── /api/dashboard/services/:name/restart ────────────────────────────────

const ALLOWED_RESTART_PROGRAMS: [&str; 4] = ["systemctl", "docker", "process-compose", "echo"];

fn is_restart_command_allowed(program: &str) -> bool {
    ALLOWED_RESTART_PROGRAMS.contains(&program)
}

fn validate_restart_command(cmd_line: &str) -> Result<(), String> {
    let mut parts: Vec<&str> = cmd_line.split_whitespace().collect();
    if parts.is_empty() {
        return Err("empty restart command".into());
    }

    let program = parts.remove(0);
    if !is_restart_command_allowed(program) {
        return Err(format!(
            "command '{}' is not in approved restart command registry: {:?}",
            program, ALLOWED_RESTART_PROGRAMS
        ));
    }

    Ok(())
}

fn build_restart_command(cmd_line: &str) -> Result<std::process::Command, String> {
    validate_restart_command(cmd_line)?;

    let mut parts: Vec<&str> = cmd_line.split_whitespace().collect();
    let program = parts.remove(0);

    let mut cmd = std::process::Command::new(program);
    if !parts.is_empty() {
        cmd.args(parts);
    }
    Ok(cmd)
}

pub async fn restart_service(
    State(_state): State<SharedState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let template = std::env::var("AGILEPLUS_SERVICE_RESTART_CMD")
        .unwrap_or_else(|_| "systemctl restart {}".to_string());

    if !template.contains("{}") {
        return axum::Json(serde_json::json!({
            "status": "error",
            "service": name,
            "error": "AGILEPLUS_SERVICE_RESTART_CMD must include '{}' placeholder",
        }));
    }

    let command_str = template.replace("{}", &name);

    let mut command = match build_restart_command(&command_str) {
        Ok(c) => c,
        Err(err) => {
            return axum::Json(serde_json::json!({
                "status": "error",
                "service": name,
                "command": command_str,
                "error": err,
            }));
        }
    };

    match command.output() {
        Ok(output) => {
            let success = output.status.success();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            axum::Json(serde_json::json!({
                "status": if success { "ok" } else { "error" },
                "service": name,
                "command": command_str,
                "stdout": stdout,
                "stderr": stderr,
            }))
        }
        Err(err) => axum::Json(serde_json::json!({
            "status": "error",
            "service": name,
            "command": command_str,
            "error": err.to_string(),
        })),
    }
}

// ── /api/dashboard/services/:name/config  (PATCH) ────────────────────────

#[derive(Debug, Deserialize)]
pub struct ServiceConfigForm {
    pub endpoint_url: Option<String>,
    pub timeout_ms: Option<u64>,
    pub max_retries: Option<u32>,
}

pub async fn patch_service_config(
    Path(name): Path<String>,
    axum::Form(form): axum::Form<ServiceConfigForm>,
) -> impl IntoResponse {
    let mut config = Config::load().unwrap_or(Config {
        plane: None,
        agents: None,
        services: None,
        dashboard: None,
    });

    let services = config.services.get_or_insert_with(Vec::new);
    if let Some(entry) = services.iter_mut().find(|s| s.name == name) {
        if let Some(url) = form.endpoint_url.filter(|u| !u.trim().is_empty()) {
            entry.endpoint_url = url;
        }
    } else if let Some(url) = form.endpoint_url.filter(|u| !u.trim().is_empty()) {
        services.push(ServiceConfig {
            name: name.clone(),
            endpoint_url: url,
            enabled: default_service_enabled(),
            timeout_ms: form.timeout_ms,
            max_retries: form.max_retries,
        });
    }

    match config.save() {
        Ok(_) => render(ToastPartial {
            message: format!("Service '{}' configuration saved", name),
            success: true,
        }),
        Err(e) => render(ToastPartial {
            message: format!("Failed to save: {}", e),
            success: false,
        }),
    }
}

// ── /api/dashboard/services/:name/toggle (POST) ──────────────────────────

#[derive(Debug, Deserialize)]
pub struct ServiceToggleBody {
    pub enabled: Option<bool>,
}

pub async fn toggle_service(
    State(state): State<SharedState>,
    Path(name): Path<String>,
    axum::Json(body): axum::Json<ServiceToggleBody>,
) -> impl IntoResponse {
    let enabled = body.enabled.unwrap_or(true);

    // Persist state in config file
    let mut config = Config::load().unwrap_or(Config {
        plane: None,
        agents: None,
        services: None,
        dashboard: None,
    });

    let services = config.services.get_or_insert_with(Vec::new);
    if let Some(entry) = services.iter_mut().find(|s| s.name == name) {
        entry.enabled = enabled;
    } else {
        services.push(ServiceConfig {
            name: name.clone(),
            endpoint_url: String::new(),
            enabled,
            timeout_ms: None,
            max_retries: None,
        });
    }

    if let Err(err) = config.save() {
        return axum::Json(serde_json::json!({
            "status": "error",
            "service": name,
            "enabled": enabled,
            "error": format!("Failed to save config: {}", err),
        }));
    }

    // Update in-memory health status for UI
    {
        let mut store = state.write().await;
        if let Some(item) = store.health.iter_mut().find(|s| s.name == name) {
            item.healthy = enabled;
            item.degraded = !enabled;
            item.last_check = Utc::now();
        } else {
            store.health.push(ServiceHealth {
                name: name.clone(),
                healthy: enabled,
                degraded: !enabled,
                latency_ms: None,
                last_check: Utc::now(),
            });
        }
    }

    axum::Json(serde_json::json!({
        "status": "ok",
        "service": name,
        "enabled": enabled,
    }))
}

// ── /api/settings/agents/test-connection (POST) ──────────────────────────

#[derive(Debug, Deserialize)]
pub struct AgentTestConnectionForm {
    pub provider: String,
}

pub async fn test_agent_connection(
    axum::Form(form): axum::Form<AgentTestConnectionForm>,
) -> impl IntoResponse {
    // Provider reachability check: validate that required env vars are present.
    let (ok, msg) = match form.provider.as_str() {
        "claude" => {
            let key = env_or_none("ANTHROPIC_API_KEY");
            if key.is_some() {
                (
                    true,
                    "Claude API key detected — connection likely valid".to_string(),
                )
            } else {
                (false, "ANTHROPIC_API_KEY not set".to_string())
            }
        }
        "gemini" => {
            let key = env_or_none("GEMINI_API_KEY").or_else(|| env_or_none("GOOGLE_API_KEY"));
            if key.is_some() {
                (
                    true,
                    "Gemini API key detected — connection likely valid".to_string(),
                )
            } else {
                (false, "GEMINI_API_KEY / GOOGLE_API_KEY not set".to_string())
            }
        }
        "local" => (
            true,
            "Local provider requires no external credentials".to_string(),
        ),
        other => (false, format!("Unknown provider: {}", other)),
    };

    let css = if ok { "text-green-400" } else { "text-red-400" };
    Html(format!(r#"<span class="{}">{}</span>"#, css, msg)).into_response()
}

// ── Router builder ───────────────────────────────────────────────────────

// ── Settings POST Handlers ─────────────────────────────────────────────────

pub async fn save_plane_settings(axum::Form(form): axum::Form<PlaneSettingsForm>) -> Response {
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(_) => Config {
            plane: None,
            agents: None,
            services: None,
            dashboard: None,
        },
    };

    config.plane = Some(PlaneConfig {
        api_url: form.api_url.trim().to_string(),
        api_key: form.api_key.trim().to_string(),
        workspace_slug: form.workspace_slug.trim().to_string(),
        project_slug: form.project_slug.trim().to_string(),
    });

    match config.save() {
        Ok(_) => render(ToastPartial {
            message: "Plane settings saved successfully".to_string(),
            success: true,
        }),
        Err(e) => render(ToastPartial {
            message: format!("Failed to save settings: {}", e),
            success: false,
        }),
    }
}

pub async fn save_agent_settings(axum::Form(form): axum::Form<AgentSettingsForm>) -> Response {
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(_) => Config {
            plane: None,
            agents: None,
            services: None,
            dashboard: None,
        },
    };

    config.agents = Some(AgentConfig {
        pool_size: form.pool_size,
        retry_budget: form.retry_budget,
        dispatch_mode: form.dispatch_mode.trim().to_string(),
        default_provider: form.default_provider.trim().to_string(),
    });

    match config.save() {
        Ok(_) => render(ToastPartial {
            message: "Agent settings saved successfully".to_string(),
            success: true,
        }),
        Err(e) => render(ToastPartial {
            message: format!("Failed to save settings: {}", e),
            success: false,
        }),
    }
}

pub async fn save_dashboard_settings(
    axum::Form(form): axum::Form<DashboardSettingsForm>,
) -> Response {
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(_) => Config {
            plane: None,
            agents: None,
            services: None,
            dashboard: None,
        },
    };

    config.dashboard = Some(DashboardConfig {
        theme: form.theme.trim().to_string(),
        log_level: form.log_level.trim().to_string(),
        data_directory: form.data_directory.trim().to_string(),
    });

    match config.save() {
        Ok(_) => render(ToastPartial {
            message: "Dashboard settings saved successfully".to_string(),
            success: true,
        }),
        Err(e) => render(ToastPartial {
            message: format!("Failed to save settings: {}", e),
            success: false,
        }),
    }
}

pub async fn save_services_settings(axum::Form(form): axum::Form<ServiceSettingsForm>) -> Response {
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(_) => Config {
            plane: None,
            agents: None,
            services: None,
            dashboard: None,
        },
    };

    let mut services = Vec::new();
    for (name, url) in form.names.into_iter().zip(form.endpoint_urls.into_iter()) {
        if !name.trim().is_empty() {
            services.push(ServiceConfig {
                name: name.trim().to_string(),
                endpoint_url: url.trim().to_string(),
                enabled: default_service_enabled(),
                timeout_ms: None,
                max_retries: None,
            });
        }
    }
    config.services = Some(services);

    match config.save() {
        Ok(_) => render(ToastPartial {
            message: "Service settings saved successfully".to_string(),
            success: true,
        }),
        Err(e) => render(ToastPartial {
            message: format!("Failed to save settings: {}", e),
            success: false,
        }),
    }
}

#[derive(Debug, Deserialize)]
pub struct SingleServiceTestForm {
    pub name: String,
    pub endpoint_url: String,
}

pub async fn test_service_connection(
    axum::Form(form): axum::Form<SingleServiceTestForm>,
) -> Response {
    let is_valid = !form.endpoint_url.trim().is_empty() && form.endpoint_url.starts_with("http");

    if is_valid {
        render(ToastPartial {
            message: format!("Connection to {} successful (mock)", form.name),
            success: true,
        })
    } else {
        render(ToastPartial {
            message: format!("Invalid endpoint for {}: {}", form.name, form.endpoint_url),
            success: false,
        })
    }
}

pub async fn test_plane_connection(axum::Form(form): axum::Form<PlaneSettingsForm>) -> Response {
    // Simple validation: check that required fields are filled and api_url looks like a URL
    let is_valid = !form.api_url.trim().is_empty()
        && !form.api_key.trim().is_empty()
        && !form.workspace_slug.trim().is_empty()
        && form.api_url.starts_with("http");

    if is_valid {
        // In a real implementation, you would make an HTTP request to verify connectivity
        render(ToastPartial {
            message: "Plane connection test passed (mock)".to_string(),
            success: true,
        })
    } else {
        render(ToastPartial {
            message: "Plane settings are incomplete or invalid".to_string(),
            success: false,
        })
    }
}

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
        .route("/api/settings/services", post(save_services_settings))
        .route("/api/settings/services/test", post(test_service_connection))
        .route("/hub", get(hub_page))
        .route("/api/settings/plane", post(save_plane_settings))
        .route("/api/settings/plane/test", post(test_plane_connection))
        .route("/api/settings/agents", post(save_agent_settings))
        .route(
            "/api/settings/agents/test-connection",
            post(test_agent_connection),
        )
        .route("/api/settings/dashboard", post(save_dashboard_settings))
        .route(
            "/api/dashboard/services/{name}/restart",
            post(restart_service),
        )
        .route(
            "/api/dashboard/services/{name}/config",
            axum::routing::patch(patch_service_config),
        )
        .route(
            "/api/dashboard/services/{name}/toggle",
            post(toggle_service),
        )
        .route("/api/dashboard/kanban", get(kanban_board))
        .route("/api/dashboard/features/{id}", get(feature_detail))
        .route("/api/dashboard/features/{id}/work-packages", get(wp_list))
        .route("/api/dashboard/features/{id}/events", get(feature_events))
        .route("/api/dashboard/features/{id}/media", get(feature_media))
        // HTML partial endpoints (HTMX-compatible)
        .route("/api/dashboard/health", get(health_panel))
        .route("/api/dashboard/events", get(event_timeline))
        .route("/api/dashboard/agents", get(agent_activity))
        // JSON API endpoints (for polling from JavaScript templates)
        .route("/api/dashboard/agents.json", get(agents_json))
        .route("/api/dashboard/health.json", get(health_json))
        .route("/api/dashboard/projects", get(project_switcher))
        .route(
            "/api/dashboard/projects/{id}/activate",
            post(switch_project),
        )
        .route("/api/time", get(time_footer))
        .route("/api/stream-placeholder", get(stream_placeholder))
        .route(
            "/api/evidence/{feature_id}/{artifact_id}/content",
            get(evidence_content),
        )
        .route(
            "/api/evidence/{feature_id}/{artifact_id}/preview",
            get(evidence_preview),
        )
        .route(
            "/api/features/{id}/evidence",
            get(feature_evidence_list),
        )
        .route(
            "/api/features/{id}/evidence/generate",
            post(feature_evidence_generate),
        )
        .route(
            "/api/dashboard/features/{id}/evidence.json",
            get(feature_evidence_json),
        )
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use tower::util::ServiceExt;
        use super::*;
    use crate::app_state::{DashboardStore, default_health};
    use crate::templates::{AgentActivityPartial, AgentView, EventTimelinePartial};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn make_state() -> SharedState {
        let store = DashboardStore {
            health: default_health(),
            ..Default::default()
        };
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
    async fn services_settings_page_renders() {
        let state = make_state();
        let store = state.read().await;
        let tpl = ServicesSettingsPage {
            services: store.health.clone(),
            configs: vec![],
        };
        let html = tpl.render().expect("template renders");
        assert!(html.contains("Service Endpoints"));
    }

    #[tokio::test]
    async fn plane_settings_page_renders() {
        let state = make_state();
        let response = plane_settings_page(State(state)).await;
        let body = response.into_body();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let html = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(html.contains("Native Plane Views"));
        assert!(html.contains("Browse Synced Features"));
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
                pid: Some(12345),
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

    #[tokio::test]
    async fn toggle_service_updates_store_and_responds() {
        let state = make_state();
        let app = router(state.clone());

        let request_body = serde_json::json!({ "enabled": false }).to_string();
        let request = axum::http::Request::builder()
            .method("POST")
            .uri("/api/dashboard/services/NATS/toggle")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(request_body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_text = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_text.contains("\"status\":\"ok\""));
        assert!(body_text.contains("\"enabled\":false"));

        let store = state.read().await;
        let health = store.health.iter().find(|s| s.name == "NATS").unwrap();
        assert!(!health.healthy);
        assert!(health.degraded);
    }

    #[tokio::test]
    async fn restart_service_executes_command() {
        let state = make_state();
        let app = router(state.clone());

        unsafe {
            std::env::set_var("AGILEPLUS_SERVICE_RESTART_CMD", "echo restarted {}");
        }

        let request = axum::http::Request::builder()
            .method("POST")
            .uri("/api/dashboard/services/NATS/restart")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_text = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_text.contains("\"status\":\"ok\""));
        assert!(body_text.contains("\"service\":\"NATS\""));
        assert!(body_text.contains("restarted NATS"));
    }

    // ── Pure-function unit tests ──────────────────────────────────────────

    #[test]
    fn test_artifact_type_for_ext_coverage_variants() {
        assert_eq!(artifact_type_for_ext("lcov"), "coverage");
        assert_eq!(artifact_type_for_ext("coverage"), "coverage");
        assert_eq!(artifact_type_for_ext("cov"), "coverage");
    }

    #[test]
    fn test_artifact_type_for_ext_test_results() {
        assert_eq!(artifact_type_for_ext("xml"), "test-results");
        assert_eq!(artifact_type_for_ext("junit"), "test-results");
        assert_eq!(artifact_type_for_ext("tap"), "test-results");
    }

    #[test]
    fn test_artifact_type_for_ext_report() {
        assert_eq!(artifact_type_for_ext("json"), "report");
        assert_eq!(artifact_type_for_ext("sarif"), "report");
    }

    #[test]
    fn test_artifact_type_for_ext_image() {
        assert_eq!(artifact_type_for_ext("png"), "image");
        assert_eq!(artifact_type_for_ext("jpg"), "image");
        assert_eq!(artifact_type_for_ext("svg"), "image");
    }

    #[test]
    fn test_artifact_type_for_ext_text() {
        assert_eq!(artifact_type_for_ext("md"), "text");
        assert_eq!(artifact_type_for_ext("log"), "text");
        assert_eq!(artifact_type_for_ext("txt"), "text");
    }

    #[test]
    fn test_artifact_type_for_ext_unknown_falls_back() {
        assert_eq!(artifact_type_for_ext("unknown"), "artifact");
        assert_eq!(artifact_type_for_ext(""), "artifact");
        assert_eq!(artifact_type_for_ext("xyz"), "artifact");
    }

    #[test]
    fn test_percent_encode_path_spaces() {
        let encoded = percent_encode_path("/path/with spaces/file.txt");
        assert!(encoded.contains("%20"));
        assert!(!encoded.contains(' '));
    }

    #[test]
    fn test_percent_encode_path_no_change_for_clean_path() {
        let path = "/artifacts/features/my-feature.md";
        assert_eq!(percent_encode_path(path), path);
    }

    #[test]
    fn test_percent_encode_path_hash_and_question() {
        let encoded = percent_encode_path("/path/file#section?query=1");
        assert!(encoded.contains("%23"));
        assert!(encoded.contains("%3F"));
    }

    #[test]
    fn test_percent_encode_path_percent_and_plus() {
        let encoded = percent_encode_path("/path/100%done+extra");
        assert!(encoded.contains("%25"));
        assert!(encoded.contains("%2B"));
    }

    #[test]
    fn test_html_escape_ampersand() {
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn test_html_escape_angle_brackets() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn test_html_escape_quotes() {
        assert_eq!(html_escape("say \"hello\""), "say &quot;hello&quot;");
        assert_eq!(html_escape("it's"), "it&#39;s");
    }

    #[test]
    fn test_html_escape_no_op_on_plain_text() {
        let plain = "Hello, world!";
        assert_eq!(html_escape(plain), plain);
    }

    #[test]
    fn test_is_htmx_true() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("HX-Request", "true".parse().unwrap());
        assert!(is_htmx(&headers));
    }

    #[test]
    fn test_is_htmx_false_absent() {
        let headers = axum::http::HeaderMap::new();
        assert!(!is_htmx(&headers));
    }

    #[test]
    fn test_is_htmx_false_wrong_value() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("HX-Request", "1".parse().unwrap());
        assert!(!is_htmx(&headers));
    }

    #[test]
    fn test_percentage_coverage_normal() {
        assert_eq!(percentage_coverage(3, 10), "3/10 (30%)");
    }

    #[test]
    fn test_percentage_coverage_zero_total() {
        assert_eq!(percentage_coverage(0, 0), "0/0 (0%)");
    }

    #[test]
    fn test_percentage_coverage_full() {
        assert_eq!(percentage_coverage(5, 5), "5/5 (100%)");
    }

    #[test]
    fn test_dashboard_filter_from_query_active() {
        let mut q = std::collections::HashMap::new();
        q.insert("filter".to_string(), "active".to_string());
        assert_eq!(dashboard_filter_from_query(&q), DashboardFilter::Active);
    }

    #[test]
    fn test_dashboard_filter_from_query_blocked() {
        let mut q = std::collections::HashMap::new();
        q.insert("filter".to_string(), "blocked".to_string());
        assert_eq!(dashboard_filter_from_query(&q), DashboardFilter::Blocked);
    }

    #[test]
    fn test_dashboard_filter_from_query_shipped() {
        let mut q = std::collections::HashMap::new();
        q.insert("filter".to_string(), "shipped".to_string());
        assert_eq!(dashboard_filter_from_query(&q), DashboardFilter::Shipped);
    }

    #[test]
    fn test_dashboard_filter_from_query_default_all() {
        let q = std::collections::HashMap::new();
        assert_eq!(dashboard_filter_from_query(&q), DashboardFilter::All);
    }

    #[test]
    fn test_dashboard_filter_from_query_unknown_maps_to_all() {
        let mut q = std::collections::HashMap::new();
        q.insert("filter".to_string(), "unknown-value".to_string());
        assert_eq!(dashboard_filter_from_query(&q), DashboardFilter::All);
    }

    #[test]
    fn test_plane_api_key_hint_none() {
        assert_eq!(plane_api_key_hint(&None), "Not configured");
    }

    #[test]
    fn test_plane_api_key_hint_some_key() {
        let key = Some("abc123xyz".to_string());
        let hint = plane_api_key_hint(&key);
        assert!(hint.starts_with('a'));
        assert!(hint.ends_with('z'));
        assert!(hint.contains('•'));
    }

    #[test]
    fn test_plane_connection_checks_both_present() {
        let (ok, status, warnings) = plane_connection_checks(
            &Some("mykey".to_string()),
            &Some("myworkspace".to_string()),
        );
        assert!(ok);
        assert!(status.contains("Connected"));
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_plane_connection_checks_missing_key() {
        let (ok, _status, warnings) =
            plane_connection_checks(&None, &Some("myworkspace".to_string()));
        assert!(!ok);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("PLANE_API_KEY"));
    }

    #[test]
    fn test_plane_connection_checks_both_missing() {
        let (ok, status, warnings) = plane_connection_checks(&None, &None);
        assert!(!ok);
        assert_eq!(warnings.len(), 2);
        assert!(status.contains("incomplete"));
    }

    #[test]
    fn test_validate_restart_command_allowed() {
        assert!(validate_restart_command("echo hello").is_ok());
        assert!(validate_restart_command("systemctl restart foo").is_ok());
    }

    #[test]
    fn test_validate_restart_command_denied() {
        let result = validate_restart_command("rm -rf /");
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("not in approved"));
    }

    #[test]
    fn test_validate_restart_command_empty() {
        let result = validate_restart_command("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_is_restart_command_allowed_known_programs() {
        assert!(is_restart_command_allowed("echo"));
        assert!(is_restart_command_allowed("systemctl"));
        assert!(is_restart_command_allowed("docker"));
        assert!(is_restart_command_allowed("process-compose"));
    }

    #[test]
    fn test_is_restart_command_allowed_unknown() {
        assert!(!is_restart_command_allowed("curl"));
        assert!(!is_restart_command_allowed("bash"));
        assert!(!is_restart_command_allowed("sh"));
        assert!(!is_restart_command_allowed("rm"));
    }

    // ── JSON API Tests ────────────────────────────────────────────────────

    #[tokio::test]
    async fn json_endpoints_integrated_agents_in_router() {
        let state = make_state();
        let app = router(state);

        // Test /api/dashboard/agents.json
        let request = axum::http::Request::builder()
            .method("GET")
            .uri("/api/dashboard/agents.json")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_text = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Verify JSON structure
        let json: serde_json::Value = serde_json::from_str(&body_text).expect("valid JSON");
        assert!(json.get("agents").is_some());
        assert!(json.get("count").is_some());
        assert!(json.get("timestamp").is_some());
    }

    #[tokio::test]
    async fn agents_json_response_structure() {
        let state = make_state();
        let app = router(state);

        let request = axum::http::Request::builder()
            .method("GET")
            .uri("/api/dashboard/agents.json")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_text = String::from_utf8(body_bytes.to_vec()).unwrap();
        let json: serde_json::Value = serde_json::from_str(&body_text).expect("valid JSON");

        let agents = json.get("agents").unwrap();
        assert!(agents.is_array());
        // When no agents detected, agents array should be empty but structure valid
        if let Some(first_agent) = agents.as_array().and_then(|a| a.first()) {
            assert!(first_agent.get("name").is_some());
            assert!(first_agent.get("status").is_some());
            assert!(first_agent.get("current_task").is_some());
            assert!(first_agent.get("uptime").is_some());
        }
    }

    #[tokio::test]
    async fn json_endpoints_integrated_health_in_router() {
        let state = make_state();
        let app = router(state);

        // Test /api/dashboard/health.json
        let request = axum::http::Request::builder()
            .method("GET")
            .uri("/api/dashboard/health.json")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_text = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Verify JSON structure
        let json: serde_json::Value = serde_json::from_str(&body_text).expect("valid JSON");
        assert!(json.get("services").is_some());
        assert!(json.get("timestamp").is_some());
        assert!(json.get("all_healthy").is_some());
    }

    #[tokio::test]
    async fn health_json_response_structure() {
        let state = make_state();
        let app = router(state);

        let request = axum::http::Request::builder()
            .method("GET")
            .uri("/api/dashboard/health.json")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_text = String::from_utf8(body_bytes.to_vec()).unwrap();
        let json: serde_json::Value = serde_json::from_str(&body_text).expect("valid JSON");

        let services = json.get("services").unwrap();
        assert!(services.is_array());
        // With default seeded data, should have services
        if let Some(first_service) = services.as_array().and_then(|a| a.first()) {
            assert!(first_service.get("name").is_some());
            assert!(first_service.get("healthy").is_some());
            assert!(first_service.get("degraded").is_some());
            assert!(first_service.get("last_check").is_some());
        }

        // Verify all_healthy flag (with seeded data, should be true)
        let all_healthy = json.get("all_healthy").unwrap().as_bool().unwrap();
        assert!(all_healthy);
    }

}
