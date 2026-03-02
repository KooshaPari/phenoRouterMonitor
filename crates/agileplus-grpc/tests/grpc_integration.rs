//! Integration tests for the gRPC server layer.
//!
//! These tests verify the server handlers, error mapping, event bus,
//! and proxy router in isolation — without a running network socket.
//!
//! Traceability: WP14-T079, T080, T080b, T083

use agileplus_grpc::{
    conversions::{feature_to_proto, wp_to_proto},
    event_bus::{AgentEvent, EventBus},
    proxy::ProxyRouter,
    server::domain_error_to_status,
};
use agileplus_domain::{
    domain::{feature::Feature, work_package::WorkPackage},
    error::DomainError,
};

// --- Conversion tests ---

#[test]
fn feature_to_proto_roundtrip() {
    let f = Feature::new("conv-test", "Conversion Test", [0xab; 32], Some("develop"));
    let p = feature_to_proto(f);
    assert_eq!(p.slug, "conv-test");
    assert_eq!(p.friendly_name, "Conversion Test");
    assert_eq!(p.state, "Created");
    assert_eq!(p.target_branch, "develop");
    assert!(!p.created_at.is_empty());
}

#[test]
fn wp_to_proto_state_lowercase() {
    let wp = WorkPackage::new(1, "My WP", 3, "done when green");
    let p = wp_to_proto(wp);
    assert_eq!(p.state, "planned");
    assert_eq!(p.sequence, 3);
    assert_eq!(p.title, "My WP");
}

// --- Error mapping tests ---

#[test]
fn not_found_maps_to_not_found_status() {
    use tonic::Code;
    let s = domain_error_to_status(DomainError::NotFound("feature xyz".into()));
    assert_eq!(s.code(), Code::NotFound);
    assert!(s.message().contains("feature xyz"));
}

#[test]
fn conflict_maps_to_already_exists() {
    use tonic::Code;
    let s = domain_error_to_status(DomainError::Conflict("slug already taken".into()));
    assert_eq!(s.code(), Code::AlreadyExists);
}

#[test]
fn invalid_transition_maps_to_failed_precondition() {
    use tonic::Code;
    let s = domain_error_to_status(DomainError::InvalidTransition {
        from: "planned".into(),
        to: "created".into(),
        reason: "backward".into(),
    });
    assert_eq!(s.code(), Code::FailedPrecondition);
}

#[test]
fn timeout_maps_to_deadline_exceeded() {
    use tonic::Code;
    let s = domain_error_to_status(DomainError::Timeout(30));
    assert_eq!(s.code(), Code::DeadlineExceeded);
}

// --- Event bus tests ---

#[tokio::test]
async fn event_bus_publish_subscribe() {
    let bus = EventBus::new(32);
    let mut rx = bus.subscribe();

    bus.publish(AgentEvent::AgentStarted {
        feature_slug: "ev-test".into(),
        wp_sequence: 1,
        agent_id: "agt-1".into(),
    });
    bus.publish(AgentEvent::AgentCompleted {
        feature_slug: "ev-test".into(),
        wp_sequence: 1,
        success: true,
    });

    let e1 = rx.recv().await.unwrap();
    assert_eq!(e1.feature_slug(), "ev-test");
    assert_eq!(e1.event_type(), "agent_started");

    let e2 = rx.recv().await.unwrap();
    assert_eq!(e2.event_type(), "agent_completed");
}

#[tokio::test]
async fn event_bus_filter_by_slug() {
    let bus = EventBus::new(32);
    let mut rx = bus.subscribe();

    bus.publish(AgentEvent::WpStateChanged {
        feature_slug: "feat-a".into(),
        wp_sequence: 1,
        old_state: "planned".into(),
        new_state: "doing".into(),
    });
    bus.publish(AgentEvent::WpStateChanged {
        feature_slug: "feat-b".into(),
        wp_sequence: 2,
        old_state: "planned".into(),
        new_state: "doing".into(),
    });

    let mut a_count = 0;
    let mut b_count = 0;
    for _ in 0..2 {
        let e = rx.recv().await.unwrap();
        if e.matches_feature("feat-a") { a_count += 1; }
        if e.matches_feature("feat-b") { b_count += 1; }
        if e.matches_feature("") { /* matches all */ }
    }
    assert_eq!(a_count, 1);
    assert_eq!(b_count, 1);
}

// --- Proxy router tests ---

#[tokio::test]
async fn proxy_router_stub_mode() {
    let router = ProxyRouter::new(None, None).await;
    assert!(!router.health().agents_reachable);
    assert!(!router.health().integrations_reachable);

    let result = router
        .dispatch_agent_command("implement", "test-feat", &Default::default())
        .await;
    assert!(result.is_success());
    assert!(result.message().contains("stub"));
}

#[tokio::test]
async fn proxy_router_unreachable_address_falls_back_to_stub() {
    let router = ProxyRouter::new(
        Some("localhost:59999".into()), // Nothing listening here
        None,
    )
    .await;
    assert!(!router.health().agents_reachable);

    let result = router
        .dispatch_agent_command("implement", "feat-x", &Default::default())
        .await;
    assert!(result.is_success()); // Stubs always succeed
}
