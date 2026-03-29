//! Integration tests for state machine (simplified).
use phenotype_state_machine::{StateMachine, Transition};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
enum State {
    Idle,
    Running,
    Done,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Ctx {
    value: i32,
}

#[test]
fn test_create_state_machine() {
    let sm = StateMachine::new(State::Idle, Ctx { value: 0 });
    assert_eq!(sm.current().unwrap(), State::Idle);
}

#[test]
fn test_valid_transition() {
    let mut sm = StateMachine::new(State::Idle, Ctx { value: 0 });
    sm.add_transition(Transition::new(State::Idle, State::Running));
    sm.transition_to(State::Running).unwrap();
    assert_eq!(sm.current().unwrap(), State::Running);
}

#[test]
fn test_invalid_transition() {
    let sm = StateMachine::new(State::Idle, Ctx { value: 0 });
    let result = sm.transition_to(State::Done);
    assert!(result.is_err());
}

#[test]
fn test_state_history() {
    let mut sm = StateMachine::new(State::Idle, Ctx { value: 0 });
    sm.add_transition(Transition::new(State::Idle, State::Running));
    sm.add_transition(Transition::new(State::Running, State::Done));

    sm.transition_to(State::Running).unwrap();
    sm.transition_to(State::Done).unwrap();

    let history = sm.history().unwrap();
    assert_eq!(history.len(), 3);
    assert_eq!(history[0], State::Idle);
    assert_eq!(history[1], State::Running);
    assert_eq!(history[2], State::Done);
}

#[test]
fn test_can_transition_to() {
    let mut sm = StateMachine::new(State::Idle, Ctx { value: 0 });
    sm.add_transition(Transition::new(State::Idle, State::Running));
    assert!(sm.can_transition_to(&State::Running).unwrap());
    assert!(!sm.can_transition_to(&State::Done).unwrap());
}

#[test]
fn test_guard_success() {
    let mut sm = StateMachine::new(State::Idle, Ctx { value: 50 });
    let transition = Transition::new(State::Idle, State::Running)
        .with_guard(|ctx: &Ctx| ctx.value >= 50);
    sm.add_transition(transition);
    sm.transition_to(State::Running).unwrap();
    assert_eq!(sm.current().unwrap(), State::Running);
}

#[test]
fn test_guard_failure() {
    let mut sm = StateMachine::new(State::Idle, Ctx { value: 10 });
    let transition = Transition::new(State::Idle, State::Running)
        .with_guard(|ctx: &Ctx| ctx.value >= 50);
    sm.add_transition(transition);
    let result = sm.transition_to(State::Running);
    assert!(result.is_err());
}

#[test]
fn test_action_execution() {
    let mut sm = StateMachine::new(State::Idle, Ctx { value: 0 });
    let transition = Transition::new(State::Idle, State::Running)
        .with_action(|ctx: &mut Ctx| {
            ctx.value = 100;
        });
    sm.add_transition(transition);

    sm.transition_to(State::Running).unwrap();
    let ctx = sm.context().unwrap();
    assert_eq!(ctx.value, 100);
}

#[test]
fn test_get_context() {
    let sm = StateMachine::new(State::Idle, Ctx { value: 42 });
    let ctx = sm.context().unwrap();
    assert_eq!(ctx.value, 42);
}

#[test]
fn test_concurrent_reads() {
    use std::sync::Arc;
    let sm = Arc::new(StateMachine::new(State::Idle, Ctx { value: 123 }));
    let mut handles = vec![];

    for _ in 0..5 {
        let sm_clone = Arc::clone(&sm);
        let handle = std::thread::spawn(move || {
            let current = sm_clone.current().unwrap();
            let ctx = sm_clone.context().unwrap();
            assert_eq!(current, State::Idle);
            assert_eq!(ctx.value, 123);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_multiple_transitions() {
    let mut sm = StateMachine::new(State::Idle, Ctx { value: 0 });
    sm.add_transition(Transition::new(State::Idle, State::Running));
    sm.add_transition(Transition::new(State::Running, State::Done));

    sm.transition_to(State::Running).unwrap();
    assert_eq!(sm.current().unwrap(), State::Running);

    sm.transition_to(State::Done).unwrap();
    assert_eq!(sm.current().unwrap(), State::Done);
}

#[test]
fn test_guard_with_action() {
    let mut sm = StateMachine::new(State::Idle, Ctx { value: 100 });
    let transition = Transition::new(State::Idle, State::Running)
        .with_guard(|ctx: &Ctx| ctx.value > 50)
        .with_action(|ctx: &mut Ctx| {
            ctx.value = 200;
        });
    sm.add_transition(transition);

    sm.transition_to(State::Running).unwrap();
    let ctx = sm.context().unwrap();
    assert_eq!(ctx.value, 200);
}
