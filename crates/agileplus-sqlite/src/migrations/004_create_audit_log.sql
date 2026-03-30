-- UP
CREATE TABLE IF NOT EXISTS audit_log (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    feature_id    INTEGER NOT NULL REFERENCES features(id) ON DELETE CASCADE,
    wp_id         INTEGER REFERENCES work_packages(id) ON DELETE SET NULL,
    timestamp     TEXT    NOT NULL,
    actor         TEXT    NOT NULL,
    transition    TEXT    NOT NULL,
    evidence_refs TEXT    NOT NULL DEFAULT '[]',
    prev_hash     BLOB    NOT NULL,
    hash          BLOB    NOT NULL
);

-- DOWN
DROP TABLE IF EXISTS audit_log;
