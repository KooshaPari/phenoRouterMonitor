-- UP
CREATE TABLE IF NOT EXISTS backlog_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    intent TEXT NOT NULL,
    priority TEXT NOT NULL,
    status TEXT NOT NULL,
    source TEXT NOT NULL,
    feature_slug TEXT,
    tags_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_backlog_items_status_priority_created
    ON backlog_items(status, priority, created_at);
CREATE INDEX IF NOT EXISTS idx_backlog_items_intent
    ON backlog_items(intent);
CREATE INDEX IF NOT EXISTS idx_backlog_items_feature_slug
    ON backlog_items(feature_slug);

-- DOWN
DROP TABLE IF EXISTS backlog_items;
