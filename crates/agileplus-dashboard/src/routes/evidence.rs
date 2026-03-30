//! Evidence bundle handlers for feature documentation.

use std::fs;
use std::path::PathBuf;

use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use crate::app_state::SharedState;
use crate::templates::{CiLinkView, EvidenceBundleView, FeatureEvidencePartial, GenerateEvidenceResponse, GitCommitView, PrLinkView};

use super::api::EvidenceArtifactJson;
use super::helpers::html_escape;

// ── Evidence Loading & Parsing ─────────────────────────────────────────────

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

// ── Evidence Content Handlers ──────────────────────────────────────────────

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

    axum::Json(super::api::EvidenceGalleryJson {
        feature_id,
        artifacts,
        generated_at,
    })
}
