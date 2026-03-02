//! Vector-clock-based synchronisation between AgilePlus devices.
//!
//! Each device maintains a `SyncVector` — a map from `(entity_type, entity_id)`
//! to the highest event sequence number that device has seen for that entity.
//! Two devices exchange their vectors, compute the symmetric difference, and
//! transfer only the missing events.
//!
//! Traceability: WP16 / T099

use std::collections::HashMap;

use agileplus_domain::domain::event::Event;
use agileplus_events::store::{EventError, EventStore};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::discovery::PeerInfo;
use crate::error::SyncError;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Per-entity watermark vector for one device.
///
/// `entries[(entity_type, entity_id)] = last_sequence` — the highest event
/// sequence number this device has applied for that entity stream.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncVector {
    pub device_id: String,
    /// Map key is `(entity_type, entity_id)` serialised as `"type/id"`.
    pub entries: HashMap<(String, String), u64>,
}

impl SyncVector {
    pub fn new(device_id: impl Into<String>) -> Self {
        Self {
            device_id: device_id.into(),
            entries: HashMap::new(),
        }
    }

    /// Record that we have seen `sequence` for `(entity_type, entity_id)`.
    pub fn advance(&mut self, entity_type: &str, entity_id: &str, sequence: u64) {
        let key = (entity_type.to_string(), entity_id.to_string());
        let entry = self.entries.entry(key).or_insert(0);
        if sequence > *entry {
            *entry = sequence;
        }
    }

    /// Merge another vector into self by taking the per-key maximum.
    pub fn merge(&mut self, other: &SyncVector) {
        for (k, &v) in &other.entries {
            let entry = self.entries.entry(k.clone()).or_insert(0);
            if v > *entry {
                *entry = v;
            }
        }
    }

    /// Return the last-known sequence for an entity (0 if unknown).
    pub fn get(&self, entity_type: &str, entity_id: &str) -> u64 {
        self.entries
            .get(&(entity_type.to_string(), entity_id.to_string()))
            .copied()
            .unwrap_or(0)
    }
}

/// Result of a full sync round-trip with one peer.
#[derive(Debug, Default)]
pub struct SyncResult {
    pub events_sent: usize,
    pub events_received: usize,
    pub conflicts_detected: usize,
    pub updated_vector: SyncVector,
}

// ── Sync algorithm ────────────────────────────────────────────────────────────

/// Compare two sync vectors and return the set of entity keys where the local
/// device has events the peer is missing (`local_seq > peer_seq`).
pub fn compute_missing_locally(
    local: &SyncVector,
    peer: &SyncVector,
) -> Vec<(String, String, u64, u64)> {
    // (entity_type, entity_id, peer_seq, local_seq)
    let mut missing = Vec::new();
    for ((et, eid), &local_seq) in &local.entries {
        let peer_seq = peer.get(et, eid);
        if local_seq > peer_seq {
            missing.push((et.clone(), eid.clone(), peer_seq, local_seq));
        }
    }
    missing
}

/// Fetch events from `event_store` for entities where local has more than peer.
async fn fetch_events_to_send(
    local_vector: &SyncVector,
    peer_vector: &SyncVector,
    event_store: &dyn EventStore,
) -> Result<Vec<Event>, EventError> {
    let missing = compute_missing_locally(local_vector, peer_vector);
    let mut events_to_send = Vec::new();
    for (entity_type, entity_id_str, peer_seq, _local_seq) in missing {
        // entity_id is stored as String in the vector but as i64 in EventStore.
        let entity_id: i64 = entity_id_str.parse().unwrap_or(0);
        let evts = event_store
            .get_events_since(&entity_type, entity_id, peer_seq as i64)
            .await?;
        events_to_send.extend(evts);
    }
    Ok(events_to_send)
}

/// Full sync with a single peer.
///
/// In a real deployment the `peer_vector` would be exchanged over NATS.  Here
/// we accept it as a parameter so the function is testable without a live peer.
pub async fn sync_with_peer_vectors(
    local_device_id: &str,
    peer: &PeerInfo,
    local_vector: &SyncVector,
    peer_vector: &SyncVector,
    event_store: &dyn EventStore,
) -> Result<SyncResult, SyncError> {
    info!(
        "Syncing with peer {} ({})",
        peer.device_id, peer.tailscale_ip
    );

    // 1. Determine what to send.
    let events_to_send = fetch_events_to_send(local_vector, peer_vector, event_store)
        .await
        .map_err(|e| SyncError::EventStore(e.to_string()))?;

    debug!(
        "Will send {} events to peer {}",
        events_to_send.len(),
        peer.device_id
    );

    // 2. Replicate events to peer via NATS.
    let rep_result =
        crate::replication::replicate_events(local_device_id, peer, events_to_send.clone())
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("Replication failed for peer {}: {e}", peer.device_id);
                crate::replication::ReplicationResult::default()
            });

    // 3. Build updated vector = max(local, peer).
    let mut updated = local_vector.clone();
    updated.merge(peer_vector);

    Ok(SyncResult {
        events_sent: rep_result.events_sent,
        events_received: rep_result.events_received,
        conflicts_detected: 0, // conflict resolution delegated to SyncOrchestrator
        updated_vector: updated,
    })
}

