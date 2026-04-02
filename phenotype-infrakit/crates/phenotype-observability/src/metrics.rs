/// Minimal metrics facade used by quick-start examples.
pub fn increment_counter(name: &str) {
    // Placeholder: replace with metrics crate binding (prometheus/otel) in full implementation
    tracing::info!(metric = %name, "counter increment (stub)");
}
