//! Unit tests for the `fallback` module.

use agileplus_agent_review::fallback::{prompt_manual_review, should_fallback};
use chrono::Utc;
use std::time::Duration;

#[test]
fn should_fallback_when_never_checked() {
    assert!(should_fallback(None, Duration::from_secs(60)));
}

#[test]
fn should_not_fallback_when_recent() {
    let now = Utc::now();
    assert!(!should_fallback(Some(now), Duration::from_secs(300)));
}

#[test]
fn should_fallback_after_timeout() {
    let old = Utc::now() - chrono::Duration::minutes(10);
    assert!(should_fallback(Some(old), Duration::from_secs(300)));
}

#[test]
fn should_not_fallback_just_under_timeout() {
    let just_now = Utc::now() - chrono::Duration::seconds(5);
    assert!(!should_fallback(Some(just_now), Duration::from_secs(300)));
}

#[test]
fn non_interactive_returns_err() {
    // In test context stdin is not a TTY so prompt_manual_review must Err.
    let result = prompt_manual_review("https://github.com/acme/repo/pull/42", "WP09");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("non-interactive"),
        "expected non-interactive error, got: {msg}"
    );
}
