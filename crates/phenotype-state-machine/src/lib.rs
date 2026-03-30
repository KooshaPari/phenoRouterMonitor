//! Generic state machine implementation for Phenotype.

use std::fmt;
use std::hash::Hash;

/// State machine errors.
#[derive(Debug, Clone)]
pub enum TransitionError {
    InvalidTransition {
        from: String,
        to: String,
    },
    GuardFailed {
        reason: String,
    },
    StateNotFound(String),
}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransitionError::InvalidTransition { from, to } => {
                write!(f, "Invalid transition from {} to {}", from, to)
            }
            TransitionError::GuardFailed { reason } => {
                write!(f, "Guard failed: {}", reason)
            }
            TransitionError::StateNotFound(state) => {
                write!(f, "State not found: {}", state)
            }
        }
    }
}

impl std::error::Error for TransitionError {}

/// Guard function type: checks if a transition is allowed.
pub type Guard<S> = Box<dyn Fn(&S) -> bool>;

/// A transition from one state to another with optional guard.
pub struct Transition<S: Clone + Eq + Hash> {
    from: S,
    to: S,
    guard: Option<Guard<S>>,
}

impl<S: Clone + Eq + Hash> Transition<S> {
    /// Create a new transition without a guard.
    pub fn new(from: S, to: S) -> Self {
        Self {
            from,
            to,
            guard: None,
        }
    }

    /// Create a new transition with a guard function.
    pub fn with_guard(from: S, to: S, guard: Guard<S>) -> Self {
        Self {
            from,
            to,
            guard: Some(guard),
        }
    }

    /// Check if transition can occur from a given state.
    fn can_transition(&self, from: &S) -> bool {
        if self.from != *from {
            return false;
        }
        if let Some(ref guard) = self.guard {
            guard(from)
        } else {
            true
        }
    }
}

/// A generic state machine.
pub struct StateMachine<S: Clone + Eq + Hash> {
    current_state: S,
    transitions: Vec<Transition<S>>,
}

impl<S: Clone + Eq + Hash> StateMachine<S> {
    /// Create a new state machine with an initial state.
    pub fn new(initial: S) -> Self {
        Self {
            current_state: initial,
            transitions: Vec::new(),
        }
    }

    /// Add a transition to the state machine.
    pub fn add_transition(&mut self, transition: Transition<S>) {
        self.transitions.push(transition);
    }

    /// Get the current state.
    pub fn current_state(&self) -> &S {
        &self.current_state
    }

    /// Check if a transition to a target state is possible.
    pub fn can_transition(&self, target: &S) -> bool {
        self.transitions
            .iter()
            .any(|t| t.from == self.current_state && t.to == *target && t.can_transition(&self.current_state))
    }

    /// Attempt to transition to a target state.
    pub fn transition(&mut self, target: S) -> Result<S, TransitionError> {
        let matching_transition = self
            .transitions
            .iter()
            .find(|t| t.from == self.current_state && t.to == target);

        match matching_transition {
            Some(t) => {
                if t.can_transition(&self.current_state) {
                    let old_state = self.current_state.clone();
                    self.current_state = target.clone();
                    Ok(old_state)
                } else {
                    Err(TransitionError::GuardFailed {
                        reason: "Guard condition not met".to_string(),
                    })
                }
            }
            None => Err(TransitionError::InvalidTransition {
                from: "current state".to_string(),
                to: "target state".to_string(),
            }),
        }
    }

