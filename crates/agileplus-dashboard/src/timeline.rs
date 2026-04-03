use axum::{
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use crate::app_state::SharedState;
use crate::templates::{EventTimelinePartial, FeatureView, WpView};
use crate::routes::{render, build_feature_events};

pub async fn event_timeline(State(state): State<SharedState>) -> Response {
    let _ = state.read().await;
    render(EventTimelinePartial {
        feature_id: 0,
        events: vec![],
    })
}

pub async fn feature_events(
    State(state): State<SharedState>,
    axum::extract::Path(feature_id): axum::extract::Path<i64>,
) -> Response {
    let store = state.read().await;
    let feature = match store.features.iter().find(|f| f.id == feature_id) {
        Some(f) => FeatureView::from_feature(f),
        None => return (axum::http::StatusCode::NOT_FOUND, "Feature not found").into_response(),
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

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/api/dashboard/events", get(event_timeline))
        .route("/api/dashboard/features/{id}/events", get(feature_events))
}