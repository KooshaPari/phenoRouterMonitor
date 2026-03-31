-- UP
CREATE TABLE IF NOT EXISTS work_packages (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    feature_id          INTEGER NOT NULL REFERENCES features(id) ON DELETE CASCADE,
    title               TEXT    NOT NULL,
    state               TEXT    NOT NULL CHECK(state IN (
                            'planned','doing','review','done','blocked')),
    sequence            INTEGER NOT NULL DEFAULT 0,
    file_scope          TEXT    NOT NULL DEFAULT '[]',
    acceptance_criteria TEXT    NOT NULL DEFAULT '',
    agent_id            TEXT,
    pr_url              TEXT,
    pr_state            TEXT    CHECK(pr_state IN (
                            'open','review','changes_requested','approved','merged') OR pr_state IS NULL),
    worktree_path       TEXT,
    created_at          TEXT    NOT NULL,
    updated_at          TEXT    NOT NULL
);

-- DOWN
DROP TABLE IF EXISTS work_packages;
