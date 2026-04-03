//! Session management utilities

use crate::{AuthError, Session, SessionStorage};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Session manager for handling user sessions
pub struct SessionManager<S: SessionStorage> {
    storage: Arc<S>,
    ttl_seconds: u64,
}

impl<S: SessionStorage> SessionManager<S> {
    /// Create a new session manager
    pub fn new(storage: Arc<S>, ttl_seconds: u64) -> Self {
        Self {
            storage,
            ttl_seconds,
        }
    }

    /// Create a new session for a user
    pub async fn create_session(&self, user_id: impl Into<String>) -> Result<Session, AuthError> {
        let session = Session {
            id: generate_session_id(),
            user_id: user_id.into(),
            expires_at: current_timestamp() + self.ttl_seconds,
        };
        self.storage.store_session(&session).await?;
        Ok(session)
    }

    /// Validate a session by ID
    pub async fn validate_session(&self, session_id: &str) -> Result<Session, AuthError> {
        match self.storage.get_session(session_id).await? {
            Some(session) => {
                if current_timestamp() > session.expires_at {
                    return Err(AuthError::ExpiredToken);
                }
                Ok(session)
            }
            None => Err(AuthError::SessionNotFound),
        }
    }

    /// Invalidate a session
    pub async fn invalidate_session(&self, session_id: &str) -> Result<(), AuthError> {
        self.storage.delete_session(session_id).await
    }
}

fn generate_session_id() -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    current_timestamp().hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
