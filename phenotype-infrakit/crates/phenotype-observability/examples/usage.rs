fn main() {
    // Initialize tracing for the service
    phenotype_observability::init_tracer("phenotype-example-service");

    // Record a metric (stubbed implementation)
    phenotype_observability::increment_counter("requests_total");

    tracing::info!(message = "Example completed");
}
