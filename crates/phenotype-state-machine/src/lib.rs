//! # Phenotype State Machine

use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct StateMachine<S, E> {
    pub state: S,
    pub transitions: Vec<(S, E, S)>,
}

impl<S: Clone, E: Clone> StateMachine<S, E> {
    pub fn new(initial: S) -> Self {
        Self { state: initial, transitions: vec![] }
    }

    pub fn transition(&mut self, event: E, target: S) {
        self.transitions.push((self.state.clone(), event, target.clone()));
        self.state = target;
    }
}
