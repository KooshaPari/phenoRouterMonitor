//! agileplus-p2p — Peer-to-peer sync via Tailscale and NATS.
//!
//! Provides:
//! - Peer discovery via the Tailscale local API UNIX socket (`discovery`)
//! - Persistent device identity (`device`)
//! - Event replication over NATS JetStream (`replication`)
//! - Vector-clock-based synchronisation (`vector_clock`)
//! - Git-backed state export, import, and merge conflict resolution (`export`, `import`, `git_merge`)
//!
//! Traceability: WP16 / T095-T099, WP17 / T100-T103

pub mod device;
pub mod discovery;
pub mod error;
pub mod export;
pub mod git_merge;
pub mod import;
pub mod replication;
pub mod vector_clock;

pub use device::{DeviceNode, DeviceStore, InMemoryDeviceStore, get_local_device, register_device};
pub use discovery::{PeerInfo, PeerStatus, discover_peers};
pub use error::{ConnectionError, PeerDiscoveryError, SyncError};
pub use export::{EntityRef, ExportError, ExportStats, export_state};
pub use git_merge::{ConflictResolution, MergeError, resolve_git_conflicts};
pub use import::{ImportError, ImportStats, import_state};
pub use replication::{EventBatch, ReplicationResult, replicate_events};
pub use vector_clock::{SyncResult, SyncVector, sync_with_peer};
