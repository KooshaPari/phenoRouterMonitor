//! Generic finite state machine with transition guards, callbacks, and optional
//! skip-state enforcement (E5.4).
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

use std::collections::{HashMap, HashSet};
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

    #[error("skip transition from '{from}' to '{to}' is not allowed")]
    SkipTransitionRejected { from: String, to: String },

    #[error("unknown state: '{0}'")]
    UnknownState(String),

    #[error("builder error: {0}")]
    BuildError(String),
}

/// Result type for state machine operations.
pub type Result<T> = std::result::Result<T, StateMachineError>;

/// A transition definition: (from_state, event) -> to_state with optional guard.
#[derive(Clone)]
struct Transition {
    to: String,
    guard: Option<Arc<dyn Fn(&str, &str) -> bool + Send + Sync>>,
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
    sequential_next: HashMap<String, String>,
    skip_states: HashSet<(String, String)>,
}

impl StateMachine {
    /// Get the current state.
    pub fn current(&self) -> String {
        self.current.read().unwrap().clone()
    }

    fn validate_transition<'a>(&'a self, from: &str, event: &str) -> Result<&'a Transition> {
        let key = (from.to_string(), event.to_string());
        let transition =
            self.transitions
                .get(&key)
                .ok_or_else(|| StateMachineError::InvalidTransition {
                    from: from.to_string(),
                    event: event.to_string(),
                })?;

        if let Some(expected) = self.sequential_next.get(from) {
            let to = &transition.to;
            if to != expected && !self.skip_states.contains(&(from.to_string(), to.clone())) {
                return Err(StateMachineError::SkipTransitionRejected {
                    from: from.to_string(),
                    to: to.clone(),
                });
            }
        }

        Ok(transition)
    }

    /// Send an event to the state machine, potentially triggering a transition.
    pub fn send(&self, event: &str) -> Result<String> {
        let mut current = self.current.write().unwrap();
        let from = current.clone();
        let transition = self.validate_transition(&from, event)?;

        if let Some(guard) = &transition.guard {
            if !guard(&from, event) {
                return Err(StateMachineError::GuardRejected {
                    from: from.clone(),
                    event: event.to_string(),
                });
            }
        }

        if let Some(cbs) = self.on_exit.get(from.as_str()) {
            for cb in cbs {
                cb(&from);
            }
        }

        let new_state = transition.to.clone();

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
        self.validate_transition(current.as_str(), event).is_ok()
            && self
                .transitions
                .get(&(current.clone(), event.to_string()))
                .map(|t| {
                    if let Some(g) = &t.guard {
                        g(current.as_str(), event)
                    } else {
                        true
                    }
                })
                .unwrap_or(false)
    }

    /// Get all events valid from the current state.
    pub fn available_events(&self) -> Vec<String> {
        let current = self.current.read().unwrap();
        let mut events: Vec<String> = self
            .transitions
            .keys()
            .filter(|(from, _)| from == current.as_str())
            .filter(|(_, ev)| self.validate_transition(current.as_str(), ev).is_ok())
            .map(|(_, event)| event.clone())
            .collect();
        events.sort();
        events.dedup();
        events
    }
}

impl fmt::Debug for StateMachine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateMachine")
            .field("current", &self.current())
            .field("transitions", &self.transitions.len())
            .field("sequential_next", &self.sequential_next.len())
            .field("skip_states", &self.skip_states.len())
            .finish()
    }
}

// Send + Sync are safe because internal state is behind RwLock.
unsafe impl Send for StateMachine {}
unsafe impl Sync for StateMachine {}

/// Builder for constructing a [`StateMachine`].
pub struct StateMachineBuilder {
    initial: String,
    transitions: HashMap<(String, String), Transition>,
    on_enter: HashMap<String, Vec<Arc<dyn Fn(&str) + Send + Sync>>>,
    on_exit: HashMap<String, Vec<Arc<dyn Fn(&str) + Send + Sync>>>,
    sequential_next: HashMap<String, String>,
    skip_states: HashSet<(String, String)>,
}

