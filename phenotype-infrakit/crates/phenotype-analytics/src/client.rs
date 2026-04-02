//! Analytics client

use crate::error::{AnalyticsError, Result};
use crate::event::AnalyticsEvent;

pub struct AnalyticsClient {
    backend: Box<dyn AnalyticsBackend>,
}

impl AnalyticsClient {
    pub fn new(backend: Box<dyn AnalyticsBackend>) -> Self {
        Self { backend }
    }
    
    pub async fn track(&self, event: AnalyticsEvent) -> Result<()> {
        self.backend.track(event).await
    }
}

pub trait AnalyticsBackend: Send + Sync {
    async fn track(&self, event: AnalyticsEvent) -> Result<()>;
}
