use std::collections::{HashMap, HashSet, VecDeque};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WpState {
    Planned,
    Doing,
    Review,
    Done,
    Blocked,
}

impl WpState {
    pub fn can_transition_to(&self, target: WpState) -> bool {
        matches!(
            (self, target),
            (WpState::Planned, WpState::Doing)
                | (WpState::Planned, WpState::Blocked)
                | (WpState::Doing, WpState::Review)
                | (WpState::Doing, WpState::Blocked)
                | (WpState::Review, WpState::Done)
                | (WpState::Review, WpState::Doing)
                | (WpState::Blocked, WpState::Planned)
                | (WpState::Blocked, WpState::Doing)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrState {
    Open,
    Review,
    ChangesRequested,
    Approved,
    Merged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPackage {
    pub id: i64,
    pub feature_id: i64,
    pub title: String,
    pub state: WpState,
    pub sequence: i32,
    pub file_scope: Vec<String>,
    pub acceptance_criteria: String,
    pub agent_id: Option<String>,
    pub pr_url: Option<String>,
    pub pr_state: Option<PrState>,
    pub worktree_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WorkPackage {
    pub fn new(feature_id: i64, title: &str, sequence: i32, acceptance_criteria: &str) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            feature_id,
            title: title.to_string(),
            state: WpState::Planned,
            sequence,
            file_scope: Vec::new(),
            acceptance_criteria: acceptance_criteria.to_string(),
            agent_id: None,
            pr_url: None,
            pr_state: None,
            worktree_path: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn transition(&mut self, target: WpState) -> Result<(), DomainError> {
        if !self.state.can_transition_to(target) {
            return Err(DomainError::InvalidTransition {
                from: format!("{:?}", self.state),
                to: format!("{:?}", target),
                reason: "transition not allowed".into(),
            });
        }
        self.state = target;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn has_file_overlap(&self, other: &WorkPackage) -> Vec<String> {
        let set: HashSet<&String> = self.file_scope.iter().collect();
        other
            .file_scope
            .iter()
            .filter(|f| set.contains(f))
            .cloned()
            .collect()
    }
}

// --- Dependency Graph ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyType {
    Explicit,
    FileOverlap,
    Data,
}

#[derive(Debug, Clone)]
pub struct WpDependency {
    pub wp_id: i64,
    pub depends_on: i64,
    pub dep_type: DependencyType,
}

#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    edges: HashMap<i64, Vec<WpDependency>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_edge(&mut self, dep: WpDependency) {
        self.edges.entry(dep.wp_id).or_default().push(dep);
    }

    pub fn add_file_overlap_edges(&mut self, work_packages: &[WorkPackage]) {
        for i in 0..work_packages.len() {
            for j in (i + 1)..work_packages.len() {
                let overlap = work_packages[i].has_file_overlap(&work_packages[j]);
                if !overlap.is_empty() {
                    // Lower sequence depends on nothing extra; higher sequence depends on lower
                    let (earlier, later) = if work_packages[i].sequence <= work_packages[j].sequence
                    {
                        (&work_packages[i], &work_packages[j])
                    } else {
                        (&work_packages[j], &work_packages[i])
                    };
                    self.add_edge(WpDependency {
                        wp_id: later.id,
                        depends_on: earlier.id,
                        dep_type: DependencyType::FileOverlap,
                    });
                }
            }
        }
    }

    pub fn ready_wps(&self, done: &HashSet<i64>) -> Vec<i64> {
        let all_ids: HashSet<i64> = self.all_node_ids();
        all_ids
            .into_iter()
            .filter(|id| !done.contains(id))
            .filter(|id| {
                self.edges
                    .get(id)
                    .map(|deps| deps.iter().all(|d| done.contains(&d.depends_on)))
                    .unwrap_or(true)
            })
            .collect()
    }

    pub fn has_cycle(&self) -> bool {
        self.execution_order().is_err()
    }

    /// Kahn's algorithm — returns layers of WP ids that can execute in parallel.
    pub fn execution_order(&self) -> Result<Vec<Vec<i64>>, DomainError> {
        let all_ids = self.all_node_ids();
        let mut in_degree: HashMap<i64, usize> = all_ids.iter().map(|&id| (id, 0)).collect();

        for deps in self.edges.values() {
            for dep in deps {
                // dep.wp_id depends on dep.depends_on, so wp_id has an incoming edge
                *in_degree.entry(dep.wp_id).or_default() += 1;
            }
        }

        // Reverse adjacency: depends_on -> list of wp_ids that depend on it
        let mut reverse: HashMap<i64, Vec<i64>> = HashMap::new();
        for deps in self.edges.values() {
            for dep in deps {
                reverse.entry(dep.depends_on).or_default().push(dep.wp_id);
            }
        }

        let mut queue: VecDeque<i64> = in_degree
            .iter()
            .filter(|&(_, deg)| *deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut layers: Vec<Vec<i64>> = Vec::new();
        let mut processed = 0usize;

        while !queue.is_empty() {
            let mut layer: Vec<i64> = queue.drain(..).collect();
            layer.sort();
            processed += layer.len();

            for &id in &layer {
                if let Some(dependents) = reverse.get(&id) {
                    for &dep_id in dependents {
                        let deg = in_degree.get_mut(&dep_id).unwrap();
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(dep_id);
                        }
                    }
                }
            }

            layers.push(layer);
        }

        if processed != all_ids.len() {
            return Err(DomainError::InvalidTransition {
                from: "graph".into(),
                to: "execution_order".into(),
                reason: "cycle detected in dependency graph".into(),
            });
        }

        Ok(layers)
    }

    fn all_node_ids(&self) -> HashSet<i64> {
        let mut ids = HashSet::new();
        for (wp_id, deps) in &self.edges {
            ids.insert(*wp_id);
            for d in deps {
                ids.insert(d.depends_on);
            }
        }
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- WP state machine tests ---
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

    // --- Dependency graph tests ---
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
        // b (seq 2) should depend on a (seq 1)
        assert!(g.edges.get(&2).unwrap().iter().any(|d| d.depends_on == 1));
    }
}
