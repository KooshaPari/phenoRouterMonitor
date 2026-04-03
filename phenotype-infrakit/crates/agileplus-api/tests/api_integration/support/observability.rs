use agileplus_domain::ports::observability::{LogEntry, ObservabilityPort, SpanContext};

#[derive(Clone)]
pub(crate) struct MockObs;

impl ObservabilityPort for MockObs {
    fn start_span(&self, _n: &str, _p: Option<&SpanContext>) -> SpanContext {
        SpanContext {
            trace_id: "t".to_string(),
            span_id: "s".to_string(),
            parent_span_id: None,
        }
    }

    fn end_span(&self, _c: &SpanContext) {}

    fn add_span_event(&self, _c: &SpanContext, _n: &str, _a: &[(&str, &str)]) {}

    fn set_span_error(&self, _c: &SpanContext, _e: &str) {}

    fn record_counter(&self, _n: &str, _v: u64, _l: &[(&str, &str)]) {}

    fn record_histogram(&self, _n: &str, _v: f64, _l: &[(&str, &str)]) {}

    fn record_gauge(&self, _n: &str, _v: f64, _l: &[(&str, &str)]) {}

    fn log(&self, _e: &LogEntry) {}

    fn log_info(&self, _m: &str) {}

    fn log_warn(&self, _m: &str) {}

    fn log_error(&self, _m: &str) {}
}
