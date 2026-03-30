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

    #[error("skip transition from '{from}' to '{to}' is not allowed (not in skip_states)")]
    SkipTransitionRejected { from: String, to: String },

    #[error("unknown state: '{0}'")]
    UnknownState(String),

    #[error("builder error: {0}")]
    BuildError(String),
}

/// Result type for state machine operations.
pub type Result<T> = std::result::Result<T, StateMachineError>;

/// Guard function type: takes (from_state, event) and returns whether transition is allowed.
type GuardFn = Arc<dyn Fn(&str, &str) -> bool + Send + Sync>;

/// A transition definition: (from_state, event) -> to_state with optional guard.
#[derive(Clone)]
struct Transition {
    to: String,
    guard: Option<GuardFn>,
}

impl fmt::Debug for Transition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Transition")
            .field("to", &self.to)
            .field("has_guard", &self.guard.is_some())
            .finish()
    }
}

/// A generic finite state machine.
///
/// Thread-safe via internal `RwLock`. States and events are string-based
/// for maximum flexibility.
pub struct StateMachine {
    current: RwLock<String>,
    transitions: HashMap<(String, String), Transition>,
    on_enter: HashMap<String, StateCallbacks>,
    on_exit: HashMap<String, StateCallbacks>,
    /// Map of state → its expected "next" state in the sequential chain.
    /// Used to enforce forward-only transitions unless the (from, to) pair
    /// is explicitly listed in `skip_states`.
    sequential_next: HashMap<String, String>,
    /// Explicitly allowed skip-state pairs: (from, to) that bypass the
    /// sequential chain. Guards and action callbacks still run for these.
    skip_states: HashSet<(String, String)>,
}

impl StateMachine {
    /// Get the current state.
    pub fn current(&self) -> String {
        self.current.read().unwrap().clone()
    }

    /// Send an event to the state machine, potentially triggering a transition.
    ///
    /// If `skip_states` is configured, any transition whose target is not the
    /// expected `sequential_next` of the current state is rejected unless the
    /// `(from, to)` pair is explicitly registered in `skip_states`.
    pub fn send(&self, event: &str) -> Result<String> {
        let mut current = self.current.write().unwrap();
        let key = (current.clone(), event.to_string());

        let transition =
            self.transitions
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

        // E5.4: skip-state validation — if sequential_next is set, enforce it.
        let new_state = transition.to.clone();
        if let Some(expected_next) = self.sequential_next.get(current.as_str()) {
            if &new_state != expected_next {
                // Not the normal sequential step — check skip_states allowlist.
                if !self.skip_states.contains(&(current.clone(), new_state.clone())) {
                    return Err(StateMachineError::SkipTransitionRejected {
                        from: current.clone(),
                        to: new_state.clone(),
                    });
                }
            }
        }

        // Fire on_exit callbacks for current state.
        if let Some(cbs) = self.on_exit.get(current.as_str()) {
            for cb in cbs {
                cb(&current);
            }
        }

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
            .field("sequential_next", &self.sequential_next.len())
            .field("skip_states", &self.skip_states.len())
            .finish()
    }
}

// Send + Sync are safe because internal state is behind RwLock.
unsafe impl Send for StateMachine {}
unsafe impl Sync for StateMachine {}

/// Callback list type for state enter/exit hooks.
type StateCallbacks = Vec<Arc<dyn Fn(&str) + Send + Sync>>;

/// Builder for constructing a [`StateMachine`].
pub struct StateMachineBuilder {
    initial: String,
    transitions: HashMap<(String, String), Transition>,
    on_enter: HashMap<String, StateCallbacks>,
    on_exit: HashMap<String, StateCallbacks>,
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
    ///
    /// After calling this, any transition from `from` to a state other than
    /// `next` will be rejected unless that `(from, to)` pair is registered via
    /// [`skip_transition`](Self::skip_transition).
    ///
    /// Multiple calls for the same `from` replace the previous declaration.
    pub fn sequential_transition(mut self, from: &str, next: &str) -> Self {
        self.sequential_next
            .insert(from.to_string(), next.to_string());
        self
    }

    /// Register an allowed skip-state pair: `(from, to)` bypasses the
    /// `sequential_transition` chain but still runs guards and action callbacks.
    ///
    /// A skip pair must be registered **before** calling [`build`](Self::build).
    /// Returns `Err` if `from` has no `sequential_next` configured.
    pub fn skip_transition(mut self, from: &str, to: &str) -> Result<Self> {
        if !self.sequential_next.contains_key(from) {
            return Err(StateMachineError::BuildError(format!(
                "skip_transition: state '{from}' has no sequential_next configured"
            )));
        }
        self.skip_states
            .insert((from.to_string(), to.to_string()));
        Ok(self)
    }

