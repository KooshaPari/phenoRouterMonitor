//! Configuration for the NATS event bus connection.

/// Configuration for connecting to a NATS server.
#[derive(Clone, Debug)]
pub struct NatsConfig {
    /// NATS server URL (e.g. `nats://localhost:4222`).
    pub url: String,
    /// Optional authentication token.
    pub auth_token: Option<String>,
    /// Subject prefix for all AgilePlus messages.
    pub subject_prefix: String,
    /// Maximum payload size in bytes (NATS default is 1 MiB).
    pub max_payload: usize,
}

impl NatsConfig {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            auth_token: None,
            subject_prefix: "agileplus".to_string(),
            max_payload: 1_048_576,
        }
    }

    pub fn with_auth(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.subject_prefix = prefix.into();
        self
    }
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self::new("nats://localhost:4222")
    }
}
