//! Dependency resolver - Graph-based dependency resolution

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::{toposort, is_cyclic_directed};
use std::collections::HashMap;

use crate::domain::{Skill, SkillId, SkillDependency};
use super::ResolutionError;

/// A dependency resolution graph
#[derive(Debug)]
pub struct ResolutionGraph {
    graph: DiGraph<SkillId, ()>,
    nodes: HashMap<SkillId, NodeIndex>,
}

impl ResolutionGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            nodes: HashMap::new(),
        }
    }
    
    pub fn add_skill(&mut self, skill_id: SkillId) -> NodeIndex {
        let node = self.graph.add_node(skill_id);
        self.nodes.insert(skill_id, node);
        node
    }
    
    pub fn add_dependency(&mut self, from: SkillId, to: SkillId) {
        if let (Some(&from_node), Some(&to_node)) = (self.nodes.get(&from), self.nodes.get(&to)) {
            self.graph.add_edge(from_node, to_node, ());
        }
    }
    
    /// Perform topological sort to get execution order
    pub fn resolve_order(&self) -> Result<Vec<SkillId>, ResolutionError> {
        // Check for cycles
        if is_cyclic_directed(&self.graph) {
            return Err(ResolutionError::CircularDependency("Cycle detected in dependency graph".to_string()));
        }
        
        // Get topological order
        let order = toposort(&self.graph, None)
            .map_err(|_| ResolutionError::CircularDependency("Cycle detected during sort".to_string()))?;
        
        let result: Vec<SkillId> = order.iter()
            .map(|node| self.graph[*node])
            .collect();
        
        Ok(result)
    }
}

impl Default for ResolutionGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolver for skill dependencies
#[derive(Debug)]
pub struct DependencyResolver;

impl DependencyResolver {
    pub fn new() -> Self {
        Self
    }
    
    /// Build a resolution graph from a list of skills
    pub fn build_graph(&self, skills: &[Skill]) -> ResolutionGraph {
        let mut graph = ResolutionGraph::new();
        
        // Add all skills to graph
        for skill in skills {
            graph.add_skill(skill.id);
        }
        
        // Add dependencies as edges
        for skill in skills {
            for dep in skill.dependencies() {
                // Find the dependency skill by name
                if let Some(dep_skill) = skills.iter().find(|s| s.name() == dep.name) {
                    graph.add_dependency(skill.id, dep_skill.id);
                }
            }
        }
        
        graph
    }
    
    /// Get the execution order for skills
    pub fn get_execution_order(&self, skills: &[Skill]) -> Result<Vec<SkillId>, ResolutionError> {
        let graph = self.build_graph(skills);
        graph.resolve_order()
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}
