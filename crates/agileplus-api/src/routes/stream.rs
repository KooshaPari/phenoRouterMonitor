//! Server-Sent Events (SSE) streaming endpoint.
//!
//! `GET /api/v1/stream` — real-time domain events via SSE.
//!
//! Clients subscribe to the broadcast channel on connection; events published
//! via `AppState::event_tx` are forwarded as SSE frames.
//!
//! Traceability: WP11-T069

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::IntoResponse;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;

use crate::state::AppState;
use agileplus_domain::ports::{observability::ObservabilityPort, storage::StoragePort, vcs::VcsPort};

/// `GET /api/v1/stream`
///
/// Opens an SSE connection and streams domain events to the client.
/// Requires a valid API key (enforced by the surrounding middleware layer).
pub async fn stream_events<S, V, O>(
    State(app): State<AppState<S, V, O>>,
) -> impl IntoResponse
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    let rx = app.event_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| {
        match msg {
            Ok(payload) => {
                // payload is a JSON string: {"event_type": "...", "data": {...}}
                let evt_type = payload
                    .get("event_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("event")
                    .to_string();
                let data = payload
                    .get("data")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null)
                    .to_string();
                Some(Ok::<Event, std::convert::Infallible>(
                    Event::default().event(evt_type).data(data),
                ))
            }
            Err(_lagged) => {
                // Receiver lagged (missed messages) — send a keepalive comment.
                Some(Ok(Event::default().comment("lagged")))
            }
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
