use super::jsonl::resolve_jsonl_conflict;
use super::resolver::resolve_git_conflicts;
use super::snapshot::resolve_snapshot_conflict;
use super::sync_state::merge_sync_state;
use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::snapshot::Snapshot;
use agileplus_domain::domain::sync_mapping::SyncMapping;

fn make_event_line(seq: i64) -> String {
    let mut e = Event::new("Feature", 1, "created", serde_json::json!({}), "test");
    e.sequence = seq;
    // Give each event a distinct hash so the dedup logic works.
    e.hash[0] = seq as u8;
    serde_json::to_string(&e).unwrap()
}

fn conflict_block(ours: &str, theirs: &str) -> String {
    format!("<<<<<<< HEAD\n{}\n=======\n{}\n>>>>>>> branch\n", ours, theirs)
}

#[test]
fn resolve_jsonl_deduplicates() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("events/Feature");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("1.jsonl");

    let ev1 = make_event_line(1);
    let ev2 = make_event_line(2);

    // Both sides contain ev1; only ours has ev2.
    let content = conflict_block(&format!("{}\n{}", ev1, ev2), &ev1);
    std::fs::write(&path, content).unwrap();

    let changed = resolve_jsonl_conflict(&path).unwrap();
    assert!(changed);

    let result = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<_> = result.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 2, "should have 2 unique events");
}

#[test]
fn resolve_snapshot_latest_wins() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("snapshots/Feature");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("1.json");

    let old_snap = Snapshot::new("Feature", 1, serde_json::json!({"v": 1}), 1);
    let new_snap = Snapshot::new("Feature", 1, serde_json::json!({"v": 5}), 5);

    let ours = serde_json::to_string_pretty(&old_snap).unwrap();
    let theirs = serde_json::to_string_pretty(&new_snap).unwrap();
    let content = conflict_block(&ours, &theirs);
    std::fs::write(&path, content).unwrap();

    let changed = resolve_snapshot_conflict(&path).unwrap();
    assert!(changed);

    let result_text = std::fs::read_to_string(&path).unwrap();
    let result: Snapshot = serde_json::from_str(&result_text).unwrap();
    assert_eq!(result.event_sequence, 5, "newer snapshot should win");
}

#[test]
fn merge_sync_state_takes_max_per_entity() {
    let ours = serde_json::json!({
        "sync_mappings": [
            {"id": 1, "entity_type": "Feature", "entity_id": 1,
             "plane_issue_id": "p1", "content_hash": "h", "last_synced_at": "2024-01-01T00:00:00Z",
             "sync_direction": "bidirectional", "conflict_count": 0}
        ],
        "sync_vector": {
            "device_id": "d1",
            "entries": {"Feature/1": 5}
        }
    });

    let theirs = serde_json::json!({
        "sync_mappings": [
            {"id": 1, "entity_type": "Feature", "entity_id": 1,
             "plane_issue_id": "p1", "content_hash": "h", "last_synced_at": "2024-01-01T00:00:00Z",
             "sync_direction": "bidirectional", "conflict_count": 2}
        ],
        "sync_vector": {
            "device_id": "d1",
            "entries": {"Feature/1": 10, "Feature/2": 3}
        }
    });

    let merged = merge_sync_state(&ours, &theirs);

    let mappings: Vec<SyncMapping> = serde_json::from_value(merged["sync_mappings"].clone()).unwrap();
    assert_eq!(mappings.len(), 1);
    assert_eq!(mappings[0].conflict_count, 2);

    let entries = &merged["sync_vector"]["entries"];
    assert_eq!(entries["Feature/1"].as_u64(), Some(10));
    assert_eq!(entries["Feature/2"].as_u64(), Some(3));
}

#[test]
fn resolve_git_conflicts_no_sync_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let r = resolve_git_conflicts(tmp.path()).unwrap();
    assert_eq!(r.jsonl_files_resolved, 0);
    assert_eq!(r.snapshot_files_resolved, 0);
    assert!(!r.sync_state_merged);
}

#[test]
fn resolve_git_conflicts_end_to_end() {
    let tmp = tempfile::tempdir().unwrap();
    let sync_dir = tmp.path().join(".agileplus/sync");

    let events_dir = sync_dir.join("events/Feature");
    std::fs::create_dir_all(&events_dir).unwrap();
    let ev1 = make_event_line(1);
    let ev2 = make_event_line(2);
    std::fs::write(
        events_dir.join("1.jsonl"),
        conflict_block(&format!("{}\n{}", ev1, ev2), &ev1),
    )
    .unwrap();

    let snap_dir = sync_dir.join("snapshots/Feature");
    std::fs::create_dir_all(&snap_dir).unwrap();
    let s1 = Snapshot::new("Feature", 1, serde_json::json!({}), 1);
    let s5 = Snapshot::new("Feature", 1, serde_json::json!({}), 5);
    std::fs::write(
        snap_dir.join("1.json"),
        conflict_block(
            &serde_json::to_string_pretty(&s1).unwrap(),
            &serde_json::to_string_pretty(&s5).unwrap(),
        ),
    )
    .unwrap();

    let ss1 = serde_json::json!({
        "sync_mappings": [],
        "sync_vector": {"device_id": "d1", "entries": {"Feature/1": 3}}
    });
    let ss2 = serde_json::json!({
        "sync_mappings": [],
        "sync_vector": {"device_id": "d1", "entries": {"Feature/1": 7}}
    });
    std::fs::write(
        sync_dir.join("sync_state.json"),
        conflict_block(
            &serde_json::to_string_pretty(&ss1).unwrap(),
            &serde_json::to_string_pretty(&ss2).unwrap(),
        ),
    )
    .unwrap();

    let r = resolve_git_conflicts(tmp.path()).unwrap();
    assert_eq!(r.jsonl_files_resolved, 1);
    assert_eq!(r.snapshot_files_resolved, 1);
    assert!(r.sync_state_merged);
}
