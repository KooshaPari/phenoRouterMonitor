//! Peer discovery via Tailscale local API.
//!
//! Connects to the Tailscale daemon's local UNIX socket and queries
//! `/localapi/v0/status` to enumerate peers on the tailnet.
//! Traceability: WP16 / T096

use bytes::Bytes;
use http_body_util::{BodyExt as _, Empty};
use hyper::Request;
use hyper_util::rt::TokioIo;
use serde::Deserialize;
use tokio::net::UnixStream;
use tracing::{debug, warn};

use crate::error::PeerDiscoveryError;

/// Information about a discovered peer on the Tailscale network.
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub device_id: String,
    pub hostname: String,
    pub tailscale_ip: String,
    pub status: PeerStatus,
}

/// Availability status of a discovered peer.
#[derive(Debug, Clone, PartialEq)]
pub enum PeerStatus {
    /// Peer is online and AgilePlus is detected.
    Online,
    /// Peer is reachable on the tailnet but AgilePlus is not running.
    Offline,
    /// Peer status could not be determined.
    Unknown,
}

// ── Tailscale JSON response shapes ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TailscaleStatus {
    #[serde(rename = "Peer", default)]
    peer: std::collections::HashMap<String, TailscalePeer>,
}

#[derive(Debug, Deserialize)]
struct TailscalePeer {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "DNSName", default)]
    dns_name: String,
    #[serde(rename = "TailscaleIPs", default)]
    tailscale_ips: Vec<String>,
    #[serde(rename = "Online", default)]
    online: bool,
}

// ── Socket path resolution ───────────────────────────────────────────────────

/// Return the path of the Tailscale daemon UNIX socket for the current platform.
pub fn tailscale_socket_path() -> Result<std::path::PathBuf, PeerDiscoveryError> {
    #[cfg(target_os = "linux")]
    {
        Ok(std::path::PathBuf::from("/var/run/tailscale/tailscaled.sock"))
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(p) = std::env::var("TAILSCALE_SOCKET") {
            return Ok(std::path::PathBuf::from(p));
        }
        Ok(std::path::PathBuf::from(
            "/var/run/tailscale/tailscaled.sock",
        ))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Err(PeerDiscoveryError::UnsupportedPlatform)
    }
}

// ── HTTP-over-UNIX-socket client ─────────────────────────────────────────────

async fn tailscale_get(path: &str) -> Result<String, PeerDiscoveryError> {
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
            warn!("Tailscale local API connection error: {e}");
        }
    });

    let req = Request::builder()
        .method("GET")
        .uri(path)
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

    Ok(String::from_utf8_lossy(&body_bytes).into_owned())
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Discover peers on the local Tailscale network.
pub async fn discover_peers() -> Result<Vec<PeerInfo>, PeerDiscoveryError> {
    let body = tailscale_get("/localapi/v0/status").await?;
    debug!("Tailscale status response: {} bytes", body.len());

    let status: TailscaleStatus = serde_json::from_str(&body)?;
    let mut peers = Vec::new();

    for (_key, peer) in status.peer {
        let tailscale_ip = peer.tailscale_ips.into_iter().next().unwrap_or_default();
        if tailscale_ip.is_empty() {
            continue;
        }

        let hostname = peer.dns_name.trim_end_matches('.').to_string();

        let status = if peer.online {
            probe_agileplus(&tailscale_ip).await
        } else {
            PeerStatus::Offline
        };

        peers.push(PeerInfo {
            device_id: peer.id,
            hostname,
            tailscale_ip,
            status,
        });
    }

    Ok(peers)
}

/// Attempt a short TCP connection to `ip:3000` to detect AgilePlus.
async fn probe_agileplus(ip: &str) -> PeerStatus {
    use tokio::net::TcpStream;
    use tokio::time::{Duration, timeout};

    let addr = format!("{ip}:3000");
    match timeout(Duration::from_secs(2), TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => PeerStatus::Online,
        Ok(Err(_)) => PeerStatus::Offline,
        Err(_) => PeerStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peer_status_equality() {
        assert_eq!(PeerStatus::Online, PeerStatus::Online);
        assert_ne!(PeerStatus::Online, PeerStatus::Offline);
    }

    #[test]
    fn socket_path_not_empty() {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let p = tailscale_socket_path().unwrap();
            assert!(!p.as_os_str().is_empty());
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            assert!(tailscale_socket_path().is_err());
        }
    }
}
