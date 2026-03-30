//! Synchronization error type for P2P and replication operations.

use thiserror::Error;

/// Error type for synchronization and P2P operations.
#[derive(Debug, Clone, Error)]
pub enum SyncError {
    #[error("conflict: {0}")]
    Conflict(String),

    #[error("merge error: {0}")]
    MergeError(String),

    #[error("replication error: {0}")]
    ReplicationError(String),

    #[error("peer error: {0}")]
    PeerError(String),

    #[error("version mismatch: {0}")]
    VersionMismatch(String),

    #[error("sync error: {0}")]
    Internal(String),
}

impl SyncError {
    pub fn conflict(msg: impl Into<String>) -> Self { Self::Conflict(msg.into()) }
    pub fn merge_error(msg: impl Into<String>) -> Self { Self::MergeError(msg.into()) }
    pub fn replication_error(msg: impl Into<String>) -> Self { Self::ReplicationError(msg.into()) }
    pub fn peer_error(msg: impl Into<String>) -> Self { Self::PeerError(msg.into()) }
    pub fn version_mismatch(msg: impl Into<String>) -> Self { Self::VersionMismatch(msg.into()) }
    pub fn internal(msg: impl Into<String>) -> Self { Self::Internal(msg.into()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_error() { let err = SyncError::conflict("both peers modified same field"); assert_eq!(err.to_string(), "conflict: both peers modified same field"); }

    #[test]
    fn test_merge_error() { let err = SyncError::merge_error("three-way merge failed"); assert_eq!(err.to_string(), "merge error: three-way merge failed"); }

    #[test]
    fn test_replication_error() { let err = SyncError::replication_error("log sync failed"); assert_eq!(err.to_string(), "replication error: log sync failed"); }

    #[test]
    fn test_peer_error() { let err = SyncError::peer_error("peer disconnected"); assert_eq!(err.to_string(), "peer error: peer disconnected"); }

    #[test]
    fn test_version_mismatch_error() { let err = SyncError::version_mismatch("expected v2, got v1"); assert_eq!(err.to_string(), "version mismatch: expected v2, got v1"); }

    #[test]
    fn test_internal_error() { let err = SyncError::internal("internal sync state corrupted"); assert_eq!(err.to_string(), "sync error: internal sync state corrupted"); }
}
