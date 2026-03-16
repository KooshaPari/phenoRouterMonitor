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

/// Work-package view model used in partials.
#[derive(Debug, Clone)]
pub struct WpView {
    pub id: i64,
    pub title: String,
    pub state: String,
    pub agent: String,
    pub progress: u8,
    pub task_count: usize,
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
#[template(path = "pages/dashboard.html")]
pub struct DashboardPage {
    pub kanban_cards: HashMap<String, Vec<FeatureView>>,
    pub health: Vec<ServiceHealth>,
    pub projects: Vec<ProjectView>,
    pub active_project: Option<ProjectView>,
}

#[derive(Template)]
#[template(path = "pages/feature-detail.html")]
pub struct FeatureDetailPage {
    pub feature: FeatureView,
    pub feature_id: i64,
    pub workpackages: Vec<WpView>,
    pub events: Vec<EventView>,
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
    pub plane_api_url: String,
    pub sync_enabled: bool,
    pub connected: bool,
    pub mapped_features: usize,
    pub mapped_work_packages: usize,
}

#[derive(Template)]
#[template(path = "pages/settings-agents.html")]
pub struct AgentSettingsPage {
    pub agent_pool_size: usize,
    pub retry_budget: usize,
    pub dispatch_mode: String,
}

#[derive(Template)]
#[template(path = "pages/settings-services.html")]
pub struct ServicesSettingsPage {
    pub services: Vec<ServiceHealth>,
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

#[derive(Debug, Clone)]
pub struct EventView {
    pub id: String,
    pub kind: String,
    pub description: String,
    pub timestamp: String,
}

/// Agent activity view model used in the sidebar panel.
#[derive(Debug, Clone)]
pub struct AgentView {
    pub name: String,
    pub status: String,
    pub current_task: String,
    pub last_action: String,
}

#[derive(Template)]
#[template(path = "partials/agent-activity.html")]
pub struct AgentActivityPartial {
    pub agents: Vec<AgentView>,
}

#[derive(Template)]
#[template(path = "partials/project-switcher.html")]
pub struct ProjectSwitcherPartial {
    pub projects: Vec<ProjectView>,
    pub active_id: Option<i64>,
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
