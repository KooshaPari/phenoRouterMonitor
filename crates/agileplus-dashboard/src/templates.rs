//! Askama template structs — one per HTML template file.
//!
//! The `#[template(path = "...")]` path is relative to the `templates/` dirs
//! configured in `[package.metadata.askama]`.

use std::collections::HashMap;

use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::WorkPackage;
use askama::Template;

use crate::app_state::ServiceHealth;

/// Project view model used in project switcher and pages.
#[derive(Debug, Clone)]
pub struct ProjectView {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: String,
}

/// Project summary view model used on the homepage.
#[derive(Debug, Clone)]
pub struct ProjectSummaryView {
    pub project: ProjectView,
    pub feature_count: usize,
    pub active_count: usize,
    pub shipped_count: usize,
}

/// Work-package view model used in partials.
#[derive(Debug, Clone)]
pub struct WpView {
    pub id: i64,
    pub title: String,
    pub state: String,
    pub agent: String,
    pub progress: u8,
    pub task_count: usize,
    /// Raw agent identifier, used to build agent deep-links.
    pub agent_id: Option<String>,
    /// GitHub PR URL when a PR has been submitted for this work package.
    pub pr_url: Option<String>,
    /// Most recent commit SHA on the work package branch.
    pub head_commit: Option<String>,
}

impl WpView {
    pub fn from_wp(wp: &WorkPackage) -> Self {
        Self {
            id: wp.id,
            title: wp.title.clone(),
            state: format!("{:?}", wp.state).to_lowercase(),
            agent: wp.agent_id.clone().unwrap_or_else(|| "—".into()),
            progress: 0,
            task_count: 0,
            agent_id: wp.agent_id.clone(),
            pr_url: wp.pr_url.clone(),
            head_commit: wp.head_commit.clone(),
        }
    }
}

/// Feature view model used on kanban cards and detail pages.
#[derive(Debug, Clone)]
pub struct FeatureView {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub state: String,
    pub labels: Vec<String>,
}

impl FeatureView {
    pub fn from_feature(f: &Feature) -> Self {
        Self {
            id: f.id,
            slug: f.slug.clone(),
            title: f.friendly_name.clone(),
            state: f.state.to_string(),
            labels: f.labels.clone(),
        }
    }
}

// ── Full-page templates ────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct HomePage {
    pub total_features: usize,
    pub active_features: usize,
    pub shipped_features: usize,
    pub projects: Vec<ProjectSummaryView>,
}

#[derive(Template)]
#[template(path = "pages/dashboard.html")]
pub struct DashboardPage {
    pub kanban_cards: HashMap<String, Vec<FeatureView>>,
    pub health: Vec<ServiceHealth>,
    pub projects: Vec<ProjectView>,
    pub active_project: Option<ProjectView>,
    pub active_filter: String,
}

#[derive(Template)]
#[template(path = "pages/feature-detail.html")]
pub struct FeatureDetailPage {
    pub feature: FeatureView,
    pub feature_id: i64,
    pub workpackages: Vec<WpView>,
    pub events: Vec<EventView>,
    pub evidence_bundles: Vec<EvidenceBundleView>,
    pub media_assets: Vec<MediaAssetView>,
    pub reports: Vec<ReportArtifactView>,
}

#[derive(Debug, Clone)]
pub struct EvidenceBundleView {
    pub id: String,
    pub fr_id: String,
    pub evidence_type: String,
    pub wp_id: String,
    pub wp_title: String,
    pub artifact_path: String,
    pub created_at: String,
    pub artifact_ext: String,
    pub status: String,
    // Preview / download
    pub content_preview: Option<String>,
    pub is_text_artifact: bool,
    pub is_image_artifact: bool,
    pub download_url: String,
    // Evidence gallery enrichment fields
    pub test_passed: Option<bool>,
    pub tests_passed_count: u32,
    pub tests_failed_count: u32,
    pub test_summary: Option<String>,
    pub commit_count: usize,
    pub pr_count: usize,
    pub ci_links: Vec<CiLinkView>,
    pub git_commits: Vec<GitCommitView>,
    pub pr_links: Vec<PrLinkView>,
}

