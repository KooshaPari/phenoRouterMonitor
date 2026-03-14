//! Cycle domain entity, CycleState lifecycle, and related types.
//!
//! Traces to: FR-C01, FR-C02, FR-C03, FR-C04, FR-C05, FR-C07

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::domain::state_machine::FeatureState;
use crate::error::DomainError;

/// Lifecycle state for a Cycle.
///
/// Allowed transitions:
/// ```text
/// Draft   -> Active
/// Active  -> Review
/// Active  -> Draft      (revert)
/// Review  -> Shipped    (gate enforced in WP02/WP04, not here)
/// Review  -> Active     (changes requested)
/// Shipped -> Archived
/// ```
///
/// Traces to: FR-C02
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum CycleState {
    Draft,
    Active,
    Review,
    Shipped,
    Archived,
}

impl fmt::Display for CycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Draft => "Draft",
                Self::Active => "Active",
                Self::Review => "Review",
                Self::Shipped => "Shipped",
                Self::Archived => "Archived",
            }
        )
    }
}

impl FromStr for CycleState {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Draft" => Ok(Self::Draft),
            "Active" => Ok(Self::Active),
            "Review" => Ok(Self::Review),
            "Shipped" => Ok(Self::Shipped),
            "Archived" => Ok(Self::Archived),
            other => Err(DomainError::Other(format!("unknown cycle state: {other}"))),
        }
    }
}

impl CycleState {
    /// Validate a transition from `self` to `target`.
    ///
    /// Returns `Ok(())` for allowed edges, `Err(NoOpTransition)` for self-to-self,
    /// and `Err(InvalidTransition)` for all other pairs.
    pub fn transition(self, target: CycleState) -> Result<(), DomainError> {
        if self == target {
            return Err(DomainError::NoOpTransition(self.to_string()));
        }
        let allowed = matches!(
            (self, target),
            (CycleState::Draft, CycleState::Active)
                | (CycleState::Active, CycleState::Review)
                | (CycleState::Active, CycleState::Draft)
                | (CycleState::Review, CycleState::Shipped)
                | (CycleState::Review, CycleState::Active)
                | (CycleState::Shipped, CycleState::Archived)
        );
        if allowed {
            Ok(())
        } else {
            Err(DomainError::InvalidTransition {
                from: self.to_string(),
                to: target.to_string(),
                reason: format!(
                    "transition from {self} to {target} is not a permitted edge in the cycle state graph"
                ),
            })
        }
    }
}

/// A Cycle groups Features into a time-boxed delivery unit.
///
/// Traces to: FR-C01
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cycle {
    pub id: i64,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub state: CycleState,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    /// Optional Module scope; if set, only features owned/tagged to that Module may be assigned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_scope_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Cycle {
    /// Create a new Cycle in `Draft` state.
    ///
    /// Returns `Err` if `end_date` is not strictly after `start_date`.
    pub fn new(
        name: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        module_scope_id: Option<i64>,
    ) -> Result<Self, DomainError> {
        if end_date <= start_date {
            return Err(DomainError::Other(
                "end_date must be after start_date".to_string(),
            ));
        }
        let now = Utc::now();
        Ok(Self {
            id: 0,
            name: name.to_string(),
            description: None,
            state: CycleState::Draft,
            start_date,
            end_date,
            module_scope_id,
            created_at: now,
            updated_at: now,
        })
    }

    /// Transition this Cycle to `target`, updating `state` and `updated_at` on success.
    ///
    /// Note: the Review -> Shipped gate (all features validated) is enforced by the
    /// storage/service layer in WP02 and CLI in WP04 -- this method validates only
    /// the state graph edges.
    pub fn transition(&mut self, target: CycleState) -> Result<(), DomainError> {
        self.state.transition(target)?;
        self.state = target;
        self.updated_at = Utc::now();
        Ok(())
    }
}

/// Many-to-many assignment join between a Cycle and a Feature.
///
/// Traces to: FR-C03
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleFeature {
    pub cycle_id: i64,
    pub feature_id: i64,
    pub added_at: DateTime<Utc>,
}

impl CycleFeature {
    pub fn new(cycle_id: i64, feature_id: i64) -> Self {
        Self {
            cycle_id,
            feature_id,
            added_at: Utc::now(),
        }
    }
}

/// Aggregate count of work packages per state across all assigned features.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WpProgressSummary {
    pub total: u32,
    pub planned: u32,
    pub in_progress: u32,
    pub done: u32,
    pub blocked: u32,
}

/// View struct carrying a Cycle together with its assigned features and WP progress.
/// Populated by the storage/query layer in WP02.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleWithFeatures {
    pub cycle: Cycle,
    pub features: Vec<crate::domain::feature::Feature>,
    /// Aggregate count of WPs per WpState across all assigned features.
    pub wp_progress: WpProgressSummary,
}

impl CycleWithFeatures {
    /// Return `true` when the cycle is safe to ship.
    ///
    /// Per FR-C07: all assigned Features must be in `Validated` or `Shipped` state.
    /// An empty feature list is treated as vacuously shippable (no blocking features).
    /// A Cycle with no assigned Features is a planning placeholder -- callers may
    /// apply an additional "at least one feature" guard at the service layer.
    pub fn is_shippable(&self) -> bool {
        self.features
            .iter()
            .all(|f| matches!(f.state, FeatureState::Validated | FeatureState::Shipped))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::feature::Feature;

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
        let c = Cycle::new("Q1", make_date(2026, 1, 1), make_date(2026, 3, 31), None)
            .expect("valid dates");
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
        let mut c =
            Cycle::new("C", make_date(2026, 1, 1), make_date(2026, 2, 1), None).expect("valid");
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
        let cycle =
            Cycle::new("C", make_date(2026, 1, 1), make_date(2026, 2, 1), None).expect("valid");
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
}
