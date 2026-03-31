use chrono::Utc;

use agileplus_domain::domain::audit::{AuditEntry, hash_entry};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;

pub(super) fn build_import_audit_entry(feature_id: i64, state: &FeatureState) -> AuditEntry {
    let mut entry = AuditEntry {
        id: 0,
        feature_id,
        wp_id: None,
        timestamp: Utc::now(),
        actor: "import".into(),
        transition: format!("Imported spec -> {state}"),
        evidence_refs: vec![],
        prev_hash: [0u8; 32],
        hash: [0u8; 32],
        event_id: None,
        archived_to: None,
    };
    entry.hash = hash_entry(&entry);
    entry
}

pub(super) fn feature_meta_json(feature: &Feature, state: FeatureState) -> String {
    #[derive(serde::Serialize)]
    struct Meta<'a> {
        slug: &'a str,
        friendly_name: &'a str,
        state: String,
        spec_hash: String,
        target_branch: &'a str,
        created_at: String,
        updated_at: String,
    }

    let meta = Meta {
        slug: &feature.slug,
        friendly_name: &feature.friendly_name,
        state: state.to_string(),
        spec_hash: feature
            .spec_hash
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>(),
        target_branch: &feature.target_branch,
        created_at: feature.created_at.to_rfc3339(),
        updated_at: feature.updated_at.to_rfc3339(),
    };
    serde_json::to_string_pretty(&meta).unwrap_or_else(|_| "{}".to_string())
}

pub(super) fn sha256_bytes(content: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}
