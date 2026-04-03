//! Observability usage example

use phenotype_observability::{init_tracer, increment_counter, record_gauge, in_span};

#[tokio::main]
async fn main() {
    init_tracer("example-service");
    increment_counter("requests.total");
    record_gauge("active_connections", 42.0);
    in_span!("process_request", {
        println!("Processing request within span...");
        increment_counter("requests.processed");
    });
    println!("Example completed successfully");
}
