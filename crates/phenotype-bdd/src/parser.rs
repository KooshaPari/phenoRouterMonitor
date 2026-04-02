//! Gherkin parser for feature files

use super::{BddError, Result};
use thiserror::Error;

/// A complete feature with scenarios
#[derive(Debug, Clone, PartialEq)]
pub struct Feature {
    /// Feature name
    pub name: String,
    /// Feature description (optional)
    pub description: Option<String>,
    /// List of scenarios in this feature
    pub scenarios: Vec<Scenario>,
}

/// A scenario within a feature
#[derive(Debug, Clone, PartialEq)]
pub struct Scenario {
    /// Scenario name
    pub name: String,
    /// Steps in this scenario
    pub steps: Vec<Step>,
    /// Tags attached to this scenario
    pub tags: Vec<String>,
}

/// Type of step in Gherkin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StepType {
    Given,
    When,
    Then,
    And,
    But,
}

impl StepType {
    /// Parse a step type from a keyword string
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword.trim().to_lowercase().as_str() {
            "given" => Some(Self::Given),
            "when" => Some(Self::When),
            "then" => Some(Self::Then),
            "and" => Some(Self::And),
            "but" => Some(Self::But),
            _ => None,
        }
    }

    /// Get the keyword string for this step type
    pub fn as_keyword(&self) -> &'static str {
        match self {
            Self::Given => "Given",
            Self::When => "When",
            Self::Then => "Then",
            Self::And => "And",
            Self::But => "But",
        }
    }
}

/// A single step in a scenario
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    /// Type of step
    pub step_type: StepType,
    /// Step text (without keyword)
    pub text: String,
    /// Step data table (optional)
    pub table: Option<DataTable>,
}

/// Data table for steps
#[derive(Debug, Clone, PartialEq)]
pub struct DataTable {
    /// Headers of the table
    pub headers: Vec<String>,
    /// Rows of data
    pub rows: Vec<Vec<String>>,
}

/// Error during parsing
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid feature file: {0}")]
    InvalidFeature(String),
    #[error("Missing feature keyword")]
    MissingFeature,
    #[error("Invalid step syntax: {0}")]
    InvalidStep(String),
}

impl From<ParseError> for BddError {
    fn from(e: ParseError) -> Self {
        BddError::ParseError(e.to_string())
    }
}

/// Parse a Gherkin feature file
pub fn parse_feature(content: &str) -> Result<Feature> {
    let mut lines = content.lines().peekable();
    let mut scenarios = Vec::new();
    let mut feature_name = None;
    let mut feature_description = None;
    let mut current_scenario: Option<Scenario> = None;
    let mut current_tags = Vec::new();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse tags
        if trimmed.starts_with('@') {
            current_tags.push(trimmed.trim_start_matches('@').to_string());
            continue;
        }

        // Parse feature
        if trimmed.starts_with("Feature:") {
            feature_name = Some(trimmed.trim_start_matches("Feature:").trim().to_string());
            // Collect description lines until we hit Scenario or Background
            let mut desc_lines = Vec::new();
            while let Some(&next_line) = lines.peek() {
                let next_trimmed = next_line.trim();
                if next_trimmed.is_empty()
                    || next_trimmed.starts_with("Scenario")
                    || next_trimmed.starts_with("Background")
                    || next_trimmed.starts_with('@')
                {
                    break;
                }
                desc_lines.push(next_trimmed.to_string());
                lines.next();
            }
            if !desc_lines.is_empty() {
                feature_description = Some(desc_lines.join("\n"));
            }
            continue;
        }

        // Parse scenario
        if trimmed.starts_with("Scenario:") || trimmed.starts_with("Scenario Outline:") {
            // Save previous scenario if any
            if let Some(scenario) = current_scenario.take() {
                scenarios.push(scenario);
            }

            let name = trimmed
                .trim_start_matches("Scenario:")
                .trim_start_matches("Scenario Outline:")
                .trim()
                .to_string();

            current_scenario = Some(Scenario {
                name,
                steps: Vec::new(),
                tags: std::mem::take(&mut current_tags),
            });
            continue;
        }

        // Parse steps
        let step_keywords = ["Given", "When", "Then", "And", "But"];
        for keyword in &step_keywords {
            if trimmed.starts_with(&format!("{} ", keyword))
                || trimmed.starts_with(&format!("{}\t", keyword))
            {
                let text = trimmed
                    .trim_start_matches(keyword)
                    .trim_start_matches(' ')
                    .trim_start_matches('\t')
                    .to_string();

                let step = Step {
                    step_type: StepType::from_keyword(keyword).unwrap(),
                    text,
                    table: None,
                };

                if let Some(ref mut scenario) = current_scenario {
                    scenario.steps.push(step);
                }
                break;
            }
        }
    }

    // Don't forget the last scenario
    if let Some(scenario) = current_scenario {
        scenarios.push(scenario);
    }

    let name = feature_name.ok_or(ParseError::MissingFeature)?;

    Ok(Feature {
        name,
        description: feature_description,
        scenarios,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_feature() {
        let content = r#"
Feature: User Management
  As an admin
  I want to manage users
  So that I can control access

  @smoke
  Scenario: Create a new user
    Given I am logged in as admin
    When I create a user with name "Alice"
    Then the user should exist
"#;

        let feature = parse_feature(content).unwrap();
        assert_eq!(feature.name, "User Management");
        assert!(feature.description.is_some());
        assert_eq!(feature.scenarios.len(), 1);

        let scenario = &feature.scenarios[0];
        assert_eq!(scenario.name, "Create a new user");
        assert!(scenario.tags.contains(&"smoke".to_string()));
        assert_eq!(scenario.steps.len(), 3);
    }

    #[test]
    fn test_parse_multiple_scenarios() {
        let content = r#"
Feature: Authentication

  Scenario: Login with valid credentials
    Given a user exists
    When they login with valid password
    Then they should be authenticated

  Scenario: Login with invalid credentials
    Given a user exists
    When they login with invalid password
    Then they should see an error
"#;

        let feature = parse_feature(content).unwrap();
        assert_eq!(feature.scenarios.len(), 2);
    }

    #[test]
    fn test_step_type_parsing() {
        assert_eq!(StepType::from_keyword("Given"), Some(StepType::Given));
        assert_eq!(StepType::from_keyword("When"), Some(StepType::When));
        assert_eq!(StepType::from_keyword("Then"), Some(StepType::Then));
        assert_eq!(StepType::from_keyword("And"), Some(StepType::And));
        assert_eq!(StepType::from_keyword("Unknown"), None);
    }
}
