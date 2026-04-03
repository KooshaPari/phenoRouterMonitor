-- UP

CREATE TABLE modules (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    slug             TEXT    NOT NULL,
    friendly_name    TEXT    NOT NULL,
    description      TEXT,
    parent_module_id INTEGER REFERENCES modules(id) ON DELETE RESTRICT,
    created_at       TEXT    NOT NULL,
    updated_at       TEXT    NOT NULL,
    UNIQUE (parent_module_id, slug)
);

CREATE INDEX idx_modules_parent ON modules(parent_module_id);

CREATE TABLE module_feature_tags (
    module_id   INTEGER NOT NULL REFERENCES modules(id)  ON DELETE CASCADE,
    feature_id  INTEGER NOT NULL REFERENCES features(id) ON DELETE CASCADE,
    created_at  TEXT    NOT NULL,
    PRIMARY KEY (module_id, feature_id)
);

CREATE INDEX idx_module_feature_tags_feature ON module_feature_tags(feature_id);

ALTER TABLE features ADD COLUMN module_id INTEGER REFERENCES modules(id) ON DELETE SET NULL;

CREATE INDEX idx_features_module ON features(module_id);

CREATE TABLE cycles (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    name             TEXT    NOT NULL UNIQUE,
    description      TEXT,
    state            TEXT    NOT NULL DEFAULT 'Draft',
    start_date       TEXT    NOT NULL,
    end_date         TEXT    NOT NULL,
    module_scope_id  INTEGER REFERENCES modules(id) ON DELETE SET NULL,
    created_at       TEXT    NOT NULL,
    updated_at       TEXT    NOT NULL,
    CHECK (end_date > start_date)
);

CREATE INDEX idx_cycles_state        ON cycles(state);
CREATE INDEX idx_cycles_module_scope ON cycles(module_scope_id);

CREATE TABLE cycle_features (
    cycle_id    INTEGER NOT NULL REFERENCES cycles(id)   ON DELETE CASCADE,
    feature_id  INTEGER NOT NULL REFERENCES features(id) ON DELETE CASCADE,
    added_at    TEXT    NOT NULL,
    PRIMARY KEY (cycle_id, feature_id)
);

CREATE INDEX idx_cycle_features_feature ON cycle_features(feature_id);

-- DOWN

DROP INDEX IF EXISTS idx_cycle_features_feature;
DROP TABLE IF EXISTS cycle_features;
DROP INDEX IF EXISTS idx_cycles_module_scope;
DROP INDEX IF EXISTS idx_cycles_state;
DROP TABLE IF EXISTS cycles;
DROP INDEX IF EXISTS idx_features_module;
DROP INDEX IF EXISTS idx_module_feature_tags_feature;
DROP TABLE IF EXISTS module_feature_tags;
DROP INDEX IF EXISTS idx_modules_parent;
DROP TABLE IF EXISTS modules;
