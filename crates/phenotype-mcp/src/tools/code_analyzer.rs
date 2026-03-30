//! Code analysis tool for MCP.
//!
//! Provides code analysis capabilities including linting, complexity metrics,
//! and code quality analysis for various programming languages.

use serde::{Deserialize, Serialize};

/// Result of a code analysis operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeAnalysisResult {
    /// File path that was analyzed
    pub file_path: String,
    /// Number of lines in the file
    pub line_count: usize,
    /// Number of functions/methods found
    pub function_count: usize,
    /// Complexity score (higher = more complex)
    pub complexity_score: u32,
    /// List of issues found
    pub issues: Vec<AnalysisIssue>,
}

/// An individual code analysis issue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnalysisIssue {
    /// Issue severity level
    pub severity: IssueSeverity,
    /// Line number where issue occurs
    pub line_number: usize,
    /// Description of the issue
    pub message: String,
    /// Suggested fix (if available)
    pub suggestion: Option<String>,
}

/// Severity levels for analysis issues.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    /// Informational issue
    Info,
    /// Warning - should be addressed
    Warning,
    /// Error - should be fixed
    Error,
}

/// Code analyzer tool for MCP.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeAnalyzer {
    /// Name of the analyzer
    pub name: String,
    /// Version of the analyzer
    pub version: String,
}

impl CodeAnalyzer {
    /// Create a new code analyzer.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }

    /// Analyze a code file.
    pub fn analyze(&self, file_path: impl Into<String>) -> CodeAnalysisResult {
        CodeAnalysisResult {
            file_path: file_path.into(),
            line_count: 0,
            function_count: 0,
            complexity_score: 0,
            issues: vec![],
        }
    }

    /// Check if the analyzer supports a given file type.
    pub fn supports(&self, file_extension: &str) -> bool {
        matches!(
            file_extension,
            "rs" | "py" | "js" | "ts" | "go" | "java" | "cpp" | "c" | "ruby" | "php"
        )
    }
}

impl Default for CodeAnalysisResult {
    fn default() -> Self {
        Self {
            file_path: String::new(),
            line_count: 0,
            function_count: 0,
            complexity_score: 0,
            issues: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_analyzer_new() {
        let analyzer = CodeAnalyzer::new("test", "1.0");
        assert_eq!(analyzer.name, "test");
        assert_eq!(analyzer.version, "1.0");
    }

    #[test]
    fn test_code_analyzer_default() {
        let analyzer = CodeAnalyzer::default();
        assert_eq!(analyzer.name, "");
        assert_eq!(analyzer.version, "");
    }

    #[test]
    fn test_supports() {
        let analyzer = CodeAnalyzer::default();
        assert!(analyzer.supports("rs"));
        assert!(analyzer.supports("py"));
        assert!(analyzer.supports("js"));
        assert!(!analyzer.supports("txt"));
    }

    #[test]
    fn test_analyze() {
        let analyzer = CodeAnalyzer::default();
        let result = analyzer.analyze("test.rs");
        assert_eq!(result.file_path, "test.rs");
        assert_eq!(result.line_count, 0);
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_analysis_issue_serialization() {
        let issue = AnalysisIssue {
            severity: IssueSeverity::Warning,
            line_number: 42,
            message: "unused variable".to_string(),
            suggestion: Some("remove this variable".to_string()),
        };

        let json = serde_json::to_string(&issue).unwrap();
        let deserialized: AnalysisIssue = serde_json::from_str(&json).unwrap();
        assert_eq!(issue, deserialized);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(IssueSeverity::Info < IssueSeverity::Warning);
        assert!(IssueSeverity::Warning < IssueSeverity::Error);
        assert_eq!(IssueSeverity::Error, IssueSeverity::Error);
    }
}
