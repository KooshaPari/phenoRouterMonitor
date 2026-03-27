use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use agileplus_domain::domain::{
    cycle::CycleState,
    module::Module,
    state_machine::FeatureState,
    work_package::{PrState, WpState},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportBundle {
    #[serde(default)]
    pub projects: Vec<ImportProject>,
    #[serde(default)]
    pub modules: Vec<ImportModule>,
    #[serde(default)]
    pub features: Vec<ImportFeature>,
    #[serde(default)]
    pub cycles: Vec<ImportCycle>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportProject {
    #[serde(default)]
    pub slug: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub features: Vec<ImportFeature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportModule {
    #[serde(default)]
    pub slug: Option<String>,
    pub friendly_name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub parent_slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportFeature {
    #[serde(default)]
    pub slug: Option<String>,
    pub friendly_name: String,
    pub spec_content: String,
    #[serde(default = "default_feature_state")]
    pub state: FeatureState,
    #[serde(default)]
    pub target_branch: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub module_slug: Option<String>,
    #[serde(default)]
    pub project_id: Option<i64>,
    #[serde(default)]
    pub plane_issue_id: Option<String>,
    #[serde(default)]
    pub plane_state_id: Option<String>,
    #[serde(default)]
    pub work_packages: Vec<ImportWorkPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportWorkPackage {
    pub title: String,
    #[serde(default)]
    pub acceptance_criteria: Option<String>,
    #[serde(default)]
    pub sequence: Option<i32>,
    #[serde(default)]
    pub file_scope: Vec<String>,
    #[serde(default = "default_wp_state")]
    pub state: WpState,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub pr_url: Option<String>,
    #[serde(default)]
    pub pr_state: Option<PrState>,
    #[serde(default)]
    pub worktree_path: Option<String>,
    #[serde(default)]
    pub plane_sub_issue_id: Option<String>,
    #[serde(default)]
    pub depends_on_sequences: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportCycle {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    #[serde(default = "default_cycle_state")]
    pub state: CycleState,
    #[serde(default)]
    pub module_scope_slug: Option<String>,
    #[serde(default)]
    pub feature_slugs: Vec<String>,
}

fn default_feature_state() -> FeatureState {
    FeatureState::Specified
}

fn default_wp_state() -> WpState {
    WpState::Planned
}

fn default_cycle_state() -> CycleState {
    CycleState::Draft
}

impl ImportModule {
    pub fn slug(&self) -> String {
        self.slug
            .clone()
            .unwrap_or_else(|| Module::slug_from_name(&self.friendly_name))
    }
}
