//! Generic finite state machine with transition guards and callbacks.
//!
//! ```rust
//! use phenotype_state_machine::{StateMachine, StateMachineBuilder};
//!
//! let sm = StateMachineBuilder::new("idle")
//!     .transition("idle", "start", "running")
//!     .transition("running", "pause", "paused")
//!     .transition("paused", "resume", "running")
//!     .transition("running", "stop", "idle")
//!     .build()
//!     .unwrap();
//!
//! assert_eq!(sm.current(), "idle");
//! sm.send("start").unwrap();
//! assert_eq!(sm.current(), "running");
//! ```

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Errors that can occur during state machine operations.
#[derive(Debug, Clone, Error)]
pub enum StateMachineError {
    #[error("invalid transition: no transition from '{from}' on event '{event}'")]
    InvalidTransition { from: String, event: String },

    #[error("transition from '{from}' on '{event}' rejected by guard")]
    GuardRejected { from: String, event: String },

    #[error("unknown state: '{0}'")]
    UnknownState(String),

    #[error("builder error: {0}")]
    BuildError(String),
}

/// Result type for state machine operations.
pub type Result<T> = std::result::Result<T, StateMachineError>;

/// Type alias for transition guard: (from_state, event) -> allowed
type TransitionGuard = Arc<dyn Fn(&str, &str) -> bool + Send + Sync>;

/// Type alias for state callback: receives state name
type StateCallback = Arc<dyn Fn(&str) + Send + Sync>;

/// A transition definition: (from_state, event) -> to_state with optional guard.
#[derive(Clone)]
struct Transition {
    to: String,
    guard: Option<TransitionGuard>,
}

/// A generic finite state machine.
///
/// Thread-safe via internal `RwLock`. States and events are string-based
/// for maximum flexibility.
pub struct StateMachine {
    current: RwLock<String>,
    transitions: HashMap<(String, String), Transition>,
    on_enter: HashMap<String, Vec<StateCallback>>,
    on_exit: HashMap<String, Vec<StateCallback>>,
}

impl StateMachine {
    /// Get the current state.
    pub fn current(&self) -> String {
        self.current.read().unwrap().clone()
    }

    /// Send an event to the state machine, potentially triggering a transition.
    pub fn send(&self, event: &str) -> Result<String> {
        let mut current = self.current.write().unwrap();
        let key = (current.clone(), event.to_string());

        let transition = self
            .transitions
            .get(&key)
            .ok_or_else(|| StateMachineError::InvalidTransition {
                from: current.clone(),
                event: event.to_string(),
            })?;

        if let Some(guard) = &transition.guard {
            if !guard(&current, event) {
                return Err(StateMachineError::GuardRejected {
                    from: current.clone(),
                    event: event.to_string(),
                });
            }
        }

        // Fire on_exit callbacks for current state.
        if let Some(cbs) = self.on_exit.get(current.as_str()) {
            for cb in cbs {
                cb(&current);
            }
        }

        let new_state = transition.to.clone();

        // Fire on_enter callbacks for new state.
        if let Some(cbs) = self.on_enter.get(&new_state) {
            for cb in cbs {
                cb(&new_state);
            }
        }

        *current = new_state.clone();
        Ok(new_state)
    }

    /// Check if a transition is possible from the current state on the given event.
    pub fn can_send(&self, event: &str) -> bool {
        let current = self.current.read().unwrap();
        self.transitions
            .contains_key(&(current.clone(), event.to_string()))
    }

    /// Get all events valid from the current state.
    pub fn available_events(&self) -> Vec<String> {
        let current = self.current.read().unwrap();
        self.transitions
            .keys()
            .filter(|(from, _)| from == current.as_str())
            .map(|(_, event)| event.clone())
            .collect()
    }
}

impl fmt::Debug for StateMachine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateMachine")
            .field("current", &self.current())
            .field("transitions", &self.transitions.len())
            .finish()
    }
}

// Send + Sync are safe because internal state is behind RwLock.
unsafe impl Send for StateMachine {}
unsafe impl Sync for StateMachine {}

/// Builder for constructing a [`StateMachine`].
pub struct StateMachineBuilder {
    initial: String,
    initial_ordinal: u32,
    transitions: HashMap<(String, String), Transition>,
    on_enter: HashMap<String, Vec<StateCallback>>,
    on_exit: HashMap<String, Vec<StateCallback>>,
    forward_only: bool,
    state_ordinals: HashMap<String, u32>,
}

