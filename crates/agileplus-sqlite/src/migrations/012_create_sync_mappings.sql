-- UP
CREATE TABLE IF NOT EXISTS sync_mappings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL,
    entity_id INTEGER NOT NULL,
    plane_issue_id TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    last_synced_at TEXT NOT NULL,
    sync_direction TEXT NOT NULL,
    conflict_count INTEGER NOT NULL DEFAULT 0,
    UNIQUE(entity_type, entity_id),
    UNIQUE(plane_issue_id)
);

CREATE INDEX IF NOT EXISTS idx_sync_entity ON sync_mappings(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_sync_plane_id ON sync_mappings(plane_issue_id);
CREATE INDEX IF NOT EXISTS idx_sync_time ON sync_mappings(last_synced_at);

-- DOWN
DROP TABLE IF EXISTS sync_mappings;