/// A single CI run link for the evidence gallery.
#[derive(Debug, Clone)]
pub struct CiLinkView {
    pub id: i64,
    pub title: String,
    pub status: String,
    pub conclusion: String,
    pub url: String,
    pub created_at: String,
}

/// A git commit entry for the evidence gallery.
#[derive(Debug, Clone)]
pub struct GitCommitView {
    pub short_hash: String,
    pub subject: String,
    pub date: String,
    pub author: String,
    pub url: String,
}

/// A PR link for the evidence gallery.
#[derive(Debug, Clone)]
pub struct PrLinkView {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub state: String,
    pub head_ref: String,
    pub created_at: String,
}

/// JSON body returned from `POST /api/features/{id}/evidence/generate`.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GenerateEvidenceResponse {
    pub feature_id: String,
    pub bundle_path: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct MediaAssetView {
    pub id: String,
    pub source: String,
    pub name: String,
    pub kind: String,
    pub mime: String,
    pub url_or_path: String,
    pub size_bytes: usize,
    pub uploaded_at: String,
}

#[derive(Debug, Clone)]
pub struct ReportArtifactView {
    pub id: String,
    pub name: String,
    pub source: String,
    pub status: String,
    pub generated_at: String,
    pub rule_count: usize,
    pub satisfied_count: usize,
    pub compliant: bool,
}

#[derive(Template)]
#[template(path = "pages/settings.html")]
pub struct SettingsPage;

#[derive(Template)]
#[template(path = "pages/features.html")]
pub struct FeaturesPage {
    pub features: Vec<FeatureView>,
}

#[derive(Template)]
#[template(path = "pages/events.html")]
pub struct EventsPage {
    pub events: Vec<EventView>,
}

