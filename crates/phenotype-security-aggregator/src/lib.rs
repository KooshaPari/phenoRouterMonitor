//! Security alert aggregation from multiple sources.
//!
//! Aggregates security findings from Snyk, CodeQL, cargo-audit, and other sources
//! into a unified security score for the health dashboard.

use async_trait::async_trait;
use phenotype_health::{DimensionScore, Finding, Severity};
use serde::{Deserialize, Serialize};

/// Aggregates security alerts from multiple sources
pub struct SecurityAggregator {
    sources: Vec<Box<dyn SecuritySource>>,
}

/// Source of security alerts
#[async_trait]
pub trait SecuritySource: Send + Sync {
    /// Source identifier
    fn name(&self) -> &str;
    /// Fetch alerts for a repository
    async fn fetch_alerts(&self, owner: &str, repo: &str) -> anyhow::Result<Vec<SecurityAlert>>;
}

/// Security alert from any source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    pub source: AlertSource,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub cve_id: Option<String>,
    pub package_name: Option<String>,
    pub affected_versions: Option<String>,
    pub fixed_versions: Option<String>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Source of the alert
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSource {
    Snyk,
    CodeQL,
    CargoAudit,
    Dependabot,
    Trivy,
    Custom(String),
}

impl SecurityAggregator {
    /// Create a new aggregator
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    /// Add a security source
    pub fn add_source(&mut self, source: impl SecuritySource + 'static) {
        self.sources.push(Box::new(source));
    }

    /// Aggregate security score for a repository
    pub async fn aggregate_security_score(
        &self,
        owner: &str,
        repo: &str,
    ) -> anyhow::Result<DimensionScore> {
        let mut all_alerts = Vec::new();

        for source in &self.sources {
            match source.fetch_alerts(owner, repo).await {
                Ok(alerts) => all_alerts.extend(alerts),
                Err(e) => tracing::warn!("Security source {} failed: {}", source.name(), e),
            }
        }

        // Calculate score based on severity counts
        let critical = all_alerts
            .iter()
            .filter(|a| matches!(a.severity, Severity::Critical))
            .count();
        let high = all_alerts
            .iter()
            .filter(|a| matches!(a.severity, Severity::Error))
            .count();
        let medium = all_alerts
            .iter()
            .filter(|a| matches!(a.severity, Severity::Warning))
            .count();

        let score =
            (100.0_f32 - (critical as f32 * 25.0) - (high as f32 * 10.0) - (medium as f32 * 2.0))
                .max(0.0);

        let findings: Vec<Finding> = all_alerts
            .iter()
            .map(|a| Finding {
                severity: a.severity,
                message: format!(
                    "[{}] {}: {}",
                    match a.source {
                        AlertSource::Snyk => "SNYK",
                        AlertSource::CodeQL => "CODEQL",
                        AlertSource::CargoAudit => "CARGO",
                        AlertSource::Dependabot => "DEPENDABOT",
                        AlertSource::Trivy => "TRIVY",
                        AlertSource::Custom(ref s) => s.as_str(),
                    },
                    a.package_name.as_deref().unwrap_or("unknown"),
                    a.title
                ),
                file_path: a.package_name.clone(),
                line_number: None,
            })
            .collect();

        Ok(DimensionScore {
            score,
            target: 100.0,
            raw_value: all_alerts.len() as f32,
            unit: "alerts".to_string(),
            findings,
        })
    }
}

impl Default for SecurityAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// GitHub Security API source (CodeQL, Dependabot)
pub struct GitHubSecuritySource {
    client: reqwest::Client,
    token: String,
}

impl GitHubSecuritySource {
    /// Create a new GitHub security source
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            token: token.into(),
        }
    }
}

#[async_trait]
impl SecuritySource for GitHubSecuritySource {
    fn name(&self) -> &str {
        "github"
    }

    async fn fetch_alerts(&self, owner: &str, repo: &str) -> anyhow::Result<Vec<SecurityAlert>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/dependabot/alerts",
            owner, repo
        );

        let _response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        // Parse and transform to SecurityAlert
        // Placeholder: return empty for now
        tracing::info!("Fetched security alerts from GitHub for {}/{}", owner, repo);

        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockSecuritySource {
        alerts: Vec<SecurityAlert>,
    }

    #[async_trait]
    impl SecuritySource for MockSecuritySource {
        fn name(&self) -> &str {
            "mock"
        }

        async fn fetch_alerts(
            &self,
            _owner: &str,
            _repo: &str,
        ) -> anyhow::Result<Vec<SecurityAlert>> {
            Ok(self.alerts.clone())
        }
    }

    #[tokio::test]
    async fn test_empty_aggregator() {
        let aggregator = SecurityAggregator::new();
        let score = aggregator
            .aggregate_security_score("test", "repo")
            .await
            .unwrap();
        assert_eq!(score.score, 100.0);
    }
}
