-- UP
CREATE TABLE IF NOT EXISTS governance_contracts (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    feature_id INTEGER NOT NULL REFERENCES features(id) ON DELETE CASCADE,
    version    INTEGER NOT NULL DEFAULT 1,
    rules      TEXT    NOT NULL DEFAULT '[]',
    bound_at   TEXT    NOT NULL,
    UNIQUE(feature_id, version)
);

-- DOWN
DROP TABLE IF EXISTS governance_contracts;
