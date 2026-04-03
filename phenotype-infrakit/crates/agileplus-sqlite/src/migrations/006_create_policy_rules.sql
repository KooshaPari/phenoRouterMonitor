-- UP
CREATE TABLE IF NOT EXISTS policy_rules (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    domain     TEXT    NOT NULL CHECK(domain IN (
                   'security','quality','compliance','performance','custom')),
    rule       TEXT    NOT NULL,
    active     INTEGER NOT NULL DEFAULT 1 CHECK(active IN (0, 1)),
    created_at TEXT    NOT NULL,
    updated_at TEXT    NOT NULL
);

-- DOWN
DROP TABLE IF EXISTS policy_rules;