    /// List all valid target states from the current state.
    pub fn valid_targets(&self) -> Vec<S> {
        self.transitions
            .iter()
            .filter(|t| t.from == self.current_state && t.can_transition(&self.current_state))
            .map(|t| t.to.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum TrafficLight {
        Red,
        Yellow,
        Green,
    }

    #[test]
    fn test_linear_transitions() {
        let mut sm = StateMachine::new(TrafficLight::Red);
        assert_eq!(sm.current_state(), &TrafficLight::Red);

        sm.add_transition(Transition::new(TrafficLight::Red, TrafficLight::Green));
        sm.add_transition(Transition::new(TrafficLight::Green, TrafficLight::Yellow));
        sm.add_transition(Transition::new(TrafficLight::Yellow, TrafficLight::Red));

        sm.transition(TrafficLight::Green).unwrap();
        assert_eq!(sm.current_state(), &TrafficLight::Green);

        sm.transition(TrafficLight::Yellow).unwrap();
        assert_eq!(sm.current_state(), &TrafficLight::Yellow);

        sm.transition(TrafficLight::Red).unwrap();
        assert_eq!(sm.current_state(), &TrafficLight::Red);
    }

    #[test]
    fn test_invalid_transition() {
        let mut sm = StateMachine::new(TrafficLight::Red);
        sm.add_transition(Transition::new(TrafficLight::Red, TrafficLight::Green));

        let result = sm.transition(TrafficLight::Yellow);
        assert!(result.is_err());
        assert_eq!(sm.current_state(), &TrafficLight::Red);
    }

    #[test]
    fn test_can_transition() {
        let mut sm = StateMachine::new(TrafficLight::Red);
        sm.add_transition(Transition::new(TrafficLight::Red, TrafficLight::Green));
        sm.add_transition(Transition::new(TrafficLight::Green, TrafficLight::Yellow));

        assert!(sm.can_transition(&TrafficLight::Green));
        assert!(!sm.can_transition(&TrafficLight::Yellow));

        sm.transition(TrafficLight::Green).unwrap();
        assert!(sm.can_transition(&TrafficLight::Yellow));
        assert!(!sm.can_transition(&TrafficLight::Red));
    }

    #[test]
    fn test_guard_condition() {
        let mut sm = StateMachine::new(false);
        sm.add_transition(Transition::with_guard(false, true, Box::new(|state| !*state)));
        sm.add_transition(Transition::with_guard(true, false, Box::new(|state| *state)));

        assert!(sm.can_transition(&true));
        sm.transition(true).unwrap();
        assert_eq!(sm.current_state(), &true);

        assert!(sm.can_transition(&false));
        sm.transition(false).unwrap();
        assert_eq!(sm.current_state(), &false);
    }

    #[test]
    fn test_guard_failure() {
        let mut sm = StateMachine::new(0i32);
        sm.add_transition(Transition::with_guard(0, 1, Box::new(|state| *state > 5)));

        let result = sm.transition(1);
        assert!(result.is_err());
        assert_eq!(sm.current_state(), &0);
    }

    #[test]
    fn test_valid_targets() {
        let mut sm = StateMachine::new(TrafficLight::Red);
        sm.add_transition(Transition::new(TrafficLight::Red, TrafficLight::Green));
        sm.add_transition(Transition::new(TrafficLight::Red, TrafficLight::Yellow));
        sm.add_transition(Transition::new(TrafficLight::Green, TrafficLight::Yellow));

        let targets = sm.valid_targets();
        assert_eq!(targets.len(), 2);
        assert!(targets.contains(&TrafficLight::Green));
        assert!(targets.contains(&TrafficLight::Yellow));
    }

    #[test]
    fn test_branching_transitions() {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        enum Process {
            Start,
            Processing,
            Success,
            Failure,
            Retry,
        }

        let mut sm = StateMachine::new(Process::Start);
        sm.add_transition(Transition::new(Process::Start, Process::Processing));
        sm.add_transition(Transition::new(Process::Processing, Process::Success));
        sm.add_transition(Transition::new(Process::Processing, Process::Failure));
        sm.add_transition(Transition::new(Process::Failure, Process::Retry));
        sm.add_transition(Transition::new(Process::Retry, Process::Processing));

        sm.transition(Process::Processing).unwrap();
        assert_eq!(sm.current_state(), &Process::Processing);

        let targets = sm.valid_targets();
        assert_eq!(targets.len(), 2);
        assert!(targets.contains(&Process::Success));
        assert!(targets.contains(&Process::Failure));

        sm.transition(Process::Failure).unwrap();
        sm.transition(Process::Retry).unwrap();
        sm.transition(Process::Processing).unwrap();
        sm.transition(Process::Success).unwrap();
        assert_eq!(sm.current_state(), &Process::Success);
    }
}
