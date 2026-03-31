use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use super::WorkPackage;
use crate::error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyType {
    Explicit,
    FileOverlap,
    Data,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
                    .map(|deps| deps.iter().all(|dep| done.contains(&dep.depends_on)))
                    .unwrap_or(true)
            })
            .collect()
    }

    pub fn has_cycle(&self) -> bool {
        self.execution_order().is_err()
    }

    pub fn execution_order(&self) -> Result<Vec<Vec<i64>>, DomainError> {
        let all_ids = self.all_node_ids();
        let mut in_degree: HashMap<i64, usize> = all_ids.iter().map(|&id| (id, 0)).collect();

        for deps in self.edges.values() {
            for dep in deps {
                *in_degree.entry(dep.wp_id).or_default() += 1;
            }
        }

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
            for dep in deps {
                ids.insert(dep.depends_on);
            }
        }
        ids
    }
}
