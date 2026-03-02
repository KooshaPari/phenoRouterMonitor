use agileplus_domain::domain::audit::{AuditChain, AuditEntry, hash_entry};
use chrono::Utc;
use proptest::prelude::*;

fn make_entry(feature_id: i64, wp_id: Option<i64>, actor: &str, transition: &str, prev_hash: [u8; 32]) -> AuditEntry {
    let mut entry = AuditEntry {
        id: 0,
        feature_id,
        wp_id,
        timestamp: Utc::now(),
        actor: actor.to_string(),
        transition: transition.to_string(),
        evidence_refs: vec![],
        prev_hash,
        hash: [0u8; 32],
    };
    entry.hash = hash_entry(&entry);
    entry
}

proptest! {
    #[test]
    fn hash_deterministic(id in 0i64..1000, actor in "[a-z]{1,10}", transition in "[a-z_ >]{1,20}") {
        let entry = make_entry(id, None, &actor, &transition, [0u8; 32]);
        let h1 = hash_entry(&entry);
        let h2 = hash_entry(&entry);
        prop_assert_eq!(h1, h2);
    }

    #[test]
    fn chain_integrity_after_appends(n in 1usize..20) {
        let mut entries = Vec::new();
        let genesis = make_entry(1, None, "test", "genesis", [0u8; 32]);
        entries.push(genesis);
        for _ in 1..n {
            let prev_hash = entries.last().unwrap().hash;
            let entry = make_entry(1, Some(1), "test", "step", prev_hash);
            entries.push(entry);
        }
        let chain = AuditChain { entries };
        prop_assert!(chain.verify_chain().is_ok());
    }

    #[test]
    fn genesis_always_verifies(actor in "[a-z]{1,10}") {
        let entry = make_entry(1, None, &actor, "genesis", [0u8; 32]);
        let recomputed = hash_entry(&entry);
        prop_assert_eq!(entry.hash, recomputed);
        prop_assert_eq!(entry.prev_hash, [0u8; 32]);
    }
}