impl StateMachineBuilder {
    /// Create a new builder with the given initial state.
    pub fn new(initial: &str) -> Self {
        Self {
            initial: initial.to_string(),
            transitions: HashMap::new(),
            on_enter: HashMap::new(),
            on_exit: HashMap::new(),
            sequential_next: HashMap::new(),
            skip_states: HashSet::new(),
        }
    }

    /// Add a transition: from `from` state, on `event`, go to `to` state.
    pub fn transition(mut self, from: &str, event: &str, to: &str) -> Self {
        self.transitions.insert(
            (from.to_string(), event.to_string()),
            Transition {
                to: to.to_string(),
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
        self.transitions.insert(
            (from.to_string(), event.to_string()),
            Transition {
                to: to.to_string(),
                guard: Some(Arc::new(guard)),
            },
        );
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
    pub fn on_exit(mut self, state: &str, callback: impl Fn(&str) + Send + Sync + 'static) -> Self {
        self.on_exit
            .entry(state.to_string())
            .or_default()
            .push(Arc::new(callback));
        self
    }

    /// Declare the normal sequential next state for `from`.
    pub fn sequential_transition(mut self, from: &str, next: &str) -> Self {
        self.sequential_next
            .insert(from.to_string(), next.to_string());
        self
    }

    /// Register an allowed skip-state pair `(from, to)`.
    pub fn skip_transition(mut self, from: &str, to: &str) -> Result<Self> {
        if !self.sequential_next.contains_key(from) {
            return Err(StateMachineError::BuildError(format!(
                "skip_transition: state '{from}' has no sequential_next configured"
            )));
        }
        self.skip_states.insert((from.to_string(), to.to_string()));
        Ok(self)
    }

    /// Build the state machine.
    pub fn build(self) -> Result<StateMachine> {
        if self.initial.is_empty() {
            return Err(StateMachineError::BuildError(
                "initial state cannot be empty".into(),
            ));
        }

        for (from, to) in &self.skip_states {
            let has_path = self
                .transitions
                .iter()
                .any(|((f, _), tr)| f == from && &tr.to == to);
            if !has_path {
                return Err(StateMachineError::BuildError(format!(
                    "skip_transition ({from}, {to}) has no transition with that target"
                )));
            }
        }

        Ok(StateMachine {
            current: RwLock::new(self.initial),
            transitions: self.transitions,
            on_enter: self.on_enter,
            on_exit: self.on_exit,
            sequential_next: self.sequential_next,
            skip_states: self.skip_states,
        })
    }
}

impl fmt::Debug for StateMachineBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateMachineBuilder")
            .field("initial", &self.initial)
            .field("transitions", &self.transitions.len())
            .field("sequential_next", &self.sequential_next)
            .field("skip_states", &self.skip_states)
            .finish()
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
        let state = sm.current();
        assert!(["red", "green", "yellow"].contains(&state.as_str()));
    }

    // --- E5.4 skip-state configuration ---

    #[test]
    fn e5_4_no_skip_config_means_no_enforcement() {
        // Traces to: FR-E5.4 — without sequential_transition, non-sequential edges are allowed.
        let sm = StateMachineBuilder::new("a")
            .transition("a", "leap", "c")
            .build()
            .unwrap();
        sm.send("leap").unwrap();
        assert_eq!(sm.current(), "c");
    }

    fn skip_light() -> StateMachine {
        StateMachineBuilder::new("red")
            .sequential_transition("red", "green")
            .sequential_transition("green", "yellow")
            .sequential_transition("yellow", "red")
            .transition("red", "next", "green")
            .transition("green", "next", "yellow")
            .transition("yellow", "next", "red")
            .transition("red", "skip_to_yellow", "yellow")
            .transition("green", "jump_to_red", "red")
            .skip_transition("red", "yellow")
            .unwrap()
            .build()
            .unwrap()
    }

    #[test]
    fn e5_4_sequential_transition_allowed() {
        let sm = skip_light();
        assert_eq!(sm.current(), "red");
        sm.send("next").unwrap();
        assert_eq!(sm.current(), "green");
    }

    #[test]
    fn e5_4_skip_transition_allowed_when_configured() {
        let sm = skip_light();
        sm.send("skip_to_yellow").unwrap();
        assert_eq!(sm.current(), "yellow");
    }

    #[test]
    fn e5_4_skip_transition_rejected_when_not_configured() {
        let sm = skip_light();
        sm.send("next").unwrap();
        assert_eq!(sm.current(), "green");
        let err = sm.send("jump_to_red").unwrap_err();
        assert!(matches!(
            err,
            StateMachineError::SkipTransitionRejected { .. }
        ));
        if let StateMachineError::SkipTransitionRejected { from, to } = &err {
            assert_eq!(from, "green");
            assert_eq!(to, "red");
        }
        assert_eq!(sm.current(), "green");
    }

    #[test]
    fn e5_4_skip_transition_with_guard() {
        let sm = StateMachineBuilder::new("red")
            .sequential_transition("red", "green")
            .sequential_transition("green", "yellow")
            .transition("red", "skip", "yellow")
            .guarded_transition("red", "skip", "yellow", |_, _| false)
            .skip_transition("red", "yellow")
            .unwrap()
            .build()
            .unwrap();
        let err = sm.send("skip").unwrap_err();
        assert!(matches!(err, StateMachineError::GuardRejected { .. }));
        assert_eq!(sm.current(), "red");
    }

    #[test]
    fn e5_4_skip_transition_callbacks_fire() {
        let enter_count = Arc::new(AtomicUsize::new(0));
        let exit_count = Arc::new(AtomicUsize::new(0));
        let ec = enter_count.clone();
        let xc = exit_count.clone();

        let sm = StateMachineBuilder::new("red")
            .sequential_transition("red", "green")
            .sequential_transition("green", "yellow")
            .transition("red", "skip", "yellow")
            .skip_transition("red", "yellow")
            .unwrap()
            .on_exit("red", move |_| {
                xc.fetch_add(1, Ordering::SeqCst);
            })
            .on_enter("yellow", move |_| {
                ec.fetch_add(1, Ordering::SeqCst);
            })
            .build()
            .unwrap();

        sm.send("skip").unwrap();
        assert_eq!(exit_count.load(Ordering::SeqCst), 1);
        assert_eq!(enter_count.load(Ordering::SeqCst), 1);
        assert_eq!(sm.current(), "yellow");
    }

    #[test]
    fn e5_4_skip_transition_requires_sequential_first() {
        let err = StateMachineBuilder::new("red")
            .transition("red", "skip", "yellow")
            .skip_transition("red", "yellow");
        assert!(matches!(err, Err(StateMachineError::BuildError(_))));
    }

    #[test]
    fn e5_4_skip_pair_requires_event_with_skip_target() {
        let err = StateMachineBuilder::new("red")
            .sequential_transition("red", "green")
            .skip_transition("red", "yellow")
            .unwrap()
            .transition("red", "next", "green")
            .build()
            .unwrap_err();
        assert!(matches!(err, StateMachineError::BuildError(_)));
    }

    #[test]
    fn e5_4_skip_via_different_event() {
        let sm = StateMachineBuilder::new("red")
            .sequential_transition("red", "green")
            .sequential_transition("green", "yellow")
            .transition("red", "walk", "green")
            .transition("red", "jump", "yellow")
            .skip_transition("red", "yellow")
            .unwrap()
            .build()
            .unwrap();
        sm.send("jump").unwrap();
        assert_eq!(sm.current(), "yellow");
    }
}
