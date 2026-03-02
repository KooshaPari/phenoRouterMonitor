//! Event query route handlers.
//!
//! - GET /api/v1/events        → paginated, filtered list of domain events
//! - GET /api/v1/events/:id    → single event detail
//!
//! The StoragePort does not yet expose event querying directly — this
//! module uses the audit trail as the event source and applies in-memory
//! filters until a dedicated EventStore port is wired up.
//!
//! Traceability: WP11-T068

use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use agileplus_domain::ports::{observability::ObservabilityPort, storage::StoragePort, vcs::VcsPort};

use crate::error::ApiError;
use crate::state::AppState;

pub fn routes<S, V, O>() -> Router<AppState<S, V, O>>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    Router::new()
        .route("/", get(list_events::<S, V, O>))
        .route("/{id}", get(get_event::<S, V, O>))
}

/// Query params for the event list endpoint.
#[derive(Debug, Deserialize)]
pub struct EventListParams {
    /// Filter by entity type: "feature" or "work_package".
    pub entity_type: Option<String>,
    /// Filter by entity id.
    pub entity_id: Option<i64>,
    /// ISO-8601 datetime or relative duration like "1h", "24h".
    pub since: Option<String>,
    /// ISO-8601 datetime upper bound.
    pub until: Option<String>,
    /// Filter by event/transition type string.
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    /// Filter by actor name.
    pub actor: Option<String>,
    /// Maximum number of results (default 100).
    pub limit: Option<usize>,
    /// Offset for pagination (default 0).
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct EventResponse {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: i64,
    pub event_type: String,
    pub actor: String,
    pub timestamp: String,
    pub payload: serde_json::Value,
}

/// `GET /api/v1/events`
pub async fn list_events<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Query(params): Query<EventListParams>,
) -> Result<Json<Vec<EventResponse>>, ApiError>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    // Gather audit entries from all features as an event source.
    let all_features = app
        .storage
        .list_all_features()
        .await
        .map_err(ApiError::from)?;

    let mut events: Vec<EventResponse> = Vec::new();
    for feature in &all_features {
        let trail = app
            .storage
            .get_audit_trail(feature.id)
            .await
            .map_err(ApiError::from)?;
        for entry in trail {
            events.push(EventResponse {
                id: entry.id,
                entity_type: "feature".to_string(),
                entity_id: entry.feature_id,
                event_type: entry.transition.clone(),
                actor: entry.actor.clone(),
                timestamp: entry.timestamp.to_rfc3339(),
                payload: serde_json::json!({
                    "wp_id": entry.wp_id,
                    "transition": entry.transition,
                }),
            });
        }
        // Include work-package events.
        let wps = app
            .storage
            .list_wps_by_feature(feature.id)
            .await
            .map_err(ApiError::from)?;
        for wp in wps {
            events.push(EventResponse {
                id: wp.id,
                entity_type: "work_package".to_string(),
                entity_id: wp.id,
                event_type: format!("wp_{:?}", wp.state).to_lowercase(),
                actor: "system".to_string(),
                timestamp: wp.updated_at.to_rfc3339(),
                payload: serde_json::json!({
                    "title": wp.title,
                    "state": format!("{:?}", wp.state).to_lowercase(),
                }),
            });
        }
    }

    // Apply filters.
    let since_dt = params.since.as_deref().map(parse_since).transpose()?;
    let until_dt: Option<DateTime<Utc>> = params
        .until
        .as_deref()
        .map(|s| s.parse::<DateTime<Utc>>().map_err(|e| ApiError::BadRequest(format!("invalid until: {e}"))))
        .transpose()?;

    let filtered: Vec<EventResponse> = events
        .into_iter()
        .filter(|e| {
            if let Some(et) = &params.entity_type {
                if &e.entity_type != et {
                    return false;
                }
            }
            if let Some(eid) = params.entity_id {
                if e.entity_id != eid {
                    return false;
                }
            }
            if let Some(ev_type) = &params.event_type {
                if &e.event_type != ev_type {
                    return false;
                }
            }
            if let Some(actor) = &params.actor {
                if &e.actor != actor {
                    return false;
                }
            }
            if let Some(since) = since_dt {
                if let Ok(ts) = e.timestamp.parse::<DateTime<Utc>>() {
                    if ts < since {
                        return false;
                    }
                }
            }
            if let Some(until) = until_dt {
                if let Ok(ts) = e.timestamp.parse::<DateTime<Utc>>() {
                    if ts > until {
                        return false;
                    }
                }
            }
            true
        })
        .collect();

    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(100);

    let page: Vec<EventResponse> = filtered.into_iter().skip(offset).take(limit).collect();
    Ok(Json(page))
}

/// `GET /api/v1/events/:id`
pub async fn get_event<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Path(id): Path<i64>,
) -> Result<Json<EventResponse>, ApiError>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    // Scan audit entries across all features for the matching id.
    let all_features = app
        .storage
        .list_all_features()
        .await
        .map_err(ApiError::from)?;

    for feature in &all_features {
        let trail = app
            .storage
            .get_audit_trail(feature.id)
            .await
            .map_err(ApiError::from)?;
        if let Some(entry) = trail.into_iter().find(|e| e.id == id) {
            return Ok(Json(EventResponse {
                id: entry.id,
                entity_type: "feature".to_string(),
                entity_id: entry.feature_id,
                event_type: entry.transition.clone(),
                actor: entry.actor.clone(),
                timestamp: entry.timestamp.to_rfc3339(),
                payload: serde_json::json!({
                    "wp_id": entry.wp_id,
                    "transition": entry.transition,
                    "evidence_refs": entry.evidence_refs,
                }),
            }));
        }
    }

    Err(ApiError::NotFound(format!("Event {id} not found")))
}

/// Parse a `since` parameter: ISO-8601 datetime or relative like "1h", "24h".
fn parse_since(s: &str) -> Result<DateTime<Utc>, ApiError> {
    if let Ok(dt) = s.parse::<DateTime<Utc>>() {
        return Ok(dt);
    }
    // Try relative durations like "1h", "24h", "30m".
    let (num, unit) = if s.ends_with('h') {
        (
            s[..s.len() - 1]
                .parse::<i64>()
                .map_err(|_| ApiError::BadRequest(format!("invalid since: {s}")))?,
            "h",
        )
    } else if s.ends_with('m') {
        (
            s[..s.len() - 1]
                .parse::<i64>()
                .map_err(|_| ApiError::BadRequest(format!("invalid since: {s}")))?,
            "m",
        )
    } else {
        return Err(ApiError::BadRequest(format!("invalid since: {s}")));
    };
    let duration = match unit {
        "h" => Duration::hours(num),
        "m" => Duration::minutes(num),
        _ => unreachable!(),
    };
    Ok(Utc::now() - duration)
}
