//! Device registration — persistent identity for the local AgilePlus instance.
//!
//! On first startup a UUID v4 `device_id` is generated and the local
//! Tailscale hostname / IP are fetched from the daemon.  The record is stored
//! so the same identity is recovered across restarts.
//!
//! Traceability: WP16 / T097

use bytes::Bytes;
use chrono::{DateTime, Utc};
use http_body_util::{BodyExt as _, Empty};
use hyper::Request;
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use tokio::net::UnixStream;
use tracing::{debug, info};
use uuid::Uuid;

use crate::discovery::tailscale_socket_path;
use crate::error::{ConnectionError, PeerDiscoveryError};

/// Persistent identity for one AgilePlus node in the tailnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceNode {
    pub device_id: String,
    pub hostname: String,
    pub tailscale_ip: String,
    pub created_at: DateTime<Utc>,
}

// ── Persistence trait ─────────────────────────────────────────────────────────

/// Minimal persistence interface for device records.
pub trait DeviceStore: Send + Sync {
    fn insert_device(&self, device: &DeviceNode) -> Result<(), ConnectionError>;
    fn get_device(&self) -> Result<Option<DeviceNode>, ConnectionError>;
}

// ── Tailscale self-info ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct TailscaleStatusSelf {
    #[serde(rename = "Self")]
    self_node: TailscaleSelf,
}

#[derive(Debug, Deserialize)]
struct TailscaleSelf {
    #[serde(rename = "DNSName", default)]
    dns_name: String,
    #[serde(rename = "TailscaleIPs", default)]
    tailscale_ips: Vec<String>,
}

async fn query_local_tailscale() -> Result<(String, String), PeerDiscoveryError> {
    let socket_path = tailscale_socket_path()?;

    let stream = UnixStream::connect(&socket_path).await.map_err(|e| {
        PeerDiscoveryError::ApiUnavailable(format!(
            "cannot connect to Tailscale socket {}: {}",
            socket_path.display(),
            e
        ))
    })?;

    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(|e: hyper::Error| PeerDiscoveryError::HttpError(e.to_string()))?;

    tokio::spawn(async move {
        if let Err(e) = conn.await {
            tracing::warn!("Tailscale local API connection error: {e}");
        }
    });

    let req = Request::builder()
        .method("GET")
        .uri("/localapi/v0/status")
        .header("Host", "local-tailscaled.sock")
        .body(Empty::<Bytes>::new())
        .map_err(|e| PeerDiscoveryError::HttpError(e.to_string()))?;

    let resp = sender
        .send_request(req)
        .await
        .map_err(|e: hyper::Error| PeerDiscoveryError::HttpError(e.to_string()))?;

    let body_bytes = resp
        .into_body()
        .collect()
        .await
        .map_err(|e: hyper::Error| PeerDiscoveryError::HttpError(e.to_string()))?
        .to_bytes();

    let status: TailscaleStatusSelf = serde_json::from_slice(&body_bytes)?;
    let hostname = status.self_node.dns_name.trim_end_matches('.').to_string();
    let ip = status
        .self_node
        .tailscale_ips
        .into_iter()
        .next()
        .unwrap_or_default();

    Ok((hostname, ip))
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Register the local device, creating a new record if none exists.
pub async fn register_device(store: &dyn DeviceStore) -> Result<DeviceNode, ConnectionError> {
    if let Some(existing) = store.get_device()? {
        debug!("Found existing device registration: {}", existing.device_id);
        return Ok(existing);
    }

    let (hostname, tailscale_ip) = match query_local_tailscale().await {
        Ok(pair) => pair,
        Err(e) => {
            tracing::warn!("Tailscale query failed during registration, using fallbacks: {e}");
            let h = hostname::get()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|_| "unknown".to_string());
            (h, String::new())
        }
    };

    let device = DeviceNode {
        device_id: Uuid::new_v4().to_string(),
        hostname,
        tailscale_ip,
        created_at: Utc::now(),
    };

    info!("Registering new device: {}", device.device_id);
    store.insert_device(&device)?;
    Ok(device)
}

/// Retrieve the locally registered device without creating one.
pub fn get_local_device(store: &dyn DeviceStore) -> Result<Option<DeviceNode>, ConnectionError> {
    store.get_device()
}

// ── In-memory store for tests ─────────────────────────────────────────────────

/// Thread-safe in-memory `DeviceStore` for unit tests.
#[derive(Debug, Default)]
pub struct InMemoryDeviceStore {
    inner: std::sync::Mutex<Option<DeviceNode>>,
}

impl DeviceStore for InMemoryDeviceStore {
    fn insert_device(&self, device: &DeviceNode) -> Result<(), ConnectionError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| ConnectionError::Database(e.to_string()))?;
        if guard.is_some() {
            return Err(ConnectionError::ConflictingRegistration);
        }
        *guard = Some(device.clone());
        Ok(())
    }

    fn get_device(&self) -> Result<Option<DeviceNode>, ConnectionError> {
        let guard = self
            .inner
            .lock()
            .map_err(|e| ConnectionError::Database(e.to_string()))?;
        Ok(guard.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_creates_device_with_uuid() {
        let store = InMemoryDeviceStore::default();
        let device = register_device(&store).await.unwrap();
        assert!(!device.device_id.is_empty());
        Uuid::parse_str(&device.device_id).expect("device_id must be a valid UUID");
    }

    #[tokio::test]
    async fn register_idempotent() {
        let store = InMemoryDeviceStore::default();
        let d1 = register_device(&store).await.unwrap();
        let d2 = register_device(&store).await.unwrap();
        assert_eq!(d1.device_id, d2.device_id);
    }

    #[tokio::test]
    async fn get_local_device_before_registration_returns_none() {
        let store = InMemoryDeviceStore::default();
        let result = get_local_device(&store).unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_local_device_after_registration_returns_some() {
        let store = InMemoryDeviceStore::default();
        let registered = register_device(&store).await.unwrap();
        let fetched = get_local_device(&store).unwrap().unwrap();
        assert_eq!(registered.device_id, fetched.device_id);
    }

    #[test]
    fn in_memory_store_prevents_double_insert() {
        let store = InMemoryDeviceStore::default();
        let device = DeviceNode {
            device_id: "abc".to_string(),
            hostname: "host".to_string(),
            tailscale_ip: "100.64.0.1".to_string(),
            created_at: Utc::now(),
        };
        store.insert_device(&device).unwrap();
        let result = store.insert_device(&device);
        assert!(matches!(result, Err(ConnectionError::ConflictingRegistration)));
    }
}
