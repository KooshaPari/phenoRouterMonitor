//! Distributed tracing utilities

use tracing::{info, Span};

/// Handle to a tracer instance
#[derive(Debug, Clone)]
pub struct TracerHandle {
    service_name: String,
}

impl TracerHandle {
    /// Create a new tracer handle
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }

    /// Get the service name
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// Create a new span with the given name
    pub fn span(&self, name: impl AsRef<str>) -> Span {
        let span_name = name.as_ref();
        tracing::info_span!("{}", span_name, service = %self.service_name)
    }
}

/// Initialize the global tracer with a service name
pub fn init_tracer(service_name: impl AsRef<str>) {
    let name = service_name.as_ref();
    info!(target: "observability", "Initializing tracer for service: {}", name);
    // In a real implementation, this would configure OTLP exporters
    println!("[TRACER] Initialized for {}", name);
}
