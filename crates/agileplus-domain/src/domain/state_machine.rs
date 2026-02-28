use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureState {
    Created,
    Specified,
    Researched,
    Planned,
    Implementing,
    Validated,
    Shipped,
    Retrospected,
}

impl FeatureState {
    pub fn ordinal(&self) -> u8 {
        match self {
            Self::Created => 0,
            Self::Specified => 1,
            Self::Researched => 2,
            Self::Planned => 3,
            Self::Implementing => 4,
            Self::Validated => 5,
            Self::Shipped => 6,
            Self::Retrospected => 7,
        }
    }

    pub fn from_ordinal(n: u8) -> Option<Self> {
        match n {
            0 => Some(Self::Created),
            1 => Some(Self::Specified),
            2 => Some(Self::Researched),
            3 => Some(Self::Planned),
            4 => Some(Self::Implementing),
            5 => Some(Self::Validated),
            6 => Some(Self::Shipped),
            7 => Some(Self::Retrospected),
            _ => None,
        }
    }

    pub fn next(&self) -> Option<FeatureState> {
        Self::from_ordinal(self.ordinal() + 1)
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Created => "Created",
            Self::Specified => "Specified",
            Self::Researched => "Researched",
            Self::Planned => "Planned",
            Self::Implementing => "Implementing",
            Self::Validated => "Validated",
            Self::Shipped => "Shipped",
            Self::Retrospected => "Retrospected",
        }
    }

    /// Attempt a state transition.
    ///
    /// - same state -> `Err(NoOpTransition)`
    /// - backward -> `Err(InvalidTransition)`
    /// - from Retrospected -> `Err(InvalidTransition)` (terminal)
    /// - next state -> `Ok(TransitionResult::Ok(...))`
    /// - skip forward > 1 -> `Ok(TransitionResult::Warning { ... })`
    pub fn transition(self, target: FeatureState) -> Result<TransitionResult, DomainError> {
        if self == target {
            return Err(DomainError::NoOpTransition(self.to_string()));
        }
        if self == Self::Retrospected {
            return Err(DomainError::InvalidTransition {
                from: self.to_string(),
                to: target.to_string(),
                reason: "retrospected is a terminal state".into(),
            });
        }
        if target.ordinal() < self.ordinal() {
            return Err(DomainError::InvalidTransition {
                from: self.to_string(),
                to: target.to_string(),
                reason: "backward transitions are not allowed".into(),
            });
        }

        let skipped: Vec<FeatureState> = ((self.ordinal() + 1)..target.ordinal())
            .filter_map(Self::from_ordinal)
            .collect();

        let transition = StateTransition {
            from: self,
            to: target,
            skipped: skipped.clone(),
        };

        if skipped.is_empty() {
            Ok(TransitionResult::Ok(transition))
        } else {
            let names: Vec<&str> = skipped.iter().map(|s| s.display_name()).collect();
            Ok(TransitionResult::Warning {
                transition,
                message: format!("skipped states: {}", names.join(", ")),
            })
        }
    }
}

impl fmt::Display for FeatureState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