    /// Build the state machine.
    pub fn build(self) -> Result<StateMachine> {
        if self.initial.is_empty() {
            return Err(StateMachineError::BuildError(
                "initial state cannot be empty".into(),
            ));
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
            // on_enter / on_exit contain dyn Fn — skip them
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
        // Should be in one of the 3 states
        let state = sm.current();
        assert!(["red", "green", "yellow"].contains(&state.as_str()));
    }

    // -------------------------------------------------------------------------
    // E5.4 — Skip-State Configuration
    // -------------------------------------------------------------------------

    /// Traffic light with skip-state configuration:
    ///   red → green (normal) or red → yellow (skip green)
    ///   green → yellow (normal)
    ///   yellow → red (normal)
    fn skip_light() -> StateMachine {
        StateMachineBuilder::new("red")
            // Sequential chain:
            .sequential_transition("red", "green")
            .sequential_transition("green", "yellow")
            .sequential_transition("yellow", "red")
            // Normal transitions:
            .transition("red", "next", "green")
            .transition("green", "next", "yellow")
            .transition("yellow", "next", "red")
            // Skip: red → yellow (bypass green) via a different event
            .transition("red", "skip_to_yellow", "yellow")
            .skip_transition("red", "yellow")
            .unwrap()
            .build()
            .unwrap()
    }

    #[test]
    fn e5_4_sequential_transition_allowed() {
        // Traces to: FR-E5.4-A1
        let sm = skip_light();
        assert_eq!(sm.current(), "red");
        sm.send("next").unwrap(); // red → green (normal sequential)
        assert_eq!(sm.current(), "green");
    }

    #[test]
    fn e5_4_skip_transition_allowed_when_configured() {
        // Traces to: FR-E5.4-A2
        let sm = skip_light();
        assert_eq!(sm.current(), "red");
        sm.send("skip_to_yellow").unwrap(); // red → yellow (skip green)
        assert_eq!(sm.current(), "yellow");
    }

    #[test]
    fn e5_4_skip_transition_rejected_when_not_configured() {
        // Traces to: FR-E5.4-A3
        let sm = skip_light();
        assert_eq!(sm.current(), "red");
        sm.send("next").unwrap(); // red → green
        assert_eq!(sm.current(), "green");
        // green → red is not in skip_states, so it is rejected.
        let err = sm.send("jump_to_red").unwrap_err();
        let err_variant = matches!(
            err,
            StateMachineError::SkipTransitionRejected { .. }
        );
        assert!(err_variant, "expected SkipTransitionRejected, got {err:?}");
        if let StateMachineError::SkipTransitionRejected { from, to } = &err {
            assert_eq!(from, "green");
            assert_eq!(to, "red");
        }
        assert_eq!(sm.current(), "green"); // state unchanged
    }

    #[test]
    fn e5_4_skip_transition_with_guard() {
        // Traces to: FR-E5.4-A4
        let sm = StateMachineBuilder::new("red")
            .sequential_transition("red", "green")
            .sequential_transition("green", "yellow")
            .transition("red", "skip", "yellow")
            .guarded_transition("red", "skip", "yellow", |_, _| false) // guard blocks
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
        // Traces to: FR-E5.4-A5
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
        assert_eq!(sm.current(), "yellow");
        assert_eq!(exit_count.load(Ordering::SeqCst), 1, "on_exit for red fired");
        assert_eq!(enter_count.load(Ordering::SeqCst), 1, "on_enter for yellow fired");
    }

    #[test]
    fn e5_4_skip_transition_requires_sequential_first() {
        // Traces to: FR-E5.4-A6
        let err = StateMachineBuilder::new("a")
            .transition("a", "jump", "c")
            .skip_transition("a", "c") // no sequential_transition for "a"
            .unwrap_err();
        assert!(matches!(err, StateMachineError::BuildError(_)));
        let err_msg = match err {
            StateMachineError::BuildError(s) => s,
            _ => unreachable!(),
        };
        assert!(err_msg.contains("no sequential_next configured"));
    }

    #[test]
    fn e5_4_no_skip_config_means_no_enforcement() {
        // Without sequential_transition, skip_states has no effect (chain not active).
        let sm = StateMachineBuilder::new("red")
            .transition("red", "next", "green")
            .transition("green", "next", "yellow")
            .transition("yellow", "next", "red")
            // No sequential_transition — no chain enforcement
            .build()
            .unwrap();
        // All transitions should work freely
        assert_eq!(sm.send("next").unwrap(), "green");
        assert_eq!(sm.send("next").unwrap(), "yellow");
    }

    #[test]
    fn e5_4_skip_pair_requires_event_with_skip_target() {
        // Traces to: FR-E5.4-A7
        // A skip pair (from, to) is registered. If no event leads from `from` to `to`,
        // send() fails with InvalidTransition (not SkipTransitionRejected) because
        // there is no transition to validate.
        let sm = StateMachineBuilder::new("red")
            .sequential_transition("red", "green")
            .transition("red", "next", "green") // only this event from red
            // skip pair for red→yellow registered but no event leads to yellow
            .skip_transition("red", "yellow").unwrap()
            .build()
            .unwrap();

        // Normal sequential works.
        assert_eq!(sm.send("next").unwrap(), "green");
        assert_eq!(sm.current(), "green");
    }

    #[test]
    fn e5_4_skip_via_different_event() {
        // The skip transition can use a different event from the normal transition.
        let sm = StateMachineBuilder::new("idle")
            .sequential_transition("idle", "step_a")
            .sequential_transition("step_a", "step_b")
            .sequential_transition("step_b", "done")
            .transition("idle", "next", "step_a")
            .transition("idle", "skip_to_b", "step_b") // skip step_a
            .transition("step_a", "next", "step_b")
            .transition("step_b", "next", "done")
            .skip_transition("idle", "step_b")
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(sm.current(), "idle");
        sm.send("skip_to_b").unwrap(); // idle → step_b (skip step_a)
        assert_eq!(sm.current(), "step_b");
        sm.send("next").unwrap(); // step_b → done
        assert_eq!(sm.current(), "done");
    }
}
