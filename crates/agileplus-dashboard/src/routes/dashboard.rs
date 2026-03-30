use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::Utc;

use crate::app_state::SharedState;
use crate::templates::{
<<<<<<< HEAD
    AgentActivityPartial, AgentView, CiLinkView, DashboardPage, EventTimelinePartial,
    EvidenceBundleView, FeatureDetailPage, FeatureView, GitCommitView, HealthPanelPartial,
    KanbanPartial, MediaAssetView, PrLinkView, ProjectSwitcherPartial, ProjectView,
=======
<<<<<<< HEAD
    AgentActivityPartial, AgentView, CiLinkView, DashboardPage, EventTimelinePartial,
    EvidenceBundleView, FeatureDetailPage, FeatureView, GitCommitView, HealthPanelPartial,
    KanbanPartial, MediaAssetView, PrLinkView, ProjectSwitcherPartial, ProjectView,
=======
    AgentActivityPartial, AgentView, DashboardPage, EventTimelinePartial,
    EvidenceBundleView, FeatureDetailPage, FeatureView, HealthPanelPartial,
    KanbanPartial, MediaAssetView, ProjectSwitcherPartial, ProjectView,
>>>>>>> origin/main
>>>>>>> origin/main
    ReportArtifactView, WpListPartial, WpView,
};

use super::helpers::{
    DashboardFilter, build_kanban_cards, dashboard_filter_from_query, is_htmx, load_projects,
    render,
};

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
<<<<<<< HEAD
=======
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
>>>>>>> origin/main
    }];

    if !workpackages.is_empty() {
        events.push(crate::templates::EventView {
            id: format!("evt-feature-{}-sync", feature.id),
            kind: "agent_action".into(),
            description: format!("{} work package entries synced", workpackages.len()),
            timestamp: now.clone(),
<<<<<<< HEAD
=======
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
>>>>>>> origin/main
        });

        for wp in workpackages {
            events.push(crate::templates::EventView {
                id: format!("evt-feature-{}-wp-{}", feature.id, wp.id),
                kind: "state_change".into(),
                description: format!("Work-package {} is in state '{}'", wp.title, wp.state),
                timestamp: now.clone(),
<<<<<<< HEAD
=======
<<<<<<< HEAD
=======
                agent_name: None,
                agent_link: None,
                wp_id: Some(wp.id.to_string()),
                wp_link: None,
                commit_sha: None,
                commit_link: None,
                ci_run_id: None,
                ci_run_link: None,
>>>>>>> origin/main
>>>>>>> origin/main
            });
        }
    } else {
        events.push(crate::templates::EventView {
            id: format!("evt-feature-{}-no-wp", feature.id),
            kind: "system".into(),
            description: "No work packages linked yet".into(),
            timestamp: now.clone(),
<<<<<<< HEAD
=======
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
>>>>>>> origin/main
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

pub async fn health_panel(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    render(HealthPanelPartial {
        services: store.health.clone(),
    })
}

pub async fn event_timeline(State(state): State<SharedState>) -> Response {
    let _ = state.read().await;
    render(EventTimelinePartial {
        feature_id: 0,
        events: vec![],
    })
}

pub async fn agent_activity(_state: State<SharedState>) -> Response {
    let agents: Vec<AgentView> = vec![
        AgentView {
            name: "spec-agent".into(),
            status: "idle".into(),
            current_task: String::new(),
            last_action: "2m ago".into(),
<<<<<<< HEAD
=======
<<<<<<< HEAD
=======
            pid: None,
            started_at: None,
            worktree: String::new(),
            worktree_label: String::new(),
            is_live: false,
>>>>>>> origin/main
>>>>>>> origin/main
        },
        AgentView {
            name: "impl-agent".into(),
            status: "running".into(),
            current_task: "WP13 implementation".into(),
            last_action: "just now".into(),
<<<<<<< HEAD
=======
<<<<<<< HEAD
=======
            pid: Some(12345),
            started_at: Some("2024-01-15 10:30:00 UTC".into()),
            worktree: "/Users/kooshapari/CodeProjects/Phenotype/repos/.worktrees/merge-spec-docs".into(),
            worktree_label: "merge-spec-docs".into(),
            is_live: true,
>>>>>>> origin/main
>>>>>>> origin/main
        },
    ];
    render(AgentActivityPartial { agents })
}

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

pub async fn switch_project(State(state): State<SharedState>, Path(id): Path<i64>) -> Response {
    {
        let mut store = state.write().await;
        if id == 0 {
            store.active_project_id = None;
        } else if store.projects.iter().any(|p| p.id == id) {
            store.active_project_id = Some(id);
        } else {
            return (StatusCode::NOT_FOUND, "Project not found").into_response();
        }
    }

    let store = state.read().await;
    let cards = build_kanban_cards(&store, DashboardFilter::All);
    render(KanbanPartial { cards })
}
<<<<<<< HEAD
=======
<<<<<<< HEAD
=======

// ── /api/dashboard/features/{id}/events ──────────────────────────────────

pub async fn feature_events(
    State(state): State<SharedState>,
    Path(feature_id): Path<i64>,
) -> Response {
    use crate::templates::WpView;

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
    let events = super::helpers::build_feature_events(&feature, &wps);

    render(crate::templates::EventTimelinePartial {
        feature_id,
        events,
    })
}

// ── /api/dashboard/features/{id}/media ───────────────────────────────────

pub async fn feature_media(
    State(state): State<SharedState>,
    Path(feature_id): Path<i64>,
) -> Response {
    use crate::templates::WpView;
    use axum::response::Html;

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
    let media = super::helpers::build_feature_media_assets(&feature, &wps);

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

pub async fn time_footer() -> axum::response::Html<String> {
    axum::response::Html(
        chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
    )
}

pub async fn stream_placeholder() -> StatusCode {
    StatusCode::NO_CONTENT
}
>>>>>>> origin/main
>>>>>>> origin/main
