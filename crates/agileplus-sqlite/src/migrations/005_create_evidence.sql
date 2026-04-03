-- UP
CREATE TABLE IF NOT EXISTS evidence (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    wp_id         INTEGER NOT NULL REFERENCES work_packages(id) ON DELETE CASCADE,
    fr_id         TEXT    NOT NULL,
    evidence_type TEXT    NOT NULL CHECK(evidence_type IN (
                      'test_result','ci_output','review_approval',
                      'security_scan','lint_result','manual_attestation')),
    artifact_path TEXT    NOT NULL,
    metadata      TEXT,
    created_at    TEXT    NOT NULL
);

-- DOWN
DROP TABLE IF EXISTS evidence;
