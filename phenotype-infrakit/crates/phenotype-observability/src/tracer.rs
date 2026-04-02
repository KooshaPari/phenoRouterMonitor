use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize a basic tracing subscriber (OTEL or fmt can be added later).
pub fn init_tracer(service_name: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_env_filter(filter)
        .init();
    tracing::info!(service = %service_name, "tracer initialized");
}
