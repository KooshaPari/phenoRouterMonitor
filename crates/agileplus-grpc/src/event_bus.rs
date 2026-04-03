//! Agent event bus — tokio broadcast channel for real-time events.
//!
//! Traceability: WP14-T083

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

/// Events published during agent execution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AgentEvent {
    AgentStarted {
        feature_slug: String,
        wp_sequence: i32,
        agent_id: String,
    },
    PrCreated {
        feature_slug: String,
        wp_sequence: i32,
        pr_url: String,
    },
    ReviewReceived {
        feature_slug: String,
        wp_sequence: i32,
        review_status: String,
        comments: usize,
    },
    AgentFixing {
        feature_slug: String,
        wp_sequence: i32,
        cycle: u32,
    },
    AgentCompleted {
        feature_slug: String,
        wp_sequence: i32,
        success: bool,
    },
    WpStateChanged {
        feature_slug: String,
        wp_sequence: i32,
        old_state: String,
        new_state: String,
    },
}

impl AgentEvent {
    /// Returns the feature slug this event belongs to.
    pub fn feature_slug(&self) -> &str {
        match self {
            AgentEvent::AgentStarted { feature_slug, .. } => feature_slug,
            AgentEvent::PrCreated { feature_slug, .. } => feature_slug,
            AgentEvent::ReviewReceived { feature_slug, .. } => feature_slug,
            AgentEvent::AgentFixing { feature_slug, .. } => feature_slug,
            AgentEvent::AgentCompleted { feature_slug, .. } => feature_slug,
            AgentEvent::WpStateChanged { feature_slug, .. } => feature_slug,
        }
    }

    /// Returns true if this event matches a given feature slug filter.
    /// Empty filter matches all events.
    pub fn matches_feature(&self, filter: &str) -> bool {
        filter.is_empty() || self.feature_slug() == filter
    }

    /// Returns the event_type string for the proto message.
    pub fn event_type(&self) -> &'static str {
        match self {
            AgentEvent::AgentStarted { .. } => "agent_started",
            AgentEvent::PrCreated { .. } => "pr_created",
            AgentEvent::ReviewReceived { .. } => "review_received",
            AgentEvent::AgentFixing { .. } => "agent_fixing",
            AgentEvent::AgentCompleted { .. } => "agent_completed",
            AgentEvent::WpStateChanged { .. } => "wp_state_changed",
        }
    }

    pub fn wp_sequence(&self) -> i32 {
        match self {
            AgentEvent::AgentStarted { wp_sequence, .. } => *wp_sequence,
            AgentEvent::PrCreated { wp_sequence, .. } => *wp_sequence,
            AgentEvent::ReviewReceived { wp_sequence, .. } => *wp_sequence,
            AgentEvent::AgentFixing { wp_sequence, .. } => *wp_sequence,
            AgentEvent::AgentCompleted { wp_sequence, .. } => *wp_sequence,
            AgentEvent::WpStateChanged { wp_sequence, .. } => *wp_sequence,
        }
    }

    pub fn agent_id(&self) -> &str {
        match self {
            AgentEvent::AgentStarted { agent_id, .. } => agent_id,
            _ => "",
        }
    }

    pub fn payload(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// Shared event bus backed by a tokio broadcast channel.
///
/// Capacity defaults to 1024. When the channel is full the oldest events
/// are dropped to avoid blocking publishers.
#[derive(Clone, Debug)]
pub struct EventBus {
    sender: broadcast::Sender<AgentEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publish an event. If no receivers are connected the send is silently
    /// dropped.
    pub fn publish(&self, event: AgentEvent) {
        let _ = self.sender.send(event);
    }

    /// Subscribe to the event stream.
    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn publish_subscribe_round_trip() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();
        bus.publish(AgentEvent::AgentStarted {
            feature_slug: "feat-a".into(),
            wp_sequence: 1,
            agent_id: "agent-1".into(),
        });
        let event = rx.recv().await.unwrap();
        assert_eq!(event.feature_slug(), "feat-a");
        assert_eq!(event.event_type(), "agent_started");
    }

    #[test]
    fn matches_feature_filter() {
        let e = AgentEvent::AgentCompleted {
            feature_slug: "feat-x".into(),
            wp_sequence: 2,
            success: true,
        };
        assert!(e.matches_feature("feat-x"));
        assert!(e.matches_feature(""));
        assert!(!e.matches_feature("feat-y"));
    }
}
