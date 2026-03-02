//! Work package route handlers.
//!
//! - GET  /api/v1/features/:slug/work-packages      → list WPs for feature
//! - GET  /api/v1/work-packages/:id                 → detail
//! - POST /api/v1/features/:slug/work-packages      → create WP under feature
//! - PATCH /api/v1/work-packages/:id                → update
//! - POST /api/v1/work-packages/:id/transition      → state transition
//!
//! Traceability: WP11-T067

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use agileplus_domain::domain::work_package::{WorkPackage, WpState};
use agileplus_domain::ports::{observability::ObservabilityPort, storage::StoragePort, vcs::VcsPort};

use crate::error::ApiError;
use crate::responses::WorkPackageResponse;
use crate::state::AppState;

pub fn routes<S, V, O>() -> Router<AppState<S, V, O>>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    Router::new()
        .route("/{id}", get(get_work_package::<S, V, O>).patch(update_work_package::<S, V, O>))
        .route("/{id}/transition", post(transition_work_package::<S, V, O>))
}

/// Routes nested under `/api/v1/features/:slug/work-packages`.
pub fn feature_wp_routes<S, V, O>() -> Router<AppState<S, V, O>>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    Router::new()
        .route(
            "/{slug}/work-packages",
            get(list_work_packages::<S, V, O>).post(create_work_package::<S, V, O>),
        )
}

/// `GET /api/v1/work-packages/:id`
pub async fn get_work_package<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Path(id): Path<i64>,
) -> Result<Json<WorkPackageResponse>, ApiError>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    let wp = state
        .storage
        .get_work_package(id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("WorkPackage {id} not found")))?;

    Ok(Json(WorkPackageResponse::from(wp)))
}

/// `GET /api/v1/features/:slug/work-packages`
pub async fn list_work_packages<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Path(slug): Path<String>,
) -> Result<Json<Vec<WorkPackageResponse>>, ApiError>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    let feature = state
        .storage
        .get_feature_by_slug(&slug)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Feature '{slug}' not found")))?;

    let wps = state
        .storage
        .list_wps_by_feature(feature.id)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(wps.into_iter().map(WorkPackageResponse::from).collect()))
}

#[derive(Debug, Deserialize)]
pub struct CreateWpRequest {
    pub title: String,
    pub acceptance_criteria: Option<String>,
    pub sequence: Option<i32>,
}

/// `POST /api/v1/features/:slug/work-packages`
pub async fn create_work_package<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Path(slug): Path<String>,
    Json(body): Json<CreateWpRequest>,
) -> Result<(StatusCode, Json<WorkPackageResponse>), ApiError>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    let feature = app
        .storage
        .get_feature_by_slug(&slug)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("Feature '{slug}' not found")))?;

    let now = Utc::now();
    let wp = WorkPackage {
        id: 0,
        feature_id: feature.id,
        title: body.title,
        state: WpState::Planned,
        sequence: body.sequence.unwrap_or(1),
        file_scope: vec![],
        acceptance_criteria: body.acceptance_criteria.unwrap_or_default(),
        agent_id: None,
        pr_url: None,
        pr_state: None,
        worktree_path: None,
        plane_sub_issue_id: None,
        created_at: now,
        updated_at: now,
    };

    let id = app.storage.create_work_package(&wp).await.map_err(ApiError::from)?;
    let created = WorkPackage { id, ..wp };
    Ok((StatusCode::CREATED, Json(WorkPackageResponse::from(created))))
}

#[derive(Debug, Deserialize)]
pub struct UpdateWpRequest {
    pub title: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub pr_url: Option<String>,
}

/// `PATCH /api/v1/work-packages/:id`
pub async fn update_work_package<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateWpRequest>,
) -> Result<Json<WorkPackageResponse>, ApiError>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    let wp = app
        .storage
        .get_work_package(id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("WorkPackage {id} not found")))?;

    let updated = WorkPackage {
        title: body.title.unwrap_or(wp.title.clone()),
        acceptance_criteria: body.acceptance_criteria.unwrap_or(wp.acceptance_criteria.clone()),
        pr_url: body.pr_url.or(wp.pr_url.clone()),
        updated_at: Utc::now(),
        ..wp
    };

    Ok(Json(WorkPackageResponse::from(updated)))
}

#[derive(Debug, Deserialize)]
pub struct WpTransitionRequest {
    pub target_state: String,
}

#[derive(Debug, Serialize)]
pub struct WpTransitionResponse {
    pub wp_id: i64,
    pub from_state: String,
    pub to_state: String,
}

/// `POST /api/v1/work-packages/:id/transition`
pub async fn transition_work_package<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Path(id): Path<i64>,
    Json(body): Json<WpTransitionRequest>,
) -> Result<Json<WpTransitionResponse>, ApiError>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    let wp = app
        .storage
        .get_work_package(id)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("WorkPackage {id} not found")))?;

    let target = parse_wp_state(&body.target_state)?;
    if !wp.state.can_transition_to(target) {
        return Err(ApiError::Conflict(format!(
            "invalid transition {:?} -> {:?}",
            wp.state, target
        )));
    }

    let from_state = format!("{:?}", wp.state).to_lowercase();
    app.storage
        .update_wp_state(id, target)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(WpTransitionResponse {
        wp_id: id,
        from_state,
        to_state: format!("{:?}", target).to_lowercase(),
    }))
}

fn parse_wp_state(s: &str) -> Result<WpState, ApiError> {
    match s.to_lowercase().as_str() {
        "planned" => Ok(WpState::Planned),
        "doing" => Ok(WpState::Doing),
        "review" => Ok(WpState::Review),
        "done" => Ok(WpState::Done),
        "blocked" => Ok(WpState::Blocked),
        other => Err(ApiError::BadRequest(format!("Unknown WP state: {other}"))),
    }
}
