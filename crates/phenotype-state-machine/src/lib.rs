//! Generic finite state machine with transition guards, validation, and forward-only enforcement.
//!
//! Extracted from AgilePlus domain state machine pattern. Provides a reusable FSM
//! that can be parameterized with any state enum implementing the `State` trait.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors for state machine operations.
#[derive(Debug, thiserror::Error)]
pub enum StateMachineError {
    #[error("invalid transition from {from} to {to}: {reason}")]
    InvalidTransition {
        from: String,
        to: String,
        reason: String,
    },

    #[error("guard rejected transition from {from} to {to}: {reason}")]
    GuardRejected {
        from: String,
        to: String,
        reason: String,
    },
}

/// A state in a finite state machine. Implement this for your state enum.
pub trait State:
    fmt::Display + Clone + PartialEq + Eq + Serialize + serde::de::DeserializeOwned
{
    /// Ordered index for determining valid forward transitions.
    /// Lower ordinals come before higher ordinals in the lifecycle.
    fn ordinal(&self) -> u32;

    /// All states in lifecycle order.
    fn all_states() -> Vec<Self>;
}

/// A guard that can approve or reject a transition.
pub trait TransitionGuard<S: State> {
    /// Check whether the transition is allowed. Return `Ok(())` to allow,
    /// or `Err(reason)` to reject.
    fn check(&self, from: &S, to: &S) -> Result<(), String>;
}

/// Record of a state transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "S: State")]
pub struct TransitionRecord<S: State> {
    pub from: S,
    pub to: S,
    pub skipped: Vec<S>,
    pub timestamp: DateTime<Utc>,
}

impl<S: State> fmt::Display for TransitionRecord<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.from, self.to)
    }
}

/// Configuration for the state machine.
#[derive(Debug, Clone)]
pub struct StateMachineConfig {
    /// Whether to allow skipping intermediate states (forward jumps).
    pub allow_skip: bool,
}

impl Default for StateMachineConfig {
    fn default() -> Self {
        Self { allow_skip: true }
    }
}

/// Generic finite state machine with forward-only enforcement and optional guards.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "S: State")]
pub struct StateMachine<S: State> {
    current: S,
    history: Vec<TransitionRecord<S>>,
}

impl<S: State> StateMachine<S> {
    /// Create a new state machine starting at the given state.
    pub fn new(initial: S) -> Self {
        Self {
            current: initial,
            history: Vec::new(),
        }
    }

    /// Get the current state.
    pub fn current(&self) -> &S {
        &self.current
    }

    /// Get the transition history.
    pub fn history(&self) -> &[TransitionRecord<S>] {
        &self.history
    }

    /// Attempt a transition to the target state (forward-only, no guards).
    pub fn transition(&mut self, target: S) -> Result<&TransitionRecord<S>, StateMachineError> {
        self.transition_with_config(target, &StateMachineConfig::default())
    }

