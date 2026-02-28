use agileplus_domain::domain::audit::{AuditChain, AuditEntry, hash_entry};
use chrono::Utc;
use proptest::prelude::*;

proptest! {
    #[test]
    fn hash_deterministic(id in 0i64..1000, actor in "[a-z]{1,10}", transition in "[a-z_ >]{1,20}") {
        let ts = Utc::now();
        let h1 = hash_entry(id, &ts, &actor, &transition, &[], &[0u8; 32]);
        let h2 = hash_entry(id, &ts, &actor, &transition, &[], &[0u8; 32]);
        prop_assert_eq!(h1, h2);
    }

    #[test]
    fn chain_integrity_after_appends(n in 1usize..20) {
        let mut chain = AuditChain::new();
        chain.append(1, None, "test", "genesis", &[]).unwrap();
        for i in 1..n {
            chain.append(1, Some(i as i64), "test", "step", &[]).unwrap();
        }
        prop_assert!(chain.verify().is_ok());
        prop_assert_eq!(chain.verify().unwrap(), chain.len());
    }

    #[test]
    fn genesis_always_verifies(actor in "[a-z]{1,10}") {
        let entry = AuditEntry::genesis(1, actor);
        prop_assert!(entry.verify_hash());
        prop_assert_eq!(entry.prev_hash, [0u8; 32]);
    }
}
