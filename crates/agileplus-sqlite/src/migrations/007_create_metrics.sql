-- UP
CREATE TABLE IF NOT EXISTS metrics (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    feature_id     INTEGER REFERENCES features(id) ON DELETE SET NULL,
    command        TEXT    NOT NULL,
    duration_ms    INTEGER NOT NULL DEFAULT 0,
    agent_runs     INTEGER NOT NULL DEFAULT 0,
    review_cycles  INTEGER NOT NULL DEFAULT 0,
    metadata       TEXT,
    timestamp      TEXT    NOT NULL
);

-- DOWN
DROP TABLE IF EXISTS metrics;
