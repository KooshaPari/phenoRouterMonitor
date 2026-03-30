-- UP
CREATE INDEX IF NOT EXISTS idx_features_state      ON features(state);
CREATE INDEX IF NOT EXISTS idx_features_slug       ON features(slug);
CREATE INDEX IF NOT EXISTS idx_wp_feature_id       ON work_packages(feature_id);
CREATE INDEX IF NOT EXISTS idx_wp_state            ON work_packages(state);
CREATE INDEX IF NOT EXISTS idx_audit_feature_id    ON audit_log(feature_id);
CREATE INDEX IF NOT EXISTS idx_audit_timestamp     ON audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_evidence_wp_id      ON evidence(wp_id);
CREATE INDEX IF NOT EXISTS idx_evidence_fr_id      ON evidence(fr_id);
CREATE INDEX IF NOT EXISTS idx_policy_domain       ON policy_rules(domain);
CREATE INDEX IF NOT EXISTS idx_policy_active       ON policy_rules(active);
CREATE INDEX IF NOT EXISTS idx_metrics_feature_id  ON metrics(feature_id);
CREATE INDEX IF NOT EXISTS idx_metrics_timestamp   ON metrics(timestamp);
CREATE INDEX IF NOT EXISTS idx_governance_feature  ON governance_contracts(feature_id);

-- DOWN
DROP INDEX IF EXISTS idx_features_state;
DROP INDEX IF EXISTS idx_features_slug;
DROP INDEX IF EXISTS idx_wp_feature_id;
DROP INDEX IF EXISTS idx_wp_state;
DROP INDEX IF EXISTS idx_audit_feature_id;
DROP INDEX IF EXISTS idx_audit_timestamp;
DROP INDEX IF EXISTS idx_evidence_wp_id;
DROP INDEX IF EXISTS idx_evidence_fr_id;
DROP INDEX IF EXISTS idx_policy_domain;
DROP INDEX IF EXISTS idx_policy_active;
DROP INDEX IF EXISTS idx_metrics_feature_id;
DROP INDEX IF EXISTS idx_metrics_timestamp;
DROP INDEX IF EXISTS idx_governance_feature;
