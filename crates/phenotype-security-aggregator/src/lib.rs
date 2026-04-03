//! Phenotype Security Aggregator
//!
//! Aggregates security alerts from multiple sources (Snyk, CodeQL, cargo-audit).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Severity levels for security findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl Severity {
    /// Get numeric value for sorting
    pub fn numeric_value(&self) -> u8 {
        match self {
            Severity::Critical => 5,
            Severity::High => 4,
            Severity::Medium => 3,
            Severity::Low => 2,
            Severity::Info => 1,
        }
    }
}

/// A security alert finding
#[derive(Debug, Clone)]
pub struct Finding {
    /// Unique identifier
    pub id: String,
    /// Alert title
    pub title: String,
    /// Alert description
    pub description: String,
    /// Severity level
    pub severity: Severity,
    /// Source system
    pub source: String,
    /// File path (if applicable)
    pub file: Option<String>,
    /// Line number (if applicable)
    pub line: Option<u32>,
    /// When the alert was created
    pub created_at: DateTime<Utc>,
    /// CWE ID (if applicable)
    pub cwe_id: Option<String>,
    /// CVSS score (if applicable)
    pub cvss_score: Option<f32>,
}

/// Score for a specific dimension of security
#[derive(Debug, Clone)]
pub struct DimensionScore {
    /// Dimension name
    pub name: String,
    /// Score value (0-100)
    pub score: f32,
    /// Related findings
    pub findings: Vec<Finding>,
}

/// Source of the alert
#[derive(Debug, Clone)]
pub enum AlertSource {
    Snyk,
    CodeQL,
    CargoAudit,
    Dependabot,
    Trivy,
    Custom(String),
}

impl AlertSource {
    /// Get short name for display
    pub fn short_name(&self) -> &str {
        match self {
            AlertSource::Snyk => "SNYK",
            AlertSource::CodeQL => "CODEQL",
            AlertSource::CargoAudit => "CARGO",
            AlertSource::Dependabot => "DEPND",
            AlertSource::Trivy => "TRIVY",
            AlertSource::Custom(s) => s.as_str(),
        }
    }
}

/// Security alert from external sources
#[derive(Debug, Clone)]
pub struct SecurityAlert {
    pub source: AlertSource,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub cve_id: Option<String>,
    pub package_name: Option<String>,
    pub affected_versions: Option<String>,
    pub fixed_versions: Option<String>,
    pub detected_at: DateTime<Utc>,
}

/// Aggregates security alerts from multiple sources
pub struct SecurityAggregator {
    sources: Vec<Box<dyn SecuritySource>>,
}

/// Trait for security alert sources
pub trait SecuritySource: Send + Sync {
    /// Fetch alerts from this source
    fn fetch_alerts(&self, owner: &str, repo: &str) -> impl std::future::Future<Output = anyhow::Result<Vec<SecurityAlert>>> + Send;
}

impl SecurityAggregator {
    /// Create a new aggregator
    pub fn new() -> Self {
        Self { sources: Vec::new() }
    }

    /// Add a security source
    pub fn add_source(&mut self, source: impl SecuritySource + 'static) {
        self.sources.push(Box::new(source));
    }

    /// Aggregate security score from all sources
    pub async fn aggregate_security_score(
        &self,
        owner: &str,
        repo: &str,
    ) -> anyhow::Result<DimensionScore> {
        let mut all_alerts = Vec::new();

        for source in &self.sources {
            match source.fetch_alerts(owner, repo).await {
                Ok(alerts) => all_alerts.extend(alerts),
                Err(e) => tracing::warn!("Security source failed: {}", e),
            }
        }

        // Calculate score based on severity counts
        let critical = all_alerts
            .iter()
            .filter(|a| matches!(a.severity, Severity::Critical))
            .count();
        let high = all_alerts
            .iter()
            .filter(|a| matches!(a.severity, Severity::High))
            .count();
        let medium = all_alerts
            .iter()
            .filter(|a| matches!(a.severity, Severity::Medium))
            .count();

        let deduction = critical as f32 * 25.0 + high as f32 * 10.0 + medium as f32 * 2.0;
        let score = (100.0_f32 - deduction).max(0.0);

        let findings: Vec<Finding> = all_alerts
            .iter()
            .map(|a| Finding {
                id: a.cve_id.clone().unwrap_or_else(|| a.title.clone()),
                title: a.title.clone(),
                description: a.description.clone(),
                severity: a.severity,
                source: a.source.short_name().to_string(),
                file: None,
                line: None,
                created_at: a.detected_at,
                cwe_id: None,
                cvss_score: None,
            })
            .collect();

        Ok(DimensionScore {
            name: "security".to_string(),
            score,
            findings,
        })
    }
}

impl Default for SecurityAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// GitHub Security API implementation
pub struct GitHubSecuritySource {
    client: reqwest::Client,
    token: String,
}

impl GitHubSecuritySource {
    /// Create a new GitHub security source
    pub fn new(token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            token,
        }
    }
}

impl SecuritySource for GitHubSecuritySource {
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
        // TODO: Implement actual parsing
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_aggregator() {
        let aggregator = SecurityAggregator::new();
        assert!(aggregator.sources.is_empty());
    }

    #[test]
    fn test_severity_numeric_value() {
        assert_eq!(Severity::Critical.numeric_value(), 5);
        assert_eq!(Severity::High.numeric_value(), 4);
        assert_eq!(Severity::Medium.numeric_value(), 3);
        assert_eq!(Severity::Low.numeric_value(), 2);
        assert_eq!(Severity::Info.numeric_value(), 1);
    }

    #[test]
    fn test_alert_source_short_name() {
        assert_eq!(AlertSource::Snyk.short_name(), "SNYK");
        assert_eq!(AlertSource::CodeQL.short_name(), "CODEQL");
        assert_eq!(AlertSource::CargoAudit.short_name(), "CARGO");
        assert_eq!(AlertSource::Dependabot.short_name(), "DEPND");
        assert_eq!(AlertSource::Trivy.short_name(), "TRIVY");
        assert_eq!(AlertSource::Custom("Custom".to_string()).short_name(), "Custom");
    }
}
