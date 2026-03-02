//! Typed message envelope wrapping domain payloads.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::subject::Subject;

/// A message envelope carrying a serialised domain payload along with
/// routing and tracing metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    /// Unique message identifier.
    pub id: String,
    /// The subject this message was published to.
    pub subject: String,
    /// Serialised JSON payload.
    pub payload: serde_json::Value,
    /// ISO-8601 timestamp of publication.
    pub timestamp: DateTime<Utc>,
    /// Optional reply-to subject for request/reply patterns.
    pub reply_to: Option<String>,
    /// Optional correlation ID linking related messages.
    pub correlation_id: Option<String>,
}

impl Envelope {
    /// Create a new envelope for a publish operation.
    pub fn new(subject: &Subject, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            subject: subject.to_string(),
            payload,
            timestamp: Utc::now(),
            reply_to: None,
            correlation_id: None,
        }
    }

    /// Attach a reply-to subject (for request/reply).
    pub fn with_reply_to(mut self, reply_to: &Subject) -> Self {
        self.reply_to = Some(reply_to.to_string());
        self
    }

    /// Attach a correlation ID.
    pub fn with_correlation(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Serialise the envelope to bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialise an envelope from bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let env = Envelope::new(
            &Subject::new("agileplus.feature.1.created"),
            serde_json::json!({"title": "Login page"}),
        );
        let bytes = env.to_bytes().unwrap();
        let env2 = Envelope::from_bytes(&bytes).unwrap();
        assert_eq!(env2.subject, "agileplus.feature.1.created");
        assert_eq!(env2.payload["title"], "Login page");
    }

    #[test]
    fn with_reply_to() {
        let env = Envelope::new(
            &Subject::new("agileplus.rpc.triage"),
            serde_json::json!({}),
        )
        .with_reply_to(&Subject::new("_INBOX.abc123"));
        assert_eq!(env.reply_to.as_deref(), Some("_INBOX.abc123"));
    }
}
