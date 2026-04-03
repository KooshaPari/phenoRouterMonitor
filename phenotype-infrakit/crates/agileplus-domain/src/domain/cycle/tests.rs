use chrono::{NaiveDate, Utc};

use super::*;
use crate::domain::feature::Feature;
use crate::domain::state_machine::FeatureState;
use crate::error::DomainError;

fn make_date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}

// --- CycleState transition tests ---

#[test]
fn valid_transitions() {
    assert!(CycleState::Draft.transition(CycleState::Active).is_ok());
    assert!(CycleState::Active.transition(CycleState::Review).is_ok());
    assert!(CycleState::Active.transition(CycleState::Draft).is_ok());
    assert!(CycleState::Review.transition(CycleState::Shipped).is_ok());
    assert!(CycleState::Review.transition(CycleState::Active).is_ok());
    assert!(CycleState::Shipped.transition(CycleState::Archived).is_ok());
}

#[test]
fn invalid_transitions() {
    let invalid_pairs = [
        (CycleState::Draft, CycleState::Review),
        (CycleState::Draft, CycleState::Shipped),
        (CycleState::Draft, CycleState::Archived),
        (CycleState::Active, CycleState::Shipped),
        (CycleState::Active, CycleState::Archived),
        (CycleState::Review, CycleState::Archived),
        (CycleState::Review, CycleState::Draft),
        (CycleState::Shipped, CycleState::Active),
        (CycleState::Shipped, CycleState::Draft),
        (CycleState::Shipped, CycleState::Review),
        (CycleState::Archived, CycleState::Draft),
        (CycleState::Archived, CycleState::Active),
        (CycleState::Archived, CycleState::Review),
        (CycleState::Archived, CycleState::Shipped),
    ];
    for (from, to) in invalid_pairs {
        let result = from.transition(to);
        assert!(
            result.is_err(),
            "expected Err for {from} -> {to} but got Ok"
        );
        assert!(
            matches!(result.unwrap_err(), DomainError::InvalidTransition { .. }),
            "expected InvalidTransition for {from} -> {to}"
        );
    }
}

#[test]
fn noop_transition() {
    let states = [
        CycleState::Draft,
        CycleState::Active,
        CycleState::Review,
        CycleState::Shipped,
        CycleState::Archived,
    ];
    for s in states {
        let result = s.transition(s);
        assert!(
            matches!(result, Err(DomainError::NoOpTransition(_))),
            "expected NoOpTransition for {s} -> {s}"
        );
    }
}

// --- Cycle struct tests ---

#[test]
fn new_cycle_valid_dates() {
    let c =
        Cycle::new("Q1", make_date(2026, 1, 1), make_date(2026, 3, 31), None).expect("valid dates");
    assert_eq!(c.id, 0);
    assert_eq!(c.state, CycleState::Draft);
    assert!(c.module_scope_id.is_none());
}

#[test]
fn new_cycle_invalid_dates_equal() {
    let d = make_date(2026, 1, 1);
    assert!(Cycle::new("Bad", d, d, None).is_err());
}

#[test]
fn new_cycle_invalid_dates_start_after_end() {
    let start = make_date(2026, 3, 1);
    let end = make_date(2026, 1, 1);
    assert!(Cycle::new("Bad", start, end, None).is_err());
}

#[test]
fn cycle_transition_updates_state() {
    let mut c = Cycle::new("C", make_date(2026, 1, 1), make_date(2026, 2, 1), None).expect("valid");
    c.transition(CycleState::Active).expect("Draft->Active ok");
    assert_eq!(c.state, CycleState::Active);
}

#[test]
fn cycle_new_with_scope() {
    let c = Cycle::new(
        "Scoped",
        make_date(2026, 1, 1),
        make_date(2026, 2, 1),
        Some(7),
    )
    .expect("valid");
    assert_eq!(c.module_scope_id, Some(7));
}

// --- CycleFeature tests ---

#[test]
fn cycle_feature_new_stamps_added_at() {
    let before = Utc::now();
    let cf = CycleFeature::new(10, 20);
    let after = Utc::now();
    assert_eq!(cf.cycle_id, 10);
    assert_eq!(cf.feature_id, 20);
    assert!(cf.added_at >= before);
    assert!(cf.added_at <= after);
}

// --- is_shippable tests ---

fn make_cycle_with_features(features: Vec<Feature>) -> CycleWithFeatures {
    let cycle = Cycle::new("C", make_date(2026, 1, 1), make_date(2026, 2, 1), None).expect("valid");
    CycleWithFeatures {
        cycle,
        features,
        wp_progress: WpProgressSummary::default(),
    }
}

#[test]
fn empty_cycle_is_shippable_true() {
    // Vacuously true: no features means no blocking features.
    let cwf = make_cycle_with_features(vec![]);
    assert!(cwf.is_shippable());
}

#[test]
fn all_validated_is_shippable() {
    let mut f = Feature::new("f", "F", [0u8; 32], None);
    f.state = FeatureState::Validated;
    let cwf = make_cycle_with_features(vec![f]);
    assert!(cwf.is_shippable());
}

#[test]
fn all_shipped_is_shippable() {
    let mut f = Feature::new("f", "F", [0u8; 32], None);
    f.state = FeatureState::Shipped;
    let cwf = make_cycle_with_features(vec![f]);
    assert!(cwf.is_shippable());
}

#[test]
fn mixed_states_not_shippable() {
    let mut f1 = Feature::new("f1", "F1", [0u8; 32], None);
    f1.state = FeatureState::Validated;
    let f2 = Feature::new("f2", "F2", [0u8; 32], None); // Created (default)
    let cwf = make_cycle_with_features(vec![f1, f2]);
    assert!(!cwf.is_shippable());
}
