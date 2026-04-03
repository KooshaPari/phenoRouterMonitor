//! Tests for ledger module

use std::collections::HashMap;
use tempfile::TempDir;

use crate::{IncidentLedger, LedgerVerifier};

#[test]
fn test_ledger_verifier_new() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("ledger.jsonl");
    let verifier = LedgerVerifier::new(&path);
    let report = verifier.verify_integrity();
    assert!(report.valid);
    assert_eq!(report.count, 0);
}

#[test]
fn test_ledger_verifier_empty_path() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("nonexistent.jsonl");
    let verifier = LedgerVerifier::new(&path);
    let report = verifier.verify_integrity();
    // Empty/missing file = valid
    assert!(report.valid);
}

#[test]
fn test_incident_ledger_new() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("incident.jsonl");
    let ledger = IncidentLedger::new(&path);
    assert!(ledger.verify_integrity());
}

#[test]
fn test_incident_ledger_record() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("incident.jsonl");
    let mut ledger = IncidentLedger::new(&path);

    let mut payload = HashMap::new();
    payload.insert("key".to_string(), serde_json::json!("value"));

    let hash1 = ledger.record_artifact("run_001", "test_action", &payload);
    assert!(!hash1.is_empty());

    // Record another
    let hash2 = ledger.record_artifact("run_001", "test_action2", &payload);
    assert!(!hash2.is_empty());
    assert_ne!(hash1, hash2); // Different content = different hash
}

#[test]
fn test_incident_ledger_get_artifacts() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("incident.jsonl");
    let mut ledger = IncidentLedger::new(&path);

    let mut payload = HashMap::new();
    payload.insert("data".to_string(), serde_json::json!(42));

    ledger.record_artifact("run_001", "action1", &payload);
    ledger.record_artifact("run_002", "action2", &payload);
    ledger.record_artifact("run_001", "action3", &payload);

    let artifacts = ledger.get_run_artifacts("run_001");
    assert_eq!(artifacts.len(), 2);

    let artifacts2 = ledger.get_run_artifacts("run_002");
    assert_eq!(artifacts2.len(), 1);
}

#[test]
fn test_incident_ledger_verify_integrity() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("incident.jsonl");
    let mut ledger = IncidentLedger::new(&path);

    let mut payload = HashMap::new();
    payload.insert("test".to_string(), serde_json::json!(true));

    ledger.record_artifact("run_001", "action", &payload);

    assert!(ledger.verify_integrity());
}
