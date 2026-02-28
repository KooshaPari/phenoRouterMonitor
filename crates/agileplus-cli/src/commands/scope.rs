//! File scope detection and overlap graph for WP planning.
//!
//! Parses WP descriptions for file path references and builds a graph of
//! file overlaps between work packages.
//! Traceability: FR-038 / WP12-T067

use std::collections::HashSet;

use agileplus_domain::domain::work_package::WorkPackage;

/// A graph of file overlaps between work packages.
#[derive(Debug, Clone, Default)]
pub struct OverlapGraph {
    /// (wp_a_id, wp_b_id, shared_files)
    pub edges: Vec<(i64, i64, Vec<String>)>,
}

impl OverlapGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return groups of WP IDs that can run in parallel (no file overlaps within group).
    /// Uses greedy graph coloring.
    pub fn parallel_groups(&self, all_ids: &[i64]) -> Vec<Vec<i64>> {
        // Build adjacency set for fast lookup
        let mut conflicting: std::collections::HashMap<i64, HashSet<i64>> =
            std::collections::HashMap::new();
        for (a, b, _) in &self.edges {
            conflicting.entry(*a).or_default().insert(*b);
            conflicting.entry(*b).or_default().insert(*a);
        }

        // Greedy coloring
        let mut colors: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
        let mut max_color = 0usize;

        for &id in all_ids {
            let neighbor_colors: HashSet<usize> = conflicting
                .get(&id)
                .into_iter()
                .flat_map(|s| s.iter())
                .filter_map(|n| colors.get(n))
                .copied()
                .collect();

            let color = (0..).find(|c| !neighbor_colors.contains(c)).unwrap();
            colors.insert(id, color);
            if color > max_color {
                max_color = color;
            }
        }

        let mut groups: Vec<Vec<i64>> = vec![Vec::new(); max_color + 1];
        for (&id, &color) in &colors {
            groups[color].push(id);
        }
        for g in &mut groups {
            g.sort();
        }
        groups
    }
}

/// Detect file paths mentioned in a WP description or acceptance criteria.
///
/// Matches explicit paths (e.g. `src/foo/bar.rs`) and returns a deduplicated,
/// sorted list.
pub fn detect_file_scope(wp_description: &str) -> Vec<String> {
    let mut found: HashSet<String> = HashSet::new();

    for word in wp_description.split_whitespace() {
        // Strip surrounding punctuation (backticks, parens, commas, etc.)
        let w = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '.' && c != '_' && c != '-');
        if w.is_empty() {
            continue;
        }
        // Skip URLs
        if w.contains("://") || w.starts_with("..") {
            continue;
        }
        // Include if it looks like a path (contains slash) or a filename (contains dot with short ext)
        if w.contains('/') {
            found.insert(w.to_string());
        } else if let Some(dot_pos) = w.rfind('.') {
            let ext = &w[dot_pos + 1..];
            if ext.len() >= 1 && ext.len() <= 6 && ext.chars().all(|c| c.is_ascii_alphabetic()) {
                found.insert(w.to_string());
            }
        }
    }

    let mut result: Vec<String> = found.into_iter().collect();
    result.sort();
    result
}

/// Build an overlap graph from a slice of work packages.
pub fn build_overlap_graph(wps: &[WorkPackage]) -> OverlapGraph {
    let mut graph = OverlapGraph::new();
    for i in 0..wps.len() {
        for j in (i + 1)..wps.len() {
            let shared = wps[i].has_file_overlap(&wps[j]);
            if !shared.is_empty() {
                graph.edges.push((wps[i].id, wps[j].id, shared));
            }
        }
    }
    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_file_scope_basic() {
        let desc = "Implement src/models/user.rs and update crates/agileplus-cli/src/main.rs";
        let scope = detect_file_scope(desc);
        assert!(scope.iter().any(|s| s.contains("user.rs")));
        assert!(scope.iter().any(|s| s.contains("main.rs")));
    }

    #[test]
    fn detect_file_scope_deduplicates() {
        let desc = "Edit src/lib.rs and src/lib.rs again";
        let scope = detect_file_scope(desc);
        let count = scope.iter().filter(|s| s.ends_with("lib.rs")).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn build_overlap_graph_with_overlap() {
        let mut a = WorkPackage::new(1, "a", 1, "criteria");
        a.id = 1;
        a.file_scope = vec!["src/lib.rs".into(), "src/main.rs".into()];

        let mut b = WorkPackage::new(1, "b", 2, "criteria");
        b.id = 2;
        b.file_scope = vec!["src/lib.rs".into()];

        let graph = build_overlap_graph(&[a, b]);
        assert_eq!(graph.edges.len(), 1);
        let (_, _, shared) = &graph.edges[0];
        assert_eq!(shared, &["src/lib.rs".to_string()]);
    }

    #[test]
    fn build_overlap_graph_no_overlap() {
        let mut a = WorkPackage::new(1, "a", 1, "c");
        a.id = 1;
        a.file_scope = vec!["src/a.rs".into()];

        let mut b = WorkPackage::new(1, "b", 2, "c");
        b.id = 2;
        b.file_scope = vec!["src/b.rs".into()];

        let graph = build_overlap_graph(&[a, b]);
        assert_eq!(graph.edges.len(), 0);
    }

    #[test]
    fn parallel_groups_no_conflicts() {
        let graph = OverlapGraph::new();
        let groups = graph.parallel_groups(&[1, 2, 3]);
        // All in same group
        let all: Vec<i64> = groups.into_iter().flatten().collect();
        assert!(all.contains(&1) && all.contains(&2) && all.contains(&3));
    }

    #[test]
    fn parallel_groups_with_conflicts() {
        let mut graph = OverlapGraph::new();
        graph.edges.push((1, 2, vec!["f.rs".into()]));
        let groups = graph.parallel_groups(&[1, 2]);
        // 1 and 2 must be in different groups
        assert_eq!(groups.len(), 2);
        for g in &groups {
            assert!(!g.contains(&1) || !g.contains(&2));
        }
    }
}
