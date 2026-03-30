use std::collections::HashMap;
use std::env;

use agileplus_domain::domain::{
    feature::Feature, state_machine::FeatureState, work_package::WpState,
};
use askama::Template;
use axum::{
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};

use crate::app_state::DashboardStore;
use crate::templates::{FeatureView, ProjectSummaryView, ProjectView, all_feature_states};

pub(super) const DEFAULT_PLANE_API_URL: &str = "https://app.plane.so";
pub(super) const DEFAULT_PLANE_WEB_URL: &str = "https://app.plane.so";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DashboardFilter {
    All,
    Active,
    Blocked,
    Shipped,
}

pub(super) fn is_htmx(headers: &HeaderMap) -> bool {
    headers
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "true")
        .unwrap_or(false)
}

pub(super) fn render<T: Template>(tpl: T) -> Response {
    match tpl.render() {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Template error: {e}"),
        )
            .into_response(),
    }
}

<<<<<<< HEAD
=======
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

>>>>>>> origin/main
pub(super) fn load_projects(store: &DashboardStore) -> (Vec<ProjectView>, Option<ProjectView>) {
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

pub(super) fn build_project_summaries(store: &DashboardStore) -> Vec<ProjectSummaryView> {
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

pub(super) fn env_or_none(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(super) fn parse_bool_env(key: &str, default: bool) -> bool {
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

pub(super) fn dashboard_filter_from_query(query: &HashMap<String, String>) -> DashboardFilter {
    match query.get("filter").map(|value| value.as_str()) {
        Some("active") => DashboardFilter::Active,
        Some("blocked") => DashboardFilter::Blocked,
        Some("shipped") => DashboardFilter::Shipped,
        _ => DashboardFilter::All,
    }
}

pub(super) fn feature_matches_filter(
    store: &DashboardStore,
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
        DashboardFilter::Shipped => {
            matches!(
                feature.state,
                FeatureState::Shipped | FeatureState::Retrospected
            )
        }
    }
}

pub(super) fn build_kanban_cards(
    store: &DashboardStore,
    filter: DashboardFilter,
) -> HashMap<String, Vec<FeatureView>> {
    let states = all_feature_states();
    let mut cards: HashMap<String, Vec<FeatureView>> = HashMap::new();
    for s in &states {
        cards.insert(s.clone(), vec![]);
    }
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

pub(super) fn sample_events() -> Vec<crate::templates::EventView> {
    vec![
        crate::templates::EventView {
            id: "evt-1".into(),
            kind: "system".into(),
            description: "Dashboard booted with native Plane surface".into(),
            timestamp: "just now".into(),
<<<<<<< HEAD
=======
            agent_name: None,
            agent_link: None,
            wp_id: None,
            wp_link: None,
            commit_sha: None,
            commit_link: None,
            ci_run_id: None,
            ci_run_link: None,
>>>>>>> origin/main
        },
        crate::templates::EventView {
            id: "evt-2".into(),
            kind: "agent_action".into(),
            description: "Planner synced feature ownership metadata".into(),
            timestamp: "2m ago".into(),
<<<<<<< HEAD
=======
            agent_name: Some("planner-agent".into()),
            agent_link: None,
            wp_id: None,
            wp_link: None,
            commit_sha: None,
            commit_link: None,
            ci_run_id: None,
            ci_run_link: None,
>>>>>>> origin/main
        },
        crate::templates::EventView {
            id: "evt-3".into(),
            kind: "state_change".into(),
            description: "Feature moved from researched to planned".into(),
            timestamp: "9m ago".into(),
<<<<<<< HEAD
        },
    ]
}
=======
            agent_name: None,
            agent_link: None,
            wp_id: None,
            wp_link: None,
            commit_sha: None,
            commit_link: None,
            ci_run_id: None,
            ci_run_link: None,
        },
    ]
}

pub fn build_feature_events(
    feature: &crate::templates::FeatureView,
    workpackages: &[crate::templates::WpView],
) -> Vec<crate::templates::EventView> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
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
            agent_name: None,
            agent_link: None,
            wp_id: None,
            wp_link: None,
            commit_sha: None,
            commit_link: None,
            ci_run_id: None,
            ci_run_link: None,
        });

        for wp in workpackages {
            events.push(crate::templates::EventView {
                id: format!("evt-feature-{}-wp-{}", feature.id, wp.id),
                kind: "state_change".into(),
                description: format!("Work-package {} is in state '{}'", wp.title, wp.state),
                timestamp: now.clone(),
                agent_name: None,
                agent_link: None,
                wp_id: Some(wp.id.to_string()),
                wp_link: None,
                commit_sha: None,
                commit_link: None,
                ci_run_id: None,
                ci_run_link: None,
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

pub fn build_feature_media_assets(
    feature: &crate::templates::FeatureView,
    workpackages: &[crate::templates::WpView],
) -> Vec<crate::templates::MediaAssetView> {
    let mut media = vec![crate::templates::MediaAssetView {
        id: format!("media-{id}-cover", id = feature.id),
        source: "dashboard".into(),
        name: format!("{slug}-hero.png", slug = feature.slug),
        kind: "image".into(),
        mime: "image/png".into(),
        url_or_path: format!("/assets/{slug}/cover.png", slug = feature.slug),
        size_bytes: 128_512,
        uploaded_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    }];

    for wp in workpackages {
        media.push(crate::templates::MediaAssetView {
            id: format!("media-{fid}-wp-{wid}", fid = feature.id, wid = wp.id),
            source: "agent-work-package".into(),
            name: format!("{slug}-wp-{wid}.png", slug = feature.slug, wid = wp.id),
            kind: "screenshot".into(),
            mime: "image/png".into(),
            url_or_path: format!("/assets/wp/{wid}/coverage.png", wid = wp.id),
            size_bytes: 84_320 + (wp.id as usize * 3_000),
            uploaded_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        });
    }

    media
}
>>>>>>> origin/main
