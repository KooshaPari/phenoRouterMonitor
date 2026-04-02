//! phenotype-observability
//! Simple, minimal wrappers to initialize tracing and record basic metrics/logs

pub mod tracer;
pub mod metrics;

pub use tracer::init_tracer;
pub use metrics::increment_counter;