impl StateMachineBuilder {
    /// Create a new builder with the given initial state.
    ///
    /// Initial ordinal defaults to 0.
    pub fn new(initial: &str) -> Self {
        Self {
            initial: initial.to_string(),
            initial_ordinal: 0,
            transitions: HashMap::new(),
            on_enter: HashMap::new(),
            on_exit: HashMap::new(),
            forward_only: false,
            state_ordinals: HashMap::new(),
        }
    }

    /// Add a transition: from `from` state, on `event`, go to `to` state.
    ///
    /// The target state is assigned an ordinal based on declaration order
    /// (incremented per unique target state).
    pub fn transition(mut self, from: &str, event: &str, to: &str) -> Self {
        // Assign ordinal for target state if not already assigned
        let to_ordinal = self
            .state_ordinals
            .entry(to.to_string())
            .or_insert_with(|| {
                // Assign next ordinal based on current count
                self.state_ordinals.len() as u32
            })
            .copied()
            .unwrap_or(0);

        self.transitions.insert(
            (from.to_string(), event.to_string()),
            Transition {
                to: to.to_string(),
                to_ordinal,
                guard: None,
            },
        );
        self
    }

    /// Add a transition with explicit ordinal for the target state.
    ///
    /// Use this when you need precise control over state ordering.
    pub fn transition_with_ordinal(
        mut self,
        from: &str,
        event: &str,
        to: &str,
        ordinal: u32,
    ) -> Self {
        self.state_ordinals.insert(to.to_string(), ordinal);

        self.transitions.insert(
            (from.to_string(), event.to_string()),
            Transition {
                to: to.to_string(),
                to_ordinal: ordinal,
                guard: None,
            },
        );
        self
    }

    /// Add a guarded transition. The guard function receives (current_state, event)
    /// and returns true to allow the transition.
    pub fn guarded_transition(
        mut self,
        from: &str,
        event: &str,
        to: &str,
        guard: impl Fn(&str, &str) -> bool + Send + Sync + 'static,
    ) -> Self {
        // Assign ordinal for target state if not already assigned
        let to_ordinal = self
            .state_ordinals
            .entry(to.to_string())
            .or_insert_with(|| {
                self.state_ordinals.len() as u32
            })
            .copied()
            .unwrap_or(0);

        self.transitions.insert(
            (from.to_string(), event.to_string()),
            Transition {
                to: to.to_string(),
                to_ordinal,
                guard: Some(Arc::new(guard)),
            },
        );
        self
    }

    /// Enable forward-only mode.
    ///
    /// When enabled, transitions to states with lower ordinals are rejected.
    /// This enforces a strictly forward progression through state space.
    pub fn forward_only(mut self) -> Self {
        self.forward_only = true;
        self
    }

    /// Register a callback for when a state is entered.
    pub fn on_enter(
        mut self,
        state: &str,
        callback: impl Fn(&str) + Send + Sync + 'static,
    ) -> Self {
        self.on_enter
            .entry(state.to_string())
            .or_default()
            .push(Arc::new(callback));
        self
    }

    /// Register a callback for when a state is exited.
    pub fn on_exit(
        mut self,
        state: &str,
        callback: impl Fn(&str) + Send + Sync + 'static,
    ) -> Self {
        self.on_exit
            .entry(state.to_string())
            .or_default()
            .push(Arc::new(callback));
        self
    }

