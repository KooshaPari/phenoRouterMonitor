//! Error types for agileplus-p2p.

use thiserror::Error;

/// Errors that occur during peer discovery via Tailscale.
#[derive(Debug, Error)]
pub enum PeerDiscoveryError {
    #[error("Tailscale local API unavailable: {0}")]
    ApiUnavailable(String),

    #[error("HTTP error querying Tailscale: {0}")]
    HttpError(String),

    #[error("Failed to parse Tailscale response: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported platform")]
    UnsupportedPlatform,
}

/// Errors during P2P synchronization.
#[derive(Debug, Error)]
pub enum SyncError {
    #[error("NATS connection failed for peer {peer_id}: {reason}")]
    ConnectionFailed { peer_id: String, reason: String },

    #[error("NATS publish failed: {0}")]
    PublishFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Event store error: {0}")]
    EventStore(String),

    #[error("Peer discovery error: {0}")]
    Discovery(#[from] PeerDiscoveryError),

    #[error("NATS error: {0}")]
    Nats(String),

    #[error("Timeout connecting to peer {peer_id}")]
    Timeout { peer_id: String },
}

impl From<async_nats::Error> for SyncError {
    fn from(e: async_nats::Error) -> Self {
        SyncError::Nats(e.to_string())
    }
}

impl From<async_nats::ConnectError> for SyncError {
    fn from(e: async_nats::ConnectError) -> Self {
        SyncError::Nats(e.to_string())
    }
}

/// Errors during device registration.
#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Could not connect to device registry: {0}")]
    RegistryUnavailable(String),

    #[error("Device already registered with different ID")]
    ConflictingRegistration,

    #[error("SQLite error: {0}")]
    Database(String),

    #[error("Tailscale query failed: {0}")]
    TailscaleQuery(#[from] PeerDiscoveryError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
