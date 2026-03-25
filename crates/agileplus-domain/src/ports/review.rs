//! Review port — code review operations abstraction.
//!
//! Traceability: FR-012, FR-013 / WP05-T028

use serde::{Deserialize, Serialize};

use crate::error::DomainError;

/// Severity level of a review comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommentSeverity {
    Critical,
    Major,
    Minor,
    Informational,
}

/// A single review comment on a pull request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    pub author: String,
    pub body: String,
    pub file_path: Option<String>,
    pub line: Option<u32>,
    pub severity: CommentSeverity,
    pub actionable: bool,
}

/// Aggregate status of a code review.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Pending,
    InProgress,
    Approved,
    ChangesRequested { comments: Vec<ReviewComment> },
    Rejected { reason: String },
}

/// Status of CI checks on a pull request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CiStatus {
    Pending,
    Running,
    Passed,
    Failed { logs_url: String },
    Cancelled,
}

/// Summary information about a pull request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrInfo {
    pub url: String,
    pub number: u64,
    pub title: String,
    pub state: String,
    pub review_status: ReviewStatus,
    pub ci_status: CiStatus,
}

/// Port for code review operations.
///
/// Abstracts review providers (Coderabbit, GitHub API, manual fallback).
/// The Review adapter (WP09) implements this.
pub trait ReviewPort: Send + Sync {
    /// Get the current review status for a pull request.
    fn get_review_status(
        &self,
        pr_url: &str,
    ) -> impl std::future::Future<Output = Result<ReviewStatus, DomainError>> + Send;

    fn get_review_comments(
        &self,
        pr_url: &str,
    ) -> impl std::future::Future<Output = Result<Vec<ReviewComment>, DomainError>> + Send;

    fn get_actionable_comments(
        &self,
        pr_url: &str,
    ) -> impl std::future::Future<Output = Result<Vec<ReviewComment>, DomainError>> + Send;

    fn get_ci_status(
        &self,
        pr_url: &str,
    ) -> impl std::future::Future<Output = Result<CiStatus, DomainError>> + Send;

    fn get_pr_info(
        &self,
        pr_url: &str,
    ) -> impl std::future::Future<Output = Result<PrInfo, DomainError>> + Send;

    fn await_review(
        &self,
        pr_url: &str,
        timeout_secs: u64,
    ) -> impl std::future::Future<Output = Result<ReviewStatus, DomainError>> + Send;

    fn await_ci(
        &self,
        pr_url: &str,
        timeout_secs: u64,
    ) -> impl std::future::Future<Output = Result<CiStatus, DomainError>> + Send;
}