    /// Build the state machine.
    pub fn build(self) -> Result<StateMachine> {
        if self.initial.is_empty() {
            return Err(StateMachineError::BuildError(
                "initial state cannot be empty".into(),
            ));
        }

        // Ensure initial state has an ordinal
        let initial_ordinal = self
            .state_ordinals
            .get(&self.initial)
            .copied()
            .unwrap_or(0);

        Ok(StateMachine {
            current: RwLock::new(self.initial),
            current_ordinal: RwLock::new(initial_ordinal),
            transitions: self.transitions,
            on_enter: self.on_enter,
            on_exit: self.on_exit,
            forward_only: self.forward_only,
            state_ordinals: self.state_ordinals,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn traffic_light() -> StateMachine {
        StateMachineBuilder::new("red")
            .transition("red", "next", "green")
            .transition("green", "next", "yellow")
            .transition("yellow", "next", "red")
            .build()
            .unwrap()
    }

    #[test]
    fn basic_transitions() {
        let sm = traffic_light();
        assert_eq!(sm.current(), "red");
        sm.send("next").unwrap();
        assert_eq!(sm.current(), "green");
        sm.send("next").unwrap();
        assert_eq!(sm.current(), "yellow");
        sm.send("next").unwrap();
        assert_eq!(sm.current(), "red");
    }

    #[test]
    fn invalid_transition() {
        let sm = traffic_light();
        let err = sm.send("invalid").unwrap_err();
        assert!(matches!(err, StateMachineError::InvalidTransition { .. }));
    }

    #[test]
    fn can_send() {
        let sm = traffic_light();
        assert!(sm.can_send("next"));
        assert!(!sm.can_send("invalid"));
    }

    #[test]
    fn available_events() {
        let sm = traffic_light();
        assert_eq!(sm.available_events(), vec!["next"]);
    }

    #[test]
    fn guard_allows() {
        let sm = StateMachineBuilder::new("locked")
            .guarded_transition("locked", "unlock", "unlocked", |_, _| true)
            .build()
            .unwrap();
        sm.send("unlock").unwrap();
        assert_eq!(sm.current(), "unlocked");
    }

    #[test]
    fn guard_rejects() {
        let sm = StateMachineBuilder::new("locked")
            .guarded_transition("locked", "unlock", "unlocked", |_, _| false)
            .build()
            .unwrap();
        let err = sm.send("unlock").unwrap_err();
        assert!(matches!(err, StateMachineError::GuardRejected { .. }));
        assert_eq!(sm.current(), "locked");
    }

    #[test]
    fn on_enter_callback() {
        let count = Arc::new(AtomicUsize::new(0));
        let c = count.clone();
        let sm = StateMachineBuilder::new("a")
            .transition("a", "go", "b")
            .on_enter("b", move |_| {
                c.fetch_add(1, Ordering::SeqCst);
            })
            .build()
            .unwrap();
        sm.send("go").unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn on_exit_callback() {
        let count = Arc::new(AtomicUsize::new(0));
        let c = count.clone();
        let sm = StateMachineBuilder::new("a")
            .transition("a", "go", "b")
            .on_exit("a", move |_| {
                c.fetch_add(1, Ordering::SeqCst);
            })
            .build()
            .unwrap();
        sm.send("go").unwrap();
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn empty_initial_state_errors() {
        let err = StateMachineBuilder::new("").build().unwrap_err();
        assert!(matches!(err, StateMachineError::BuildError(_)));
    }

    #[test]
    fn forward_only_mode_prevents_backward_transition() {
        // Create a linear state machine: A -> B -> C
        let sm = StateMachineBuilder::new("a")
            .transition("a", "next", "b")
            .transition("b", "next", "c")
            .transition("c", "back", "b") // backward transition
            .forward_only()
            .build()
            .unwrap();

        assert!(sm.is_forward_only());

        // Go through the forward path
        sm.send("next").unwrap(); // a -> b
        sm.send("next").unwrap(); // b -> c
        assert_eq!(sm.current(), "c");

        // Try backward transition - should be blocked
        let err = sm.send("back").unwrap_err();
        assert!(matches!(err, StateMachineError::ForwardOnlyViolation { .. }));
        assert_eq!(sm.current(), "c"); // Still in c
    }

    #[test]
    fn ordinal_assignment() {
        let sm = StateMachineBuilder::new("a")
            .transition("a", "to_b", "b")
            .transition("a", "to_c", "c")
            .transition("b", "to_d", "d")
            .build()
            .unwrap();

        // Check ordinals are assigned
        assert_eq!(sm.ordinal(), Some(0)); // "a" has ordinal 0 (initial)
    }

    #[test]
    fn transition_with_explicit_ordinal() {
        let sm = StateMachineBuilder::new("start")
            .transition_with_ordinal("start", "go", "phase1", 10)
            .transition_with_ordinal("phase1", "go", "phase2", 20)
            .transition_with_ordinal("phase2", "go", "phase3", 30)
            .build()
            .unwrap();

        assert_eq!(sm.current(), "start");
        assert_eq!(sm.ordinal(), Some(10));

        sm.send("go").unwrap();
        assert_eq!(sm.current(), "phase1");
        assert_eq!(sm.ordinal(), Some(20));
    }

    #[test]
    fn thread_safety() {
        let sm = Arc::new(traffic_light());
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let sm = sm.clone();
                std::thread::spawn(move || {
                    for _ in 0..100 {
                        let _ = sm.send("next");
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        // Should be in one of the 3 states
        let state = sm.current();
        assert!(["red", "green", "yellow"].contains(&state.as_str()));
    }
}