#[derive(Template)]
#[template(path = "pages/settings-plane.html")]
pub struct PlaneSettingsPage {
    pub workspace_name: String,
    pub workspace_slug: String,
    pub project_slug: String,
    pub plane_api_url: String,
    pub plane_web_url: String,
    pub plane_api_url_set: bool,
    pub plane_web_url_set: bool,
    pub plane_api_key_hint: String,
    pub plane_api_key_set: bool,
    pub sync_enabled: bool,
    pub sync_mode: String,
    pub connected: bool,
    pub connection_status: String,
    pub connection_status_configured: bool,
    pub plane_service_healthy: bool,
    pub plane_api_latency_ms: Option<u64>,
    pub plane_health_endpoints: Vec<PlaneHealthEndpointView>,
    pub mapped_features_coverage: String,
    pub mapped_work_packages_coverage: String,
    pub mapped_features: usize,
    pub mapped_work_packages: usize,
    pub config_warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PlaneHealthEndpointView {
    pub name: String,
    pub healthy: bool,
    pub degraded: bool,
    pub latency_ms: Option<u64>,
    pub last_check_utc: String,
}

#[derive(Template)]
#[template(path = "pages/settings-agents.html")]
pub struct AgentSettingsPage {
    pub agent_pool_size: usize,
    pub retry_budget: usize,
    pub dispatch_mode: String,
    pub default_provider: String,
}

#[derive(Template)]
#[template(path = "pages/settings-services.html")]
pub struct ServicesSettingsPage {
    pub services: Vec<ServiceHealth>,
    pub configs: Vec<ServiceConfigView>,
}

#[derive(Debug, Clone)]
pub struct ServiceConfigView {
    pub name: String,
    pub endpoint_url: String,
}

// ── Partial templates ──────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "partials/kanban.html")]
pub struct KanbanPartial {
    pub cards: HashMap<String, Vec<FeatureView>>,
}

#[derive(Template)]
#[template(path = "partials/wp-list.html")]
pub struct WpListPartial {
    pub feature_id: i64,
    pub workpackages: Vec<WpView>,
}

#[derive(Template)]
#[template(path = "partials/feature-evidence.html")]
pub struct FeatureEvidencePartial {
    pub evidence_bundles: Vec<EvidenceBundleView>,
}

#[derive(Template)]
#[template(path = "partials/feature-media.html")]
pub struct FeatureMediaPartial {
    pub media_assets: Vec<MediaAssetView>,
}

#[derive(Template)]
#[template(path = "partials/feature-reports.html")]
pub struct FeatureReportsPartial {
    pub reports: Vec<ReportArtifactView>,
}

#[derive(Template)]
#[template(path = "partials/health-panel.html")]
pub struct HealthPanelPartial {
    pub services: Vec<ServiceHealth>,
}

#[derive(Template)]
#[template(path = "partials/event-timeline.html")]
pub struct EventTimelinePartial {
    pub feature_id: i64,
    pub events: Vec<EventView>,
}

/// Enhanced event view model with clickable links and metadata.
#[derive(Debug, Clone)]
pub struct EventView {
    pub id: String,
    pub kind: String,
    pub description: String,
    pub timestamp: String,
    // New clickable fields for enhanced timeline
    pub agent_name: Option<String>,
    pub agent_link: Option<String>,
    pub wp_id: Option<String>,
    pub wp_link: Option<String>,
    pub commit_sha: Option<String>,
    pub commit_link: Option<String>,
    pub ci_run_id: Option<String>,
    pub ci_run_link: Option<String>,
}

/// Agent activity view model used in the sidebar panel.
#[derive(Debug, Clone)]
pub struct AgentView {
    pub name: String,
    pub status: String,
    pub current_task: String,
    pub last_action: String,
    pub pid: Option<u32>,
    pub started_at: Option<String>,
    /// Full path to the worktree (used as link href). Empty string = not available.
    pub worktree: String,
    /// Short display label for the worktree (last path segment). Empty string = not available.
    pub worktree_label: String,
    pub is_live: bool,
}

#[derive(Template)]
#[template(path = "partials/agent-activity.html")]
pub struct AgentActivityPartial {
    pub agents: Vec<AgentView>,
}

#[derive(Template)]
#[template(path = "partials/toast.html")]
pub struct ToastPartial {
    pub message: String,
    pub success: bool,
}

#[derive(Template)]
#[template(path = "partials/project-switcher.html")]
pub struct ProjectSwitcherPartial {
    pub projects: Vec<ProjectView>,
    pub active_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct EcosystemProject {
    pub name: &'static str,
    pub tagline: &'static str,
    pub stack: &'static str,
    pub port: Option<u16>,
    pub github: &'static str,
    pub category: &'static str,
}

#[derive(Template)]
#[template(path = "pages/hub.html")]
pub struct HubPage {
    pub projects: Vec<EcosystemProject>,
}

/// Helper: build ordered kanban states list.
pub fn all_feature_states() -> Vec<String> {
    vec![
        FeatureState::Created,
        FeatureState::Specified,
        FeatureState::Researched,
        FeatureState::Planned,
        FeatureState::Implementing,
        FeatureState::Validated,
        FeatureState::Shipped,
        FeatureState::Retrospected,
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect()
}

pub fn label_color(label: &str) -> &'static str {
    match label.to_lowercase().as_str() {
        "platform" | "infrastructure" => "bg-blue-900/40 text-blue-300 border-blue-700/50",
        "governance" | "policy" => "bg-purple-900/40 text-purple-300 border-purple-700/50",
        "automation" | "agents" => "bg-orange-900/40 text-orange-300 border-orange-700/50",
        "testing" | "qa" => "bg-green-900/40 text-green-300 border-green-700/50",
        "bug" | "defect" => "bg-red-900/40 text-red-300 border-red-700/50",
        "research" | "exploration" => "bg-cyan-900/40 text-cyan-300 border-cyan-700/50",
        _ => "bg-zinc-800 text-zinc-300 border-zinc-700",
    }
}
