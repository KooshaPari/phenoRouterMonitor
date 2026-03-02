//! OpenTelemetry trace spans for AgilePlus.
//!
//! Uses the `tracing` crate (not raw OTel spans) so that the
//! `tracing-opentelemetry` bridge exports spans automatically when a provider
//! is configured.

use std::time::Instant;

// ---------------------------------------------------------------------------
// Span attribute name constants
// ---------------------------------------------------------------------------

pub const ATTR_COMMAND: &str = "agileplus.command";
pub const ATTR_FEATURE_SLUG: &str = "agileplus.feature.slug";
pub const ATTR_WP_ID: &str = "agileplus.wp.id";
pub const ATTR_AGENT_TYPE: &str = "agileplus.agent.type";
pub const ATTR_REVIEW_CYCLE: &str = "agileplus.review.cycle";

// ---------------------------------------------------------------------------
// Span creation helpers
// ---------------------------------------------------------------------------

/// Create a top-level command span.
///
/// This span should be the parent of all child spans within a single CLI
/// invocation.
pub fn create_command_span(command_name: &str, feature_slug: Option<&str>) -> tracing::Span {
    match feature_slug {
        Some(slug) => tracing::info_span!(
            "agileplus.command",
            { ATTR_COMMAND } = command_name,
            { ATTR_FEATURE_SLUG } = slug,
        ),
        None => tracing::info_span!(
            "agileplus.command",
            { ATTR_COMMAND } = command_name,
        ),
    }
}

/// Create an agent-dispatch span as a child of `parent`.
pub fn create_agent_span(
    parent: &tracing::Span,
    wp_id: &str,
    agent_type: &str,
) -> tracing::Span {
    let _guard = parent.enter();
    tracing::info_span!(
        "agileplus.agent",
        { ATTR_WP_ID } = wp_id,
        { ATTR_AGENT_TYPE } = agent_type,
    )
}

/// Create a review-loop iteration span as a child of `parent`.
pub fn create_review_span(parent: &tracing::Span, cycle: u32) -> tracing::Span {
    let _guard = parent.enter();
    tracing::info_span!(
        "agileplus.review",
        { ATTR_REVIEW_CYCLE } = cycle,
    )
}

/// Add a named event with key-value attributes to an existing span.
///
/// Used for milestone markers: "PR created", "review received", "CI passed".
pub fn record_span_event(span: &tracing::Span, name: &str, attributes: &[(String, String)]) {
    let _guard = span.enter();
    // Build a KV string for the event fields.
    let fields: Vec<String> = attributes
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect();
    tracing::info!(event = name, fields = ?fields);
}

// ---------------------------------------------------------------------------
// SpanGuard — auto-records duration on drop
// ---------------------------------------------------------------------------

/// RAII guard that records `duration_ms` on the wrapped span when dropped.
pub struct SpanGuard {
    pub span: tracing::Span,
    start: Instant,
}

impl SpanGuard {
    /// Wrap an existing span.
    pub fn new(span: tracing::Span) -> Self {
        Self {
            span,
            start: Instant::now(),
        }
    }

    /// Create a new command span and immediately wrap it in a guard.
    pub fn command(command_name: &str, feature_slug: Option<&str>) -> Self {
        Self::new(create_command_span(command_name, feature_slug))
    }
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        let duration_ms = self.start.elapsed().as_millis() as u64;
        self.span.record("duration_ms", duration_ms);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::prelude::*;

    fn init_test_subscriber() {
        // Ignore errors if subscriber already set (other tests).
        let _ = tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .try_init();
    }

    #[test]
    fn command_span_created() {
        init_test_subscriber();
        let span = create_command_span("implement", Some("001-sde"));
        // Span may be disabled or noop depending on test subscriber state;
        // verify it can be entered and dropped without panic.
        let _guard = span.enter();
    }

    #[test]
    fn child_spans_created() {
        init_test_subscriber();
        let parent = create_command_span("implement", None);
        let agent = create_agent_span(&parent, "WP10", "claude-code");
        let review = create_review_span(&agent, 1);
        // Enter each span to verify they're usable.
        let _g1 = parent.enter();
        let _g2 = agent.enter();
        let _g3 = review.enter();
    }

    #[test]
    fn span_guard_records_duration() {
        init_test_subscriber();
        let guard = SpanGuard::command("test", None);
        // Just verify it doesn't panic on drop.
        drop(guard);
    }

    #[test]
    fn record_span_event_does_not_panic() {
        init_test_subscriber();
        let span = create_command_span("test", None);
        record_span_event(
            &span,
            "pr_created",
            &[("wp_id".to_string(), "WP10".to_string())],
        );
    }
}
