use std::time::{SystemTime, Duration};

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub expires_at: SystemTime,
}

pub struct SessionManager {
    ttl: Duration,
}

impl SessionManager {
    pub fn new(ttl: Duration) -> Self { Self { ttl } }

    pub fn create_session(&self, user_id: &str) -> Session {
        // simple placeholder implementation
        Session {
            id: format!("sess-{}", user_id),
            user_id: user_id.to_string(),
            expires_at: SystemTime::now() + self.ttl,
        }
    }
}
