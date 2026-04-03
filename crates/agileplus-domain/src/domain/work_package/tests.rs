use std::collections::HashSet;

use super::{DependencyGraph, DependencyType, WorkPackage, WpDependency, WpState};

#[test]
fn wp_planned_to_doing() {
    let mut wp = WorkPackage::new(1, "test", 1, "criteria");
    wp.transition(WpState::Doing).unwrap();
    assert_eq!(wp.state, WpState::Doing);
}

#[test]
fn wp_doing_to_review() {
    let mut wp = WorkPackage::new(1, "t", 1, "c");
    wp.transition(WpState::Doing).unwrap();
    wp.transition(WpState::Review).unwrap();
    assert_eq!(wp.state, WpState::Review);
}

#[test]
fn wp_review_to_done() {
    let mut wp = WorkPackage::new(1, "t", 1, "c");
    wp.transition(WpState::Doing).unwrap();
    wp.transition(WpState::Review).unwrap();
    wp.transition(WpState::Done).unwrap();
    assert_eq!(wp.state, WpState::Done);
}

#[test]
fn wp_invalid_planned_to_done() {
    let mut wp = WorkPackage::new(1, "t", 1, "c");
    assert!(wp.transition(WpState::Done).is_err());
}

#[test]
fn wp_blocked_and_back() {
    let mut wp = WorkPackage::new(1, "t", 1, "c");
    wp.transition(WpState::Blocked).unwrap();
    wp.transition(WpState::Planned).unwrap();
    assert_eq!(wp.state, WpState::Planned);
}

#[test]
fn wp_file_overlap() {
    let mut a = WorkPackage::new(1, "a", 1, "c");
    a.file_scope = vec!["src/main.rs".into(), "src/lib.rs".into()];
    let mut b = WorkPackage::new(1, "b", 2, "c");
    b.file_scope = vec!["src/lib.rs".into(), "src/other.rs".into()];
    assert_eq!(a.has_file_overlap(&b), vec!["src/lib.rs".to_string()]);
}

#[test]
fn graph_empty() {
    let g = DependencyGraph::new();
    assert!(!g.has_cycle());
}

#[test]
fn graph_linear_order() {
    let mut g = DependencyGraph::new();
    g.add_edge(WpDependency {
        wp_id: 2,
        depends_on: 1,
        dep_type: DependencyType::Explicit,
    });
    g.add_edge(WpDependency {
        wp_id: 3,
        depends_on: 2,
        dep_type: DependencyType::Explicit,
    });
    let order = g.execution_order().unwrap();
    assert_eq!(order, vec![vec![1], vec![2], vec![3]]);
}

#[test]
fn graph_parallel() {
    let mut g = DependencyGraph::new();
    g.add_edge(WpDependency {
        wp_id: 2,
        depends_on: 1,
        dep_type: DependencyType::Explicit,
    });
    g.add_edge(WpDependency {
        wp_id: 3,
        depends_on: 1,
        dep_type: DependencyType::Explicit,
    });
    let order = g.execution_order().unwrap();
    assert_eq!(order.len(), 2);
    assert_eq!(order[0], vec![1]);
    assert!(order[1].contains(&2) && order[1].contains(&3));
}

#[test]
fn graph_cycle_detected() {
    let mut g = DependencyGraph::new();
    g.add_edge(WpDependency {
        wp_id: 1,
        depends_on: 2,
        dep_type: DependencyType::Explicit,
    });
    g.add_edge(WpDependency {
        wp_id: 2,
        depends_on: 1,
        dep_type: DependencyType::Explicit,
    });
    assert!(g.has_cycle());
}

#[test]
fn graph_ready_wps() {
    let mut g = DependencyGraph::new();
    g.add_edge(WpDependency {
        wp_id: 2,
        depends_on: 1,
        dep_type: DependencyType::Explicit,
    });
    g.add_edge(WpDependency {
        wp_id: 3,
        depends_on: 1,
        dep_type: DependencyType::Explicit,
    });
    let done = HashSet::new();
    let mut ready = g.ready_wps(&done);
    ready.sort();
    assert_eq!(ready, vec![1]);
    let done: HashSet<i64> = [1].into();
    let mut ready = g.ready_wps(&done);
    ready.sort();
    assert_eq!(ready, vec![2, 3]);
}

#[test]
fn graph_file_overlap_edges() {
    let mut a = WorkPackage::new(1, "a", 1, "c");
    a.id = 1;
    a.file_scope = vec!["f.rs".into()];
    let mut b = WorkPackage::new(1, "b", 2, "c");
    b.id = 2;
    b.file_scope = vec!["f.rs".into()];
    let mut g = DependencyGraph::new();
    g.add_file_overlap_edges(&[a, b]);
    assert_eq!(g.execution_order().unwrap(), vec![vec![1], vec![2]]);
}
