-- UP
CREATE TABLE IF NOT EXISTS snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL,
    entity_id INTEGER NOT NULL,
    state TEXT NOT NULL,
    event_sequence INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(entity_type, entity_id, event_sequence)
);

CREATE INDEX IF NOT EXISTS idx_snapshots_entity ON snapshots(entity_type, entity_id, event_sequence DESC);
CREATE INDEX IF NOT EXISTS idx_snapshots_time ON snapshots(created_at);

-- DOWN
DROP TABLE IF EXISTS snapshots;
