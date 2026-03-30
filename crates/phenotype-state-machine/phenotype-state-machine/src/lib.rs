//! phenotype-state-machine
//!
//! A strongly-typed state machine implementation with:
//! - Type-safe state and event definitions
//! - Guard conditions for transition validation
//! - Optional actions on state transitions
//! - Immutable state history for auditability

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Errors that can occur during state machine operations.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum StateMachineError {
    #[error("Invalid transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },

    #[error("Guard condition failed: {reason}")]
    GuardConditionFailed { reason: String },

    #[error("State machine locked")]
    Locked,

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

pub type Result<T> = std::result::Result<T, StateMachineError>;

/// A guard condition function that validates whether a transition is allowed.
///
/// Returns true if the transition is allowed, false otherwise.
pub type GuardFn<C> = Arc<dyn Fn(&C) -> bool + Send + Sync>;

/// An action function that runs on successful state transition.
pub type ActionFn<C> = Arc<dyn Fn(&mut C) + Send + Sync>;

/// Represents a valid transition between two states with optional guard and action.
#[derive(Clone)]
pub struct Transition<S, C>
where
    S: Clone + PartialEq + Debug + Serialize + for<'de> Deserialize<'de>,
    C: Clone + Serialize + for<'de> Deserialize<'de>,
{
    pub from: S,
    pub to: S,
    pub guard: Option<GuardFn<C>>,
    pub action: Option<ActionFn<C>>,
}

impl<S, C> Transition<S, C>
where
    S: Clone + PartialEq + Debug + Serialize + for<'de> Deserialize<'de>,
    C: Clone + Serialize + for<'de> Deserialize<'de>,
{
    /// Creates a new transition.
    pub fn new(from: S, to: S) -> Self {
        Self {
            from,
            to,
            guard: None,
            action: None,
        }
    }

    /// Adds a guard condition to this transition.
    pub fn with_guard<F>(mut self, guard: F) -> Self
    where
        F: Fn(&C) -> bool + Send + Sync + 'static,
    {
        self.guard = Some(Arc::new(guard));
        self
    }

    /// Adds an action to this transition.
    pub fn with_action<F>(mut self, action: F) -> Self
    where
        F: Fn(&mut C) + Send + Sync + 'static,
    {
        self.action = Some(Arc::new(action));
        self
    }
}

/// A state machine that enforces valid state transitions.
pub struct StateMachine<S, C>
where
    S: Clone + PartialEq + Debug + Serialize + for<'de> Deserialize<'de>,
    C: Clone + Serialize + for<'de> Deserialize<'de>,
{
    current_state: Arc<RwLock<S>>,
    context: Arc<RwLock<C>>,
    transitions: Vec<Transition<S, C>>,
    history: Arc<RwLock<Vec<S>>>,
}

impl<S, C> StateMachine<S, C>
where
    S: Clone + PartialEq + Debug + Serialize + for<'de> Deserialize<'de>,
    C: Clone + Serialize + for<'de> Deserialize<'de>,
{
    /// Creates a new state machine with an initial state and context.
    pub fn new(initial_state: S, initial_context: C) -> Self {
        let history = vec![initial_state.clone()];
        Self {
            current_state: Arc::new(RwLock::new(initial_state)),
            context: Arc::new(RwLock::new(initial_context)),
            transitions: Vec::new(),
            history: Arc::new(RwLock::new(history)),
        }
    }

    /// Adds a transition to the state machine.
    pub fn add_transition(&mut self, transition: Transition<S, C>) {
        self.transitions.push(transition);
    }

    /// Attempts to transition to a new state.
    ///
    /// Returns an error if:
    /// - No valid transition exists from current state to target state
    /// - A guard condition fails
    pub fn transition_to(&self, target_state: S) -> Result<()> {
        let current = self.current_state.read()
            .map_err(|_| StateMachineError::Locked)?
            .clone();

        // Find a matching transition
        let transition = self.transitions
            .iter()
            .find(|t| t.from == current && t.to == target_state)
            .ok_or_else(|| StateMachineError::InvalidTransition {
                from: format!("{:?}", current),
                to: format!("{:?}", target_state),
            })?;

        // Check guard condition
        let mut context = self.context.write()
            .map_err(|_| StateMachineError::Locked)?;

        if let Some(guard) = &transition.guard {
            if !guard(&context) {
                return Err(StateMachineError::GuardConditionFailed {
                    reason: "Guard returned false".to_string(),
                });
            }
        }

        // Execute action if present
        if let Some(action) = &transition.action {
            action(&mut context);
        }

        // Update state
        *self.current_state.write()
            .map_err(|_| StateMachineError::Locked)? = target_state.clone();

        // Record in history
        self.history.write()
            .map_err(|_| StateMachineError::Locked)?
            .push(target_state);

        Ok(())
    }

    /// Gets the current state.
    pub fn current(&self) -> Result<S> {
        self.current_state.read()
            .map_err(|_| StateMachineError::Locked)
            .map(|s| s.clone())
    }

    /// Gets the context.
    pub fn context(&self) -> Result<C> {
        self.context.read()
            .map_err(|_| StateMachineError::Locked)
            .map(|c| c.clone())
    }

    /// Gets the state history.
    pub fn history(&self) -> Result<Vec<S>> {
        self.history.read()
            .map_err(|_| StateMachineError::Locked)
            .map(|h| h.clone())
    }

    /// Checks if a transition from current state to target state is valid.
    pub fn can_transition_to(&self, target_state: &S) -> Result<bool> {
        let current = self.current()?;

        let transition = self.transitions
            .iter()
            .find(|t| &t.from == &current && &t.to == target_state);

        match transition {
            None => Ok(false),
            Some(t) => {
                if let Some(guard) = &t.guard {
                    let context = self.context()?;
                    Ok(guard(&context))
                } else {
                    Ok(true)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    enum OrderState {
        Pending,
        Confirmed,
        Shipped,
        Delivered,
        Cancelled,
    }

    #[derive(Clone, Serialize, Deserialize)]
    struct OrderContext {
        order_id: String,
        amount: f64,
        notes: String,
    }

    #[test]
    fn test_create_state_machine() {
        let ctx = OrderContext {
            order_id: "123".to_string(),
            amount: 100.0,
            notes: String::new(),
        };
        let sm = StateMachine::new(OrderState::Pending, ctx);

        let current = sm.current().unwrap();
        assert_eq!(current, OrderState::Pending);
    }

    #[test]
    fn test_valid_transition() {
        let ctx = OrderContext {
            order_id: "123".to_string(),
            amount: 100.0,
            notes: String::new(),
        };
        let mut sm = StateMachine::new(OrderState::Pending, ctx);

        let t = Transition::new(OrderState::Pending, OrderState::Confirmed);
        sm.add_transition(t);

        sm.transition_to(OrderState::Confirmed).unwrap();
        assert_eq!(sm.current().unwrap(), OrderState::Confirmed);
    }

    #[test]
    fn test_invalid_transition() {
        let ctx = OrderContext {
            order_id: "123".to_string(),
            amount: 100.0,
            notes: String::new(),
        };
        let sm = StateMachine::new(OrderState::Pending, ctx);

        let result = sm.transition_to(OrderState::Delivered);
        assert!(result.is_err());
    }

    #[test]
    fn test_guard_success() {
        let ctx = OrderContext {
            order_id: "123".to_string(),
            amount: 100.0,
            notes: String::new(),
        };
        let mut sm = StateMachine::new(OrderState::Pending, ctx);

        let t = Transition::new(OrderState::Pending, OrderState::Confirmed)
            .with_guard(|ctx: &OrderContext| ctx.amount > 0.0);
        sm.add_transition(t);

        sm.transition_to(OrderState::Confirmed).unwrap();
        assert_eq!(sm.current().unwrap(), OrderState::Confirmed);
    }

    #[test]
    fn test_guard_failure() {
        let ctx = OrderContext {
            order_id: "123".to_string(),
            amount: 0.0,  // Fails guard
            notes: String::new(),
        };
        let mut sm = StateMachine::new(OrderState::Pending, ctx);

        let t = Transition::new(OrderState::Pending, OrderState::Confirmed)
            .with_guard(|ctx: &OrderContext| ctx.amount > 0.0);
        sm.add_transition(t);

        let result = sm.transition_to(OrderState::Confirmed);
        assert!(result.is_err());
    }

    #[test]
    fn test_action_execution() {
        let ctx = OrderContext {
            order_id: "123".to_string(),
            amount: 100.0,
            notes: String::new(),
        };
        let mut sm = StateMachine::new(OrderState::Pending, ctx);

        let t = Transition::new(OrderState::Pending, OrderState::Confirmed)
            .with_action(|ctx: &mut OrderContext| {
                ctx.notes.push_str("Order confirmed");
            });
        sm.add_transition(t);

        sm.transition_to(OrderState::Confirmed).unwrap();
        let ctx = sm.context().unwrap();
        assert_eq!(ctx.notes, "Order confirmed");
    }

    #[test]
    fn test_state_history() {
        let ctx = OrderContext {
            order_id: "123".to_string(),
            amount: 100.0,
            notes: String::new(),
        };
        let mut sm = StateMachine::new(OrderState::Pending, ctx);

        sm.add_transition(Transition::new(OrderState::Pending, OrderState::Confirmed));
        sm.add_transition(Transition::new(OrderState::Confirmed, OrderState::Shipped));

        sm.transition_to(OrderState::Confirmed).unwrap();
        sm.transition_to(OrderState::Shipped).unwrap();

        let history = sm.history().unwrap();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0], OrderState::Pending);
        assert_eq!(history[1], OrderState::Confirmed);
        assert_eq!(history[2], OrderState::Shipped);
    }

    #[test]
    fn test_can_transition_to() {
        let ctx = OrderContext {
            order_id: "123".to_string(),
            amount: 100.0,
            notes: String::new(),
        };
        let mut sm = StateMachine::new(OrderState::Pending, ctx);

        sm.add_transition(Transition::new(OrderState::Pending, OrderState::Confirmed));
        sm.add_transition(Transition::new(OrderState::Pending, OrderState::Cancelled));

        assert!(sm.can_transition_to(&OrderState::Confirmed).unwrap());
        assert!(sm.can_transition_to(&OrderState::Cancelled).unwrap());
        assert!(!sm.can_transition_to(&OrderState::Shipped).unwrap());
    }
}
