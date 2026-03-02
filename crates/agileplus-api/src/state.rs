//! Shared application state threaded through every axum handler.
//!
//! Traceability: WP11-T069

use std::sync::Arc;

use agileplus_domain::config::AppConfig;
use agileplus_domain::credentials::CredentialStore;
use agileplus_domain::ports::{observability::ObservabilityPort, storage::StoragePort, vcs::VcsPort};
use tokio::sync::broadcast;

/// Broadcast channel capacity for SSE event streaming.
const EVENT_CHANNEL_CAPACITY: usize = 256;

/// Shared state injected into every axum handler via `State<AppState<…>>`.
#[derive(Clone)]
pub struct AppState<S, V, O>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    pub storage: Arc<S>,
    pub vcs: Arc<V>,
    pub telemetry: Arc<O>,
    pub config: Arc<AppConfig>,
    pub credentials: Arc<dyn CredentialStore>,
    /// Broadcast sender for real-time SSE event streaming (T069).
    /// Publish JSON objects with `event_type` and `data` keys.
    pub event_tx: broadcast::Sender<serde_json::Value>,
}

impl<S, V, O> AppState<S, V, O>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    pub fn new(
        storage: Arc<S>,
        vcs: Arc<V>,
        telemetry: Arc<O>,
        config: Arc<AppConfig>,
        credentials: Arc<dyn CredentialStore>,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Self {
            storage,
            vcs,
            telemetry,
            config,
            credentials,
            event_tx,
        }
    }

    /// Create state with an explicit broadcast sender (allows sharing the channel
    /// with other subsystems such as a NATS bridge).
    pub fn with_event_tx(
        storage: Arc<S>,
        vcs: Arc<V>,
        telemetry: Arc<O>,
        config: Arc<AppConfig>,
        credentials: Arc<dyn CredentialStore>,
        event_tx: broadcast::Sender<serde_json::Value>,
    ) -> Self {
        Self {
            storage,
            vcs,
            telemetry,
            config,
            credentials,
            event_tx,
        }
    }
}
