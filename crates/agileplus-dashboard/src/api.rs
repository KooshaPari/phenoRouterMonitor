use std::collections::HashMap;
use std::path::PathBuf;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    response::Response,
    routing::{get, post},
    Router,
};
use chrono::Utc;
use crate::app_state::SharedState;
use crate::templates::{
    AgentActivityPartial, AgentView, EvidenceBundleView, FeatureDetailPage,
    FeatureEvidencePartial, FeatureView, WpView,
};
use crate::routes::{
    render, is_htmx, html_escape, load_projects, build_kanban_cards,
    build_feature_events, build_feature_evidence_bundles, build_feature_media_assets,
    build_feature_reports, calculate_uptime, load_evidence_bundles_from_disk,
    project_switcher, switch_project, process_detector, ServiceHealth,
    AgentInfo, EvidenceArtifactJson, EvidenceGalleryJson, GenerateEvidenceResponse,
};

pub async fn feature_detail(
    State(state): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    _headers: axum::http::HeaderMap,
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

pub async fn wp_list(
    State(state): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Response {
    let store = state.read().await;
    let wps: Vec<WpView> = store
        .work_packages
        .get(&id)
        .map(|v| v.iter().map(WpView::from_wp).collect())
        .unwrap_or_default();
    use crate::templates::WpListPartial;
    render(WpListPartial { feature_id: id, workpackages: wps })
}

pub async fn feature_media(
    State(state): State<SharedState>,
    axum::extract::Path(feature_id): axum::extract::Path<i64>,
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

    axum::response::Html(format!(
        r#"<div class="grid grid-cols-2 gap-3 media-gallery">{}</div>"#,
        html
    )).into_response()
}

pub async fn agent_activity(State(state): State<SharedState>) -> Response {
    let _ = state.read().await;
    let detected = process_detector::detect_agents();
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

pub async fn agents_json(State(_state): State<SharedState>) -> impl IntoResponse {
    let detected = process_detector::detect_agents();
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

pub async fn project_switcher(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let projects: Vec<crate::templates::ProjectView> = store
        .projects
        .iter()
        .map(|p| crate::templates::ProjectView {
            id: p.id,
            slug: p.slug.clone(),
            name: p.name.clone(),
            description: p.description.clone(),
        })
        .collect();
    use crate::templates::ProjectSwitcherPartial;
    render(ProjectSwitcherPartial {
        projects,
        active_id: store.active_project_id,
    })
}

pub async fn switch_project(
    State(state): State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Response {
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
    use crate::routes::DashboardFilter;
    let cards = build_kanban_cards(&store, DashboardFilter::All);
    use crate::templates::KanbanPartial;
    render(KanbanPartial { cards })
}

pub async fn evidence_content(
    State(_state): State<SharedState>,
    axum::extract::Path((feature_id, artifact_id)): axum::extract::Path<(i64, String)>,
) -> Response {
    let artifact_path = PathBuf::from(".agileplus")
        .join("evidence")
        .join(feature_id.to_string())
        .join(&artifact_id);

    if let Ok(content) = std::fs::read_to_string(&artifact_path) {
        let escaped = html_escape(&content);
        return axum::response::Html(format!(
            "<pre class='text-xs font-mono text-zinc-300 whitespace-pre-wrap'>{escaped}</pre>"
        )).into_response();
    }

    axum::response::Html(format!(
        "# Evidence Bundle {feature_id}\n\n## Artifact ID: {artifact_id}\n\nNo artifact found at expected path."
    )).into_response()
}

pub async fn evidence_preview(
    State(_state): State<SharedState>,
    axum::extract::Path((feature_id, artifact_id)): axum::extract::Path<(i64, String)>,
) -> Response {
    let artifact_path = PathBuf::from(".agileplus")
        .join("evidence")
        .join(feature_id.to_string())
        .join(&artifact_id);

    let text = std::fs::read_to_string(&artifact_path)
        .unwrap_or_else(|_| format!("No preview — artifact not found: {artifact_id}"));
    let escaped = html_escape(&text);
    let preview = format!(
        "<div class='p-3 rounded bg-zinc-800 border border-zinc-700'>
  <pre class='text-xs font-mono text-zinc-300 max-h-48 overflow-y-auto'>{escaped}</pre>
</div>"
    );
    axum::response::Html(preview).into_response()
}

pub async fn feature_evidence_list(
    State(_state): State<SharedState>,
    axum::extract::Path(feature_id): axum::extract::Path<String>,
) -> Response {
    let bundles = load_evidence_bundles_from_disk(&feature_id);
    let tmpl = FeatureEvidencePartial { evidence_bundles: bundles };
    match tmpl.render() {
        Ok(html) => axum::response::Html(html).into_response(),
        Err(e) => {
            tracing::error!("Template render error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn feature_evidence_generate(
    State(_state): State<SharedState>,
    axum::extract::Path(feature_id): axum::extract::Path<String>,
) -> Response {
    let script = PathBuf::from("scripts").join("generate-evidence.sh");
    if !script.exists() {
        return axum::Json(GenerateEvidenceResponse {
            feature_id: feature_id.clone(),
            bundle_path: String::new(),
            status: "error".into(),
            message: "generate-evidence.sh not found — ensure the server is started from the repo root".into(),
        }).into_response();
    }

    let bundle_path = format!(".agileplus/evidence/{feature_id}/bundle.json");
    let fid = feature_id.clone();

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
    }).into_response()
}

pub async fn feature_evidence_json(
    State(_state): State<SharedState>,
    axum::extract::Path(feature_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let bundles = load_evidence_bundles_from_disk(&feature_id);
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

    let generated_at = bundles.first().map(|b| b.created_at.clone());

    axum::Json(EvidenceGalleryJson {
        feature_id,
        artifacts,
        generated_at,
    })
}

pub async fn time_footer() -> axum::response::Html<String> {
    axum::response::Html(
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    )
}

pub async fn stream_placeholder() -> StatusCode {
    StatusCode::NO_CONTENT
}

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
    axum::extract::Path(name): axum::extract::Path<String>,
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

#[derive(serde::Deserialize)]
pub struct ServiceConfigForm {
    pub endpoint_url: Option<String>,
    pub timeout_ms: Option<u64>,
    pub max_retries: Option<u32>,
}

pub async fn patch_service_config(
    axum::extract::Path(name): axum::extract::Path<String>,
    axum::Form(form): axum::Form<ServiceConfigForm>,
) -> impl IntoResponse {
    use crate::routes::Config;
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
        services.push(crate::routes::ServiceConfig {
            name: name.clone(),
            endpoint_url: url,
            enabled: default_service_enabled(),
            timeout_ms: form.timeout_ms,
            max_retries: form.max_retries,
        });
    }

    use crate::templates::ToastPartial;
    match config.save() {
        Ok(_) => render(ToastPartial { message: format!("Service '{}' configuration saved", name), success: true }),
        Err(e) => render(ToastPartial { message: format!("Failed to save: {}", e), success: false }),
    }
}

#[derive(serde::Deserialize)]
pub struct ServiceToggleBody {
    pub enabled: Option<bool>,
}

pub async fn toggle_service(
    State(state): State<SharedState>,
    axum::extract::Path(name): axum::extract::Path<String>,
    axum::Json(body): axum::Json<ServiceToggleBody>,
) -> impl IntoResponse {
    let enabled = body.enabled.unwrap_or(true);

    let mut config = crate::routes::Config::load().unwrap_or(Config {
        plane: None,
        agents: None,
        services: None,
        dashboard: None,
    });

    let services = config.services.get_or_insert_with(Vec::new);
    if let Some(entry) = services.iter_mut().find(|s| s.name == name) {
        entry.enabled = enabled;
    } else {
        services.push(crate::routes::ServiceConfig {
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

pub async fn features_page(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let features = store
        .features
        .iter()
        .map(FeatureView::from_feature)
        .collect::<Vec<_>>();
    use crate::templates::FeaturesPage;
    render(FeaturesPage { features })
}

pub async fn hub_page() -> Response {
    use crate::templates::{HubPage, EcosystemProject};
    let projects = vec![
        EcosystemProject { name: "phenodocs", tagline: "Ecosystem docs hub".into(), stack: "TypeScript · Vue".into(), port: Some(4100), github: "https://github.com/KooshaPari/phenodocs".into(), category: "docs".into() },
        EcosystemProject { name: "AgilePlus", tagline: "Spec-driven PM platform".into(), stack: "Rust · Tauri".into(), port: Some(4101), github: "https://github.com/KooshaPari/AgilePlus".into(), category: "app".into() },
        EcosystemProject { name: "heliosApp", tagline: "TypeScript runtime app".into(), stack: "TypeScript · Bun".into(), port: Some(4102), github: "https://github.com/KooshaPari/heliosApp".into(), category: "app".into() },
        EcosystemProject { name: "thegent", tagline: "Agent framework".into(), stack: "TypeScript · Python".into(), port: Some(4103), github: "https://github.com/KooshaPari/thegent".into(), category: "lib".into() },
        EcosystemProject { name: "bifrost-extensions", tagline: "LLM gateway extensions".into(), stack: "Go".into(), port: Some(4104), github: "https://github.com/KooshaPari/bifrost-extensions".into(), category: "lib".into() },
        EcosystemProject { name: "civ", tagline: "CI validation".into(), stack: "TypeScript".into(), port: Some(4105), github: "https://github.com/KooshaPari/civ".into(), category: "docs".into() },
        EcosystemProject { name: "TraceRTM", tagline: "Requirements traceability".into(), stack: "Python · Go · TS".into(), port: Some(4110), github: "https://github.com/KooshaPari/trace".into(), category: "app".into() },
        EcosystemProject { name: "agentapi-plusplus", tagline: "Agent HTTP API".into(), stack: "Go".into(), port: None, github: "https://github.com/KooshaPari/agentapi-plusplus".into(), category: "api".into() },
        EcosystemProject { name: "cliproxyapi-plusplus", tagline: "Multi-provider CLI proxy".into(), stack: "Go".into(), port: None, github: "https://github.com/KooshaPari/cliproxyapi-plusplus".into(), category: "api".into() },
    ];
    render(HubPage { projects })
}

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/features", get(features_page))
        .route("/hub", get(hub_page))
        .route("/api/dashboard/features/{id}", get(feature_detail))
        .route("/api/dashboard/features/{id}/work-packages", get(wp_list))
        .route("/api/dashboard/features/{id}/media", get(feature_media))
        .route("/api/dashboard/agents", get(agent_activity))
        .route("/api/dashboard/agents.json", get(agents_json))
        .route("/api/dashboard/projects", get(project_switcher))
        .route("/api/dashboard/projects/{id}/activate", post(switch_project))
        .route("/api/time", get(time_footer))
        .route("/api/stream-placeholder", get(stream_placeholder))
        .route("/api/evidence/{feature_id}/{artifact_id}/content", get(evidence_content))
        .route("/api/evidence/{feature_id}/{artifact_id}/preview", get(evidence_preview))
        .route("/api/features/{id}/evidence", get(feature_evidence_list))
        .route("/api/features/{id}/evidence/generate", post(feature_evidence_generate))
        .route("/api/dashboard/features/{id}/evidence.json", get(feature_evidence_json))
        .route("/api/dashboard/services/{name}/restart", post(restart_service))
        .route("/api/dashboard/services/{name}/config", axum::routing::patch(patch_service_config))
        .route("/api/dashboard/services/{name}/toggle", post(toggle_service))
}