impl FromStr for FeatureState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "created" => Ok(Self::Created),
            "specified" => Ok(Self::Specified),
            "researched" => Ok(Self::Researched),
            "planned" => Ok(Self::Planned),
            "implementing" => Ok(Self::Implementing),
            "validated" => Ok(Self::Validated),
            "shipped" => Ok(Self::Shipped),
            "retrospected" => Ok(Self::Retrospected),
            _ => Err(format!("unknown feature state: {s}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StateTransition {
    pub from: FeatureState,
    pub to: FeatureState,
    pub skipped: Vec<FeatureState>,
}

impl fmt::Display for StateTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.from, self.to)?;
        if !self.skipped.is_empty() {
            let names: Vec<&str> = self.skipped.iter().map(|s| s.display_name()).collect();
            write!(f, " (skipped: {})", names.join(", "))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum TransitionResult {
    Ok(StateTransition),
    Warning {
        transition: StateTransition,
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- 7 valid forward transitions ---
    #[test]
    fn forward_created_to_specified() {
        let r = FeatureState::Created
            .transition(FeatureState::Specified)
            .unwrap();
        assert!(matches!(r, TransitionResult::Ok(_)));
    }

    #[test]
    fn forward_specified_to_researched() {
        let r = FeatureState::Specified
            .transition(FeatureState::Researched)
            .unwrap();
        assert!(matches!(r, TransitionResult::Ok(_)));
    }

    #[test]
    fn forward_researched_to_planned() {
        let r = FeatureState::Researched
            .transition(FeatureState::Planned)
            .unwrap();
        assert!(matches!(r, TransitionResult::Ok(_)));
    }

    #[test]
    fn forward_planned_to_implementing() {
        let r = FeatureState::Planned
            .transition(FeatureState::Implementing)
            .unwrap();
        assert!(matches!(r, TransitionResult::Ok(_)));
    }

    #[test]
    fn forward_implementing_to_validated() {
        let r = FeatureState::Implementing
            .transition(FeatureState::Validated)
            .unwrap();
        assert!(matches!(r, TransitionResult::Ok(_)));
    }

    #[test]
    fn forward_validated_to_shipped() {
        let r = FeatureState::Validated
            .transition(FeatureState::Shipped)
            .unwrap();
        assert!(matches!(r, TransitionResult::Ok(_)));
    }

    #[test]
    fn forward_shipped_to_retrospected() {
        let r = FeatureState::Shipped
            .transition(FeatureState::Retrospected)
            .unwrap();
        assert!(matches!(r, TransitionResult::Ok(_)));
    }

    // --- 8 no-op transitions ---
    #[test]
    fn noop_created() {
        assert!(matches!(
            FeatureState::Created.transition(FeatureState::Created),
            Err(DomainError::NoOpTransition(_))
        ));
    }

    #[test]
    fn noop_specified() {
        assert!(matches!(
            FeatureState::Specified.transition(FeatureState::Specified),
            Err(DomainError::NoOpTransition(_))
        ));
    }

    #[test]
    fn noop_researched() {
        assert!(matches!(
            FeatureState::Researched.transition(FeatureState::Researched),
            Err(DomainError::NoOpTransition(_))
        ));
    }

    #[test]
    fn noop_planned() {
        assert!(matches!(
            FeatureState::Planned.transition(FeatureState::Planned),
            Err(DomainError::NoOpTransition(_))
        ));
    }

    #[test]
    fn noop_implementing() {
        assert!(matches!(
            FeatureState::Implementing.transition(FeatureState::Implementing),
            Err(DomainError::NoOpTransition(_))
        ));
    }

    #[test]
    fn noop_validated() {
        assert!(matches!(
            FeatureState::Validated.transition(FeatureState::Validated),
            Err(DomainError::NoOpTransition(_))
        ));
    }

    #[test]
    fn noop_shipped() {
        assert!(matches!(
            FeatureState::Shipped.transition(FeatureState::Shipped),
            Err(DomainError::NoOpTransition(_))
        ));
    }

    #[test]
    fn noop_retrospected() {
        assert!(matches!(
            FeatureState::Retrospected.transition(FeatureState::Retrospected),
            Err(DomainError::NoOpTransition(_))
        ));
    }

    // --- 7+ invalid backward transitions ---
    #[test]
    fn backward_specified_to_created() {
        assert!(matches!(
            FeatureState::Specified.transition(FeatureState::Created),
            Err(DomainError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn backward_researched_to_specified() {
        assert!(matches!(
            FeatureState::Researched.transition(FeatureState::Specified),
            Err(DomainError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn backward_planned_to_created() {
        assert!(matches!(
            FeatureState::Planned.transition(FeatureState::Created),
            Err(DomainError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn backward_implementing_to_planned() {
        assert!(matches!(
            FeatureState::Implementing.transition(FeatureState::Planned),
            Err(DomainError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn backward_validated_to_created() {
        assert!(matches!(
            FeatureState::Validated.transition(FeatureState::Created),
            Err(DomainError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn backward_shipped_to_implementing() {
        assert!(matches!(
            FeatureState::Shipped.transition(FeatureState::Implementing),
            Err(DomainError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn backward_retrospected_to_shipped() {
        assert!(matches!(
            FeatureState::Retrospected.transition(FeatureState::Shipped),
            Err(DomainError::InvalidTransition { .. })
        ));
    }

    // --- 5+ skip-with-warning transitions ---
    #[test]
    fn skip_created_to_planned() {
        let r = FeatureState::Created
            .transition(FeatureState::Planned)
            .unwrap();
        assert!(matches!(r, TransitionResult::Warning { .. }));
    }

    #[test]
    fn skip_created_to_implementing() {
        let r = FeatureState::Created
            .transition(FeatureState::Implementing)
            .unwrap();
        assert!(matches!(r, TransitionResult::Warning { .. }));
    }

    #[test]
    fn skip_created_to_retrospected() {
        let r = FeatureState::Created
            .transition(FeatureState::Retrospected)
            .unwrap();
        assert!(matches!(r, TransitionResult::Warning { .. }));
    }

    #[test]
    fn skip_specified_to_implementing() {
        let r = FeatureState::Specified
            .transition(FeatureState::Implementing)
            .unwrap();
        assert!(matches!(r, TransitionResult::Warning { .. }));
    }

    #[test]
    fn skip_planned_to_shipped() {
        let r = FeatureState::Planned
            .transition(FeatureState::Shipped)
            .unwrap();
        assert!(matches!(r, TransitionResult::Warning { .. }));
    }

    // --- ordinal / from_ordinal / next / display / fromstr ---
    #[test]
    fn ordinal_roundtrip() {
        for i in 0..=7u8 {
            let s = FeatureState::from_ordinal(i).unwrap();
            assert_eq!(s.ordinal(), i);
        }
        assert!(FeatureState::from_ordinal(8).is_none());
    }

    #[test]
    fn next_chain() {
        let mut s = FeatureState::Created;
        let mut count = 0;
        while let Some(n) = s.next() {
            s = n;
            count += 1;
        }
        assert_eq!(count, 7);
        assert_eq!(s, FeatureState::Retrospected);
    }

    #[test]
    fn display_and_fromstr() {
        for i in 0..=7u8 {
            let s = FeatureState::from_ordinal(i).unwrap();
            let name = s.to_string();
            let parsed: FeatureState = name.parse().unwrap();
            assert_eq!(parsed, s);
        }
    }

    #[test]
    fn retrospected_terminal() {
        assert!(matches!(
            FeatureState::Retrospected.transition(FeatureState::Created),
            Err(DomainError::InvalidTransition { .. })
        ));
    }

    // --- proptest ---
    use proptest::prelude::*;

    fn arb_state() -> impl Strategy<Value = FeatureState> {
        (0u8..=7).prop_map(|n| FeatureState::from_ordinal(n).unwrap())
    }

    proptest! {
        #[test]
        fn prop_forward_never_errors(from in arb_state(), to in arb_state()) {
            if to.ordinal() > from.ordinal() {
                prop_assert!(from.transition(to).is_ok());
            }
        }

        #[test]
        fn prop_backward_always_errors(from in arb_state(), to in arb_state()) {
            if to.ordinal() < from.ordinal() {
                prop_assert!(from.transition(to).is_err());
            }
        }

        #[test]
        fn prop_skip_produces_warning(from in arb_state(), to in arb_state()) {
            if to.ordinal() > from.ordinal() + 1 {
                let r = from.transition(to).unwrap();
                let is_warning = matches!(r, TransitionResult::Warning { .. });
                prop_assert!(is_warning);
            }
        }
    }

    #[test]
    fn transition_display() {
        let t = StateTransition {
            from: FeatureState::Created,
            to: FeatureState::Planned,
            skipped: vec![FeatureState::Specified, FeatureState::Researched],
        };
        let s = t.to_string();
        assert!(s.contains("Created"));
        assert!(s.contains("Planned"));
        assert!(s.contains("skipped"));
    }
}
