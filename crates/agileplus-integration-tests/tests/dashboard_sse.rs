//! T109: Dashboard SSE (Server-Sent Events) integration test.
//!
//! Verifies that feature mutations broadcast SSE events to connected clients.
//!
//! Traceability: WP19-T109

use std::time::Duration;

use agileplus_integration_tests::common::{
    fixtures::feature_create_payload,
    harness::{TestHarness, is_process_compose_installed},
};

/// Helper: skip the test if services are unavailable.
macro_rules! require_services {
    () => {
        if !is_process_compose_installed() {
            eprintln!(
                "SKIP: process-compose not installed — \
                 run with --features integration and a live stack to execute this test."
            );
            return Ok(());
        }
    };
}

/// SSE connection lifecycle: connect → trigger event → receive message.
///
/// Requires a running service stack.
#[tokio::test]
#[ignore]
#[cfg(feature = "integration")]
async fn dashboard_sse_integration() -> anyhow::Result<()> {
    require_services!();

    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .try_init();

    let harness = TestHarness::start().await?;
    let base = harness.base_url().to_string();

    // -----------------------------------------------------------------------
    // Step 1: Open SSE connection
    // -----------------------------------------------------------------------
    let sse_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let sse_resp = sse_client
        .get(format!("{base}/api/stream"))
        .header("Accept", "text/event-stream")
        .send()
        .await?;

    assert_eq!(
        sse_resp.status().as_u16(),
        200,
        "SSE endpoint should return 200"
    );
    assert_eq!(
        sse_resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(""),
        "text/event-stream",
        "SSE content-type header should be text/event-stream"
    );

    // -----------------------------------------------------------------------
    // Step 2: Spawn a task to trigger a feature creation event
    // -----------------------------------------------------------------------
    let api_url = harness.url("/api/features");
    let api_client = harness.client().clone();
    let payload = feature_create_payload("SSE test feature", "Verify SSE delivery");

    let create_task = tokio::spawn(async move {
        // Small delay to ensure the SSE consumer is ready.
        tokio::time::sleep(Duration::from_millis(200)).await;
        api_client.post(&api_url).json(&payload).send().await
    });

    // -----------------------------------------------------------------------
    // Step 3: Read bytes from the SSE stream until we see the expected event
    // -----------------------------------------------------------------------
    use futures_util::StreamExt as _;
    let mut byte_stream = sse_resp.bytes_stream();
    let mut buffer = String::new();
    let deadline = tokio::time::sleep(Duration::from_secs(10));
    tokio::pin!(deadline);

    let mut received_event: Option<serde_json::Value> = None;

    loop {
        tokio::select! {
            chunk = byte_stream.next() => {
                match chunk {
                    Some(Ok(bytes)) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));
                        // SSE events are delimited by double newlines.
                        while let Some(idx) = buffer.find("\n\n") {
                            let raw_event = buffer[..idx + 2].to_string();
                            buffer = buffer[idx + 2..].to_string();

                            for line in raw_event.lines() {
                                if let Some(data) = line.strip_prefix("data: ") {
                                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                                        if v["type"] == "FeatureCreated" {
                                            received_event = Some(v);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(e)) => {
                        panic!("SSE stream error: {e}");
                    }
                    None => break,
                }
            }
            _ = &mut deadline => {
                break;
            }
        }
        if received_event.is_some() {
            break;
        }
    }

    // Wait for the creation task to finish.
    let create_result = create_task.await??;
    assert_eq!(
        create_result.status().as_u16(),
        201,
        "feature creation during SSE test should succeed"
    );

    // -----------------------------------------------------------------------
    // Step 4: Assert expected SSE event received
    // -----------------------------------------------------------------------
    let event = received_event.expect("should have received a FeatureCreated SSE event");
    assert_eq!(event["type"], "FeatureCreated");
    assert_eq!(event["data"]["friendly_name"], "SSE test feature");

    Ok(())
}

// ---------------------------------------------------------------------------
// Unit-safe stub tests (always run)
// ---------------------------------------------------------------------------

#[test]
fn dashboard_sse_module_compiles() {
    let _ = feature_create_payload("sse-test", "payload check");
}
