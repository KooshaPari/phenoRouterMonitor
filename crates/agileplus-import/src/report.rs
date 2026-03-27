use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ImportReport {
    pub projects_created: usize,
    pub projects_updated: usize,
    pub modules_created: usize,
    pub modules_updated: usize,
    pub features_created: usize,
    pub features_updated: usize,
    pub cycles_created: usize,
    pub cycles_updated: usize,
    pub work_packages_created: usize,
    pub work_packages_updated: usize,
    pub module_links_created: usize,
    pub cycle_links_created: usize,
    pub artifacts_written: usize,
    pub audits_written: usize,
}

