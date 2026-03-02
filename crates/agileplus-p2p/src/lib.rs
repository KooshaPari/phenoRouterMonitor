//! agileplus-p2p — Peer-to-peer sync via Tailscale and NATS.
//!
//! Provides:
//! - Peer discovery via the Tailscale local API UNIX socket (`discovery`)
//! - Persistent device identity (`device`)
//! - Event replication over NATS JetStream (`replication`)
//! - Vector-clock-based synchronisation (`vector_clock`)
//!
//! Traceability: WP16 / T095-T099

pub mod device;
pub mod discovery;
pub mod error;
pub mod replication;
pub mod vector_clock;

pub use device::{DeviceNode, DeviceStore, InMemoryDeviceStore, get_local_device, register_device};
pub use discovery::{PeerInfo, PeerStatus, discover_peers};
pub use error::{ConnectionError, PeerDiscoveryError, SyncError};
pub use replication::{EventBatch, ReplicationResult, replicate_events};
pub use vector_clock::{SyncResult, SyncVector, sync_with_peer};
