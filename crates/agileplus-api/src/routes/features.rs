//! Feature route handlers: GET /api/v1/features, GET /api/v1/features/:slug
//!
//! Traceability: WP15-T086

use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::ports::{observability::ObservabilityPort, storage::StoragePort, vcs::VcsPort};

use crate::error::ApiError;
use crate::responses::FeatureResponse;
use crate::state::AppState;

pub fn routes<S, V, O>() -> Router<AppState<S, V, O>>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    Router::new()
        .route("/", get(list_features::<S, V, O>))
        .route("/{slug}", get(get_feature::<S, V, O>))
}

#[derive(Debug, Deserialize)]
pub struct FeatureListParams {
    pub state: Option<String>,
}

/// `GET /api/v1/features[?state=<state>]`
pub async fn list_features<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Query(params): Query<FeatureListParams>,
) -> Result<Json<Vec<FeatureResponse>>, ApiError>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    let features = if let Some(state_filter) = params.state {
        let fs = parse_feature_state(&state_filter)?;
        state
            .storage
            .list_features_by_state(fs)
            .await
            .map_err(ApiError::from)?
    } else {
        state
            .storage
            .list_all_features()
            .await
            .map_err(ApiError::from)?
    };

    Ok(Json(features.into_iter().map(FeatureResponse::from).collect()))
}

/// `GET /api/v1/features/:slug`
pub async fn get_feature<S, V, O>(
    State(state): State<AppState<S, V, O>>,
    Path(slug): Path<String>,
) -> Result<Json<FeatureResponse>, ApiError>
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

    Ok(Json(FeatureResponse::from(feature)))
}

fn parse_feature_state(s: &str) -> Result<FeatureState, ApiError> {
    match s.to_lowercase().as_str() {
        "created" => Ok(FeatureState::Created),
        "specified" => Ok(FeatureState::Specified),
        "researched" => Ok(FeatureState::Researched),
        "planned" => Ok(FeatureState::Planned),
        "implementing" => Ok(FeatureState::Implementing),
        "validated" => Ok(FeatureState::Validated),
        "shipped" => Ok(FeatureState::Shipped),
        "retrospected" => Ok(FeatureState::Retrospected),
        other => Err(ApiError::BadRequest(format!("Unknown feature state: {other}"))),
    }
}
