//! Server-streaming helpers for agent event delivery.
//!
//! Traceability: WP14-T083

use std::pin::Pin;

use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::Stream;
use tokio_stream::StreamExt as TokioStreamExt;
use tonic::Status;

use agileplus_proto::agileplus::v1::AgentEvent as ProtoAgentEvent;

use crate::event_bus::AgentEvent;

/// Convert a domain AgentEvent to its Protobuf counterpart.
pub fn domain_event_to_proto(e: AgentEvent) -> ProtoAgentEvent {
    ProtoAgentEvent {
        event_type: e.event_type().to_string(),
        feature_slug: e.feature_slug().to_string(),
        wp_sequence: e.wp_sequence(),
        agent_id: e.agent_id().to_string(),
        payload: e.payload(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

/// Build a tonic-compatible output stream from a broadcast receiver.
///
/// Events that do not match `feature_slug_filter` are silently dropped.
/// When the sender side is dropped the stream terminates cleanly.
pub fn agent_event_stream(
    rx: broadcast::Receiver<AgentEvent>,
    feature_slug_filter: String,
) -> Pin<Box<dyn Stream<Item = Result<ProtoAgentEvent, Status>> + Send + 'static>> {
    let base = TokioStreamExt::filter_map(BroadcastStream::new(rx), move |result| {
        let filter = feature_slug_filter.clone();
        match result {
            Ok(event) if event.matches_feature(&filter) => {
                Some(Ok(domain_event_to_proto(event)))
            }
            Ok(_) => None, // Filtered out
            Err(_) => None, // Lagged — drop silently
        }
    });

    Box::pin(base)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_bus::EventBus;
    use futures::StreamExt as _;

    #[tokio::test]
    async fn stream_delivers_matching_events() {
        let bus = EventBus::new(16);
        let rx = bus.subscribe();
        let mut stream = agent_event_stream(rx, "feat-a".to_string());

        bus.publish(AgentEvent::AgentStarted {
            feature_slug: "feat-b".into(),
            wp_sequence: 1,
            agent_id: "ag".into(),
        });
        bus.publish(AgentEvent::AgentStarted {
            feature_slug: "feat-a".into(),
            wp_sequence: 2,
            agent_id: "ag".into(),
        });
        // Drop sender side so the stream terminates
        drop(bus);

        let mut received = Vec::new();
        while let Some(item) = tokio_stream::StreamExt::next(&mut stream).await {
            received.push(item.unwrap());
        }
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].feature_slug, "feat-a");
    }
}
