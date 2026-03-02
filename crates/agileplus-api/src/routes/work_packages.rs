//! Work package route handlers.
//!
//! - GET /api/v1/features/:slug/work-packages
//! - GET /api/v1/work-packages/:id
//!
//! Traceability: WP15-T086

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};

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
        .route("/{id}", get(get_work_package::<S, V, O>))
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
