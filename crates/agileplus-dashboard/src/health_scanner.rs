use phenotype_health::{ProjectHealth, HealthBand, DimensionScore, Finding};
use phenotype_project_registry::discover_projects;
use std::path::Path;
use std::collections::HashMap;

const REQUIRED_DOCS: &[&str] = &["CLAUDE.md", "README.md", "CONTRIBUTING.md", "LICENSE", "CHANGELOG.md"];

#[derive(Debug, Clone)]
pub struct HealthScanner {
    root_path: String,
    scan_interval_hours: u32,
    last_scan: Option<chrono::DateTime<chrono::Utc>>,
    project_health: HashMap<String, ProjectHealth>,
}

impl HealthScanner {
    pub fn new(root_path: impl Into<String>, scan_interval_hours: u32) -> Self {
        Self {
            root_path: root_path.into(),
            scan_interval_hours,
            last_scan: None,
            project_health: HashMap::new(),
        }
    }

    pub fn projects(&self) -> &HashMap<String, ProjectHealth> {
        &self.project_health
    }

    pub async fn scan_all(&mut self) -> anyhow::Result<Vec<ProjectHealth>> {
        let projects = discover_projects(Path::new(&self.root_path));
        let mut health_results = Vec::new();

        for project in projects {
            let mut present = 0;
            let mut missing = Vec::new();
            
            for doc in REQUIRED_DOCS {
                if project.path.join(doc).exists() {
                    present += 1;
                } else {
                    missing.push(doc.to_string());
                }
            }
            
            let doc_score = (present as f32 / REQUIRED_DOCS.len() as f32) * 100.0;
            
            let mut findings = Vec::new();
            if !missing.is_empty() {
                findings.push(Finding::warning(format!("Missing: {:?}", missing)));
            } else {
                findings.push(Finding::info("All required docs present"));
            }
            
            let doc_dimension = DimensionScore {
                dimension: "documentation".to_string(),
                weight: 15.0,
                score: doc_score,
                findings,
            };

            let mut health = ProjectHealth {
                repo_name: project.name.clone(),
                language: format!("{:?}", project.project_type),
                overall_score: 0.0,
                band: HealthBand::Unknown,
                dimensions: vec![doc_dimension],
                findings_count: missing.len(),
            };
            health.compute_overall_score();

            self.project_health.insert(project.name.clone(), health.clone());
            health_results.push(health);
        }

        self.last_scan = Some(chrono::Utc::now());
        Ok(health_results)
    }

    pub fn health_summary(&self) -> HealthSummary {
        let mut by_band: HashMap<HealthBand, usize> = HashMap::new();
        let mut total_score = 0.0;
        let mut count = 0;

        for health in self.project_health.values() {
            *by_band.entry(health.band).or_insert(0) += 1;
            total_score += health.overall_score;
            count += 1;
        }

        HealthSummary {
            total_projects: self.project_health.len(),
            average_score: if count > 0 { total_score / count as f32 } else { 0.0 },
            by_band,
            last_scan: self.last_scan,
        }
    }

    pub fn get_project(&self, name: &str) -> Option<&ProjectHealth> {
        self.project_health.get(name)
    }

    pub fn needs_scan(&self) -> bool {
        match self.last_scan {
            None => true,
            Some(last) => {
                let elapsed = chrono::Utc::now().signed_duration_since(last);
                elapsed.num_hours() >= self.scan_interval_hours as i64
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HealthSummary {
    pub total_projects: usize,
    pub average_score: f32,
    pub by_band: HashMap<HealthBand, usize>,
    pub last_scan: Option<chrono::DateTime<chrono::Utc>>,
}
