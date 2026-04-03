-- Git provenance bindings for entities
CREATE TABLE IF NOT EXISTS git_bindings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    commit_sha TEXT NOT NULL,
    branch TEXT,
    event_type TEXT NOT NULL,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(entity_type, entity_id, commit_sha, event_type)
);

CREATE INDEX IF NOT EXISTS idx_git_bindings_entity ON git_bindings(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_git_bindings_commit ON git_bindings(commit_sha);

-- Worktree claims for multi-agent coordination
CREATE TABLE IF NOT EXISTS worktree_claims (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    wp_id TEXT NOT NULL,
    branch TEXT NOT NULL,
    heartbeat TEXT NOT NULL DEFAULT (datetime('now')),
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'stale', 'released')),
    file_scope TEXT, -- JSON array of file paths
    UNIQUE(path, status) -- Only one active claim per path
);

CREATE INDEX IF NOT EXISTS idx_worktree_claims_agent ON worktree_claims(agent_id);
CREATE INDEX IF NOT EXISTS idx_worktree_claims_status ON worktree_claims(status);

-- Add git provenance columns to features
ALTER TABLE features ADD COLUMN created_at_commit TEXT;
ALTER TABLE features ADD COLUMN last_modified_commit TEXT;

-- Add git provenance columns to work_packages
ALTER TABLE work_packages ADD COLUMN base_commit TEXT;
ALTER TABLE work_packages ADD COLUMN head_commit TEXT;