    /// Attempt a transition with custom configuration.
    pub fn transition_with_config(
        &mut self,
        target: S,
        config: &StateMachineConfig,
    ) -> Result<&TransitionRecord<S>, StateMachineError> {
        // Enforce forward-only
        if target.ordinal() <= self.current.ordinal() {
            return Err(StateMachineError::InvalidTransition {
                from: self.current.to_string(),
                to: target.to_string(),
                reason: "backward transitions are not allowed".to_string(),
            });
        }

        // Check for skipped states
        let skipped: Vec<S> = S::all_states()
            .into_iter()
            .filter(|s| s.ordinal() > self.current.ordinal() && s.ordinal() < target.ordinal())
            .collect();

        if !config.allow_skip && !skipped.is_empty() {
            return Err(StateMachineError::InvalidTransition {
                from: self.current.to_string(),
                to: target.to_string(),
                reason: format!(
                    "skipping states is not allowed; would skip: {}",
                    skipped
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            });
        }

        let record = TransitionRecord {
            from: self.current.clone(),
            to: target.clone(),
            skipped,
            timestamp: Utc::now(),
        };

        self.current = target;
        self.history.push(record);
        Ok(self.history.last().unwrap())
    }

    /// Attempt a guarded transition. The guard is checked before the transition proceeds.
    pub fn guarded_transition(
        &mut self,
        target: S,
        guard: &dyn TransitionGuard<S>,
    ) -> Result<&TransitionRecord<S>, StateMachineError> {
        // Check guard first
        guard
            .check(&self.current, &target)
            .map_err(|reason| StateMachineError::GuardRejected {
                from: self.current.to_string(),
                to: target.to_string(),
                reason,
            })?;

        self.transition(target)
    }

    /// Check if a transition to the target state would be valid (forward-only check only).
    pub fn can_transition(&self, target: &S) -> bool {
        target.ordinal() > self.current.ordinal()
    }

    /// Get the list of reachable states from the current state.
    pub fn reachable_states(&self) -> Vec<S> {
        S::all_states()
            .into_iter()
            .filter(|s| s.ordinal() > self.current.ordinal())
            .collect()
    }
}

impl<S: State> fmt::Display for StateMachine<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StateMachine({})", self.current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    enum TestState {
        Created,
        InProgress,
        Review,
        Done,
    }

    impl fmt::Display for TestState {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Created => write!(f, "created"),
                Self::InProgress => write!(f, "in_progress"),
                Self::Review => write!(f, "review"),
                Self::Done => write!(f, "done"),
            }
        }
    }

    impl State for TestState {
        fn ordinal(&self) -> u32 {
            match self {
                Self::Created => 0,
                Self::InProgress => 1,
                Self::Review => 2,
                Self::Done => 3,
            }
        }

        fn all_states() -> Vec<Self> {
            vec![Self::Created, Self::InProgress, Self::Review, Self::Done]
        }
    }

    #[test]
    fn forward_transition() {
        let mut sm = StateMachine::new(TestState::Created);
        let record = sm.transition(TestState::InProgress).unwrap();
        assert_eq!(record.from, TestState::Created);
        assert_eq!(record.to, TestState::InProgress);
        assert!(record.skipped.is_empty());
        assert_eq!(*sm.current(), TestState::InProgress);
    }

    #[test]
    fn backward_transition_rejected() {
        let mut sm = StateMachine::new(TestState::Review);
        let err = sm.transition(TestState::Created).unwrap_err();
        match err {
            StateMachineError::InvalidTransition { from, to, .. } => {
                assert_eq!(from, "review");
                assert_eq!(to, "created");
            }
            _ => panic!("expected InvalidTransition"),
        }
    }

    #[test]
    fn same_state_transition_rejected() {
        let mut sm = StateMachine::new(TestState::InProgress);
        assert!(sm.transition(TestState::InProgress).is_err());
    }

    #[test]
    fn skip_states_allowed_by_default() {
        let mut sm = StateMachine::new(TestState::Created);
        let record = sm.transition(TestState::Done).unwrap();
        assert_eq!(record.skipped.len(), 2);
        assert!(record.skipped.contains(&TestState::InProgress));
        assert!(record.skipped.contains(&TestState::Review));
    }

    #[test]
    fn skip_states_rejected_when_disallowed() {
        let mut sm = StateMachine::new(TestState::Created);
        let config = StateMachineConfig { allow_skip: false };
        let err = sm
            .transition_with_config(TestState::Done, &config)
            .unwrap_err();
        match err {
            StateMachineError::InvalidTransition { reason, .. } => {
                assert!(reason.contains("skipping"));
            }
            _ => panic!("expected InvalidTransition"),
        }
    }

    #[test]
    fn guard_allows_transition() {
        struct AllowAll;
        impl TransitionGuard<TestState> for AllowAll {
            fn check(&self, _from: &TestState, _to: &TestState) -> Result<(), String> {
                Ok(())
            }
        }

        let mut sm = StateMachine::new(TestState::Created);
        sm.guarded_transition(TestState::InProgress, &AllowAll)
            .unwrap();
        assert_eq!(*sm.current(), TestState::InProgress);
    }

    #[test]
    fn guard_rejects_transition() {
        struct DenyAll;
        impl TransitionGuard<TestState> for DenyAll {
            fn check(&self, _from: &TestState, _to: &TestState) -> Result<(), String> {
                Err("not allowed".to_string())
            }
        }

        let mut sm = StateMachine::new(TestState::Created);
        let err = sm
            .guarded_transition(TestState::InProgress, &DenyAll)
            .unwrap_err();
        match err {
            StateMachineError::GuardRejected { reason, .. } => {
                assert_eq!(reason, "not allowed");
            }
            _ => panic!("expected GuardRejected"),
        }
    }

    #[test]
    fn can_transition_check() {
        let sm = StateMachine::new(TestState::InProgress);
        assert!(sm.can_transition(&TestState::Review));
        assert!(sm.can_transition(&TestState::Done));
        assert!(!sm.can_transition(&TestState::Created));
        assert!(!sm.can_transition(&TestState::InProgress));
    }

    #[test]
    fn reachable_states() {
        let sm = StateMachine::new(TestState::InProgress);
        let reachable = sm.reachable_states();
        assert_eq!(reachable, vec![TestState::Review, TestState::Done]);
    }

    #[test]
    fn history_tracking() {
        let mut sm = StateMachine::new(TestState::Created);
        sm.transition(TestState::InProgress).unwrap();
        sm.transition(TestState::Review).unwrap();
        assert_eq!(sm.history().len(), 2);
    }

    #[test]
    fn display_impl() {
        let sm = StateMachine::new(TestState::Created);
        assert_eq!(format!("{}", sm), "StateMachine(created)");
    }
}
