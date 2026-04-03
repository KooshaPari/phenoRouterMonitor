use phenotype_health::{ProjectHealth, HealthBand, DimensionScore};
use phenotype_project_registry::discover_projects;
use phenotype_compliance_scanner::DocumentationScanner;
use std::path::Path;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HealthScanner {
    root_path: String,
    project_health: HashMap<String, ProjectHealth>,
}

impl HealthScanner {
    pub fn new(root_path: impl Into<String>, _interval: u32) -> Self {
        Self {
            root_path: root_path.into(),
            project_health: HashMap::new(),
        }
    }

    pub async fn scan_all(&mut self) -> anyhow::Result<Vec<ProjectHealth>> {
        let projects = discover_projects(Path::new(&self.root_path));
        let scanner = DocumentationScanner::new();
        let mut results = Vec::new();

        for project in projects {
            let compliance = scanner.scan(&project.path);
            let doc_dimension = DimensionScore::new("documentation", 15.0, compliance.score);

            let mut health = ProjectHealth {
                repo_name: project.name.clone(),
                language: format!("{:?}", project.project_type),
                overall_score: 0.0,
                band: HealthBand::Unknown,
                dimensions: vec![doc_dimension],
                findings_count: compliance.missing.len(),
            };
            health.compute_overall_score();

            self.project_health.insert(project.name, health.clone());
            results.push(health);
        }

        Ok(results)
    }

    pub fn health_summary(&self) -> HealthSummary {
        let total = self.project_health.len();
        let avg = if total > 0 {
            self.project_health.values().map(|h| h.overall_score).sum::<f32>() / total as f32
        } else { 0.0 };

        HealthSummary {
            total_projects: total,
            average_score: avg,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HealthSummary {
    pub total_projects: usize,
    pub average_score: f32,
}