/// Convenience wrapper matching the spec signature.
///
/// Accepts peer vector inline; the caller is responsible for exchanging
/// vectors with the peer (e.g. via an initial NATS request/reply).
pub async fn sync_with_peer(
    local_device_id: &str,
    peer: &PeerInfo,
    local_vector: &SyncVector,
    event_store: &dyn EventStore,
) -> Result<SyncResult, SyncError> {
    // In production the peer vector would be fetched from the peer over NATS.
    // We start with an empty vector (peer has nothing) so we send everything.
    let empty_peer_vector = SyncVector::new(&peer.device_id);
    sync_with_peer_vectors(
        local_device_id,
        peer,
        local_vector,
        &empty_peer_vector,
        event_store,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── SyncVector unit tests ────────────────────────────────────────────────

    #[test]
    fn advance_sets_initial_value() {
        let mut v = SyncVector::new("dev-a");
        v.advance("Feature", "1", 5);
        assert_eq!(v.get("Feature", "1"), 5);
    }

    #[test]
    fn advance_only_moves_forward() {
        let mut v = SyncVector::new("dev-a");
        v.advance("Feature", "1", 10);
        v.advance("Feature", "1", 3); // lower — should not regress
        assert_eq!(v.get("Feature", "1"), 10);
    }

    #[test]
    fn get_returns_zero_for_unknown_entity() {
        let v = SyncVector::new("dev-a");
        assert_eq!(v.get("Epic", "999"), 0);
    }

    #[test]
    fn merge_takes_maximum() {
        let mut a = SyncVector::new("dev-a");
        a.advance("Feature", "1", 5);
        a.advance("Epic", "2", 3);

        let mut b = SyncVector::new("dev-b");
        b.advance("Feature", "1", 8);
        b.advance("Story", "3", 2);

        a.merge(&b);
        assert_eq!(a.get("Feature", "1"), 8); // b had higher value
        assert_eq!(a.get("Epic", "2"), 3);     // a had higher value
        assert_eq!(a.get("Story", "3"), 2);    // new key from b
    }

    #[test]
    fn merge_is_commutative() {
        let mut a = SyncVector::new("dev-a");
        a.advance("Feature", "1", 5);

        let mut b = SyncVector::new("dev-b");
        b.advance("Feature", "1", 8);

        let mut a_then_b = a.clone();
        a_then_b.merge(&b);

        let mut b_then_a = b.clone();
        b_then_a.merge(&a);

        assert_eq!(
            a_then_b.get("Feature", "1"),
            b_then_a.get("Feature", "1")
        );
    }

    // ── compute_missing_locally tests ────────────────────────────────────────

    #[test]
    fn missing_events_detected_correctly() {
        let mut local = SyncVector::new("dev-a");
        local.advance("Feature", "1", 10);
        local.advance("Epic", "2", 5);

        let mut peer = SyncVector::new("dev-b");
        peer.advance("Feature", "1", 7); // behind
        peer.advance("Epic", "2", 5);    // equal — no transfer needed

        let missing = compute_missing_locally(&local, &peer);
        assert_eq!(missing.len(), 1);
        let (et, eid, peer_seq, local_seq) = &missing[0];
        assert_eq!(et, "Feature");
        assert_eq!(eid, "1");
        assert_eq!(*peer_seq, 7);
        assert_eq!(*local_seq, 10);
    }

    #[test]
    fn no_missing_events_when_vectors_equal() {
        let mut local = SyncVector::new("dev-a");
        local.advance("Feature", "1", 10);

        let peer = local.clone();

        let missing = compute_missing_locally(&local, &peer);
        assert!(missing.is_empty());
    }

    #[test]
    fn peer_ahead_generates_no_local_transfer() {
        let mut local = SyncVector::new("dev-a");
        local.advance("Feature", "1", 3);

        let mut peer = SyncVector::new("dev-b");
        peer.advance("Feature", "1", 10); // peer is ahead

        let missing = compute_missing_locally(&local, &peer);
        assert!(missing.is_empty(), "local should not send events it doesn't have");
    }
}
