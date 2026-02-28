-- UP
CREATE TABLE IF NOT EXISTS features (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    slug         TEXT    UNIQUE NOT NULL,
    friendly_name TEXT   NOT NULL,
    state        TEXT    NOT NULL CHECK(state IN (
                     'created','specified','researched','planned',
                     'implementing','validated','shipped','retrospected')),
    spec_hash    BLOB    NOT NULL,
    target_branch TEXT   NOT NULL DEFAULT 'main',
    created_at   TEXT    NOT NULL,
    updated_at   TEXT    NOT NULL
);

-- DOWN
DROP TABLE IF EXISTS features;
