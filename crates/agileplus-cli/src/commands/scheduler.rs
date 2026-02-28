//! Dependency-aware scheduler for work package execution.
//!
//! Implements Kahn's topological sort to determine execution waves and
//! runtime "next ready" queries.
//! Traceability: FR-039 / WP12-T068

use std::collections::{HashMap, HashSet, VecDeque};

use agileplus_domain::domain::work_package::{WpDependency, WpState};

/// A wave of work packages that can execute in parallel.
#[derive(Debug, Clone)]
pub struct ExecutionWave {
    pub wave_number: u32,
    pub wp_ids: Vec<i64>,
}

/// Scheduler computes execution order from WP dependency graph.
#[derive(Debug)]
pub struct Scheduler {
    /// All WP IDs with their states.
    pub wp_states: HashMap<i64, WpState>,
    /// Dependencies: (wp_id, depends_on)
    pub deps: Vec<WpDependency>,
}

impl Scheduler {
    pub fn new(wp_states: HashMap<i64, WpState>, deps: Vec<WpDependency>) -> Self {
        Self { wp_states, deps }
    }

    /// Compute execution waves using Kahn's algorithm.
    ///
    /// Returns `Err` if the dependency graph contains a cycle.
    pub fn execution_plan(&self) -> Result<Vec<ExecutionWave>, String> {
        let all_ids: HashSet<i64> = self.wp_states.keys().copied().collect();
        let mut in_degree: HashMap<i64, usize> = all_ids.iter().map(|&id| (id, 0)).collect();

        for dep in &self.deps {
            if all_ids.contains(&dep.wp_id) && all_ids.contains(&dep.depends_on) {
                *in_degree.entry(dep.wp_id).or_default() += 1;
            }
        }

        // Reverse adjacency: depends_on -> list of wp_ids that depend on it
        let mut reverse: HashMap<i64, Vec<i64>> = HashMap::new();
        for dep in &self.deps {
            if all_ids.contains(&dep.wp_id) && all_ids.contains(&dep.depends_on) {
                reverse.entry(dep.depends_on).or_default().push(dep.wp_id);
            }
        }

        let mut queue: VecDeque<i64> = in_degree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut waves: Vec<ExecutionWave> = Vec::new();
        let mut processed = 0usize;
        let mut wave_number = 0u32;

        while !queue.is_empty() {
            let mut wave_ids: Vec<i64> = queue.drain(..).collect();
            wave_ids.sort();
            processed += wave_ids.len();

            for &id in &wave_ids {
                if let Some(dependents) = reverse.get(&id) {
                    for &dep_id in dependents {
                        if let Some(deg) = in_degree.get_mut(&dep_id) {
                            *deg -= 1;
                            if *deg == 0 {
                                queue.push_back(dep_id);
                            }
                        }
                    }
                }
            }

            waves.push(ExecutionWave { wave_number, wp_ids: wave_ids });
            wave_number += 1;
        }

        if processed != all_ids.len() {
            return Err("cycle detected in dependency graph".to_string());
        }

        Ok(waves)
    }

    /// Return WPs whose dependencies are all completed and whose state is Planned.
    pub fn next_ready(&self, completed: &HashSet<i64>) -> Vec<i64> {
        let mut ready = Vec::new();
        for (&wp_id, &state) in &self.wp_states {
            if completed.contains(&wp_id) {
                continue;
            }
            if state != WpState::Planned {
                continue;
            }
            let deps_met = self
                .deps
                .iter()
                .filter(|d| d.wp_id == wp_id)
                .all(|d| completed.contains(&d.depends_on));
            if deps_met {
                ready.push(wp_id);
            }
        }
        ready.sort();
        ready
    }

    /// If the WP has unmet dependencies, return the list of blocking WP IDs.
    pub fn is_blocked(&self, wp_id: i64, completed: &HashSet<i64>) -> Option<Vec<i64>> {
        let blocking: Vec<i64> = self
            .deps
            .iter()
            .filter(|d| d.wp_id == wp_id && !completed.contains(&d.depends_on))
            .map(|d| d.depends_on)
            .collect();
        if blocking.is_empty() {
            None
        } else {
            Some(blocking)
        }
    }

    /// Check if the dependency graph contains a cycle.
    pub fn has_cycle(&self) -> bool {
        self.execution_plan().is_err()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::domain::work_package::DependencyType;

    fn dep(wp_id: i64, depends_on: i64) -> WpDependency {
        WpDependency { wp_id, depends_on, dep_type: DependencyType::Explicit }
    }

    fn states(ids: &[i64]) -> HashMap<i64, WpState> {
        ids.iter().map(|&id| (id, WpState::Planned)).collect()
    }

    #[test]
    fn single_wp_no_deps() {
        let s = Scheduler::new(states(&[1]), vec![]);
        let plan = s.execution_plan().unwrap();
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].wp_ids, vec![1]);
    }

    #[test]
    fn linear_three_wps() {
        let s = Scheduler::new(states(&[1, 2, 3]), vec![dep(2, 1), dep(3, 2)]);
        let plan = s.execution_plan().unwrap();
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0].wp_ids, vec![1]);
        assert_eq!(plan[1].wp_ids, vec![2]);
        assert_eq!(plan[2].wp_ids, vec![3]);
    }

    #[test]
    fn diamond_dependency_pattern() {
        // A -> B, A -> C, B -> D, C -> D
        // Wave 0: A (id=1), Wave 1: B(2), C(3), Wave 2: D(4)
        let s = Scheduler::new(
            states(&[1, 2, 3, 4]),
            vec![dep(2, 1), dep(3, 1), dep(4, 2), dep(4, 3)],
        );
        let plan = s.execution_plan().unwrap();
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0].wp_ids, vec![1]);
        let mut wave1 = plan[1].wp_ids.clone();
        wave1.sort();
        assert_eq!(wave1, vec![2, 3]);
        assert_eq!(plan[2].wp_ids, vec![4]);
    }

    #[test]
    fn cycle_detected() {
        let s = Scheduler::new(states(&[1, 2]), vec![dep(1, 2), dep(2, 1)]);
        assert!(s.has_cycle());
        assert!(s.execution_plan().is_err());
    }

    #[test]
    fn next_ready_basic() {
        let s = Scheduler::new(states(&[1, 2, 3]), vec![dep(2, 1), dep(3, 1)]);
        let ready = s.next_ready(&HashSet::new());
        assert_eq!(ready, vec![1]);

        let done: HashSet<i64> = [1].into_iter().collect();
        let mut ready2 = s.next_ready(&done);
        ready2.sort();
        assert_eq!(ready2, vec![2, 3]);
    }

    #[test]
    fn is_blocked_reports_blockers() {
        let s = Scheduler::new(states(&[1, 2]), vec![dep(2, 1)]);
        let blockers = s.is_blocked(2, &HashSet::new());
        assert_eq!(blockers, Some(vec![1]));

        let done: HashSet<i64> = [1].into_iter().collect();
        assert_eq!(s.is_blocked(2, &done), None);
    }

    #[test]
    fn next_ready_skips_non_planned() {
        let mut wp_states = HashMap::new();
        wp_states.insert(1i64, WpState::Done);
        wp_states.insert(2i64, WpState::Planned);
        let s = Scheduler::new(wp_states, vec![dep(2, 1)]);
        // 1 is done so not returned; 2 is planned with dep on done 1
        let done: HashSet<i64> = [1].into_iter().collect();
        let ready = s.next_ready(&done);
        assert_eq!(ready, vec![2]);
    }
}
