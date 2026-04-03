//! Event types

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub event_type: String,
    pub properties: std::collections::HashMap<String, String>,
    pub user_id: Option<String>,
    pub timestamp: String,
}

impl AnalyticsEvent {
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            properties: std::collections::HashMap::new(),
            user_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
    
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }
    
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
}
