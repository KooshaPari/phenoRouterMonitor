use super::*;

#[test]
fn sha256_is_deterministic() {
    let h1 = sha256_bytes("hello");
    let h2 = sha256_bytes("hello");
    assert_eq!(h1, h2);
}

#[test]
fn sha256_differs_on_different_input() {
    let h1 = sha256_bytes("hello");
    let h2 = sha256_bytes("world");
    assert_ne!(h1, h2);
}

#[test]
fn diff_summary_detects_changes() {
    let old = "line1\nline2\n";
    let new = "line1\nline3\n";
    let diff = compute_diff_summary(old, new);
    assert!(diff.contains("-line2") || diff.contains("+line3"));
}

#[test]
fn diff_summary_no_change() {
    let content = "line1\nline2\n";
    let diff = compute_diff_summary(content, content);
    assert!(!diff.contains('-'));
    assert!(!diff.contains('+'));
}

#[test]
fn build_audit_entry_hash_correct() {
    let entry = build_audit_entry(1, "user", "Created -> Specified", [0u8; 32]);
    assert_ne!(entry.hash, [0u8; 32]);
    let expected = hash_entry(&entry);
    assert_eq!(entry.hash, expected);
}
