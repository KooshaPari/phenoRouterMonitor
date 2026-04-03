//! Analytics client

use crate::error::Result;
use crate::event::AnalyticsEvent;

/// Analytics client
pub struct AnalyticsClient<B: AnalyticsBackend> {
    backend: B,
}

impl<B: AnalyticsBackend> AnalyticsClient<B> {
    /// Create a new client
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
    
    /// Track an event
    pub async fn track(&self, event: AnalyticsEvent) -> Result<()> {
        self.backend.track(event).await
    }
}

/// Analytics backend trait
#[async_trait::async_trait]
pub trait AnalyticsBackend: Send + Sync {
    /// Track an event
    async fn track(&self, event: AnalyticsEvent) -> Result<()>;
}
