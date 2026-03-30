//! YAML parser for agent configurations

use std::path::Path;

use crate::config::{AgentConfig, SubagentConfig};
use crate::error::{ForgeError, Result};

/// YAML parser for loading agent configurations
pub struct YamlParser;

impl YamlParser {
    /// Parse a YAML string into an AgentConfig
    ///
    /// # Arguments
    /// * `yaml_content` - The YAML content as a string
    ///
    /// # Returns
    /// * `Ok(AgentConfig)` - Successfully parsed agent configuration
    /// * `Err(ForgeError)` - If parsing fails
    pub fn parse_string(yaml_content: &str) -> Result<AgentConfig> {
        let config: AgentConfig = serde_yaml::from_str(yaml_content)
            .map_err(|e| ForgeError::YamlError(e.to_string()))?;

        config.validate()?;
        Ok(config)
    }

    /// Parse a YAML file into an AgentConfig
    ///
    /// # Arguments
    /// * `path` - Path to the YAML file
    ///
    /// # Returns
    /// * `Ok(AgentConfig)` - Successfully parsed agent configuration
    /// * `Err(ForgeError)` - If file read or parsing fails
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<AgentConfig> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| {
            ForgeError::IoError(std::io::Error::new(
                e.kind(),
                format!("Failed to read agent file {}: {}", path.display(), e),
            ))
        })?;

        Self::parse_string(&content)
    }

    /// Parse multiple YAML files
    ///
    /// # Arguments
    /// * `paths` - Slice of paths to YAML files
    ///
    /// # Returns
    /// * `Ok(Vec<AgentConfig>)` - Vector of parsed configurations
    /// * `Err(ForgeError)` - If any file fails to parse
    pub fn parse_files<P: AsRef<Path>>(paths: &[P]) -> Result<Vec<AgentConfig>> {
        let mut configs = Vec::new();
        for path in paths {
            configs.push(Self::parse_file(path)?);
        }
        Ok(configs)
    }

    /// Parse a YAML directory (recursively finds all .yaml/.yml files)
    ///
    /// # Arguments
    /// * `dir` - Path to directory containing YAML files
    ///
    /// # Returns
    /// * `Ok(Vec<AgentConfig>)` - Vector of parsed configurations
    /// * `Err(ForgeError)` - If directory read fails
    pub fn parse_directory<P: AsRef<Path>>(dir: P) -> Result<Vec<AgentConfig>> {
        let dir = dir.as_ref();

        if !dir.is_dir() {
            return Err(ForgeError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("{} is not a directory", dir.display()),
            )));
        }

        let mut configs = Vec::new();
        let mut yaml_files = Vec::new();

        // Recursively find all .yaml and .yml files
        Self::find_yaml_files(dir, &mut yaml_files)?;

        for file_path in yaml_files {
            match Self::parse_file(&file_path) {
                Ok(config) => configs.push(config),
                Err(e) => {
                    tracing::warn!("Failed to parse {}: {}", file_path.display(), e);
                    // Continue processing other files on error
                }
            }
        }

        Ok(configs)
    }

    /// Recursively find all YAML files in a directory
    fn find_yaml_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
        for entry in std::fs::read_dir(dir).map_err(|e| {
            ForgeError::IoError(std::io::Error::new(
                e.kind(),
                format!("Failed to read directory {}: {}", dir.display(), e),
            ))
        })? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::find_yaml_files(&path, files)?;
            } else if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy();
                if ext_str == "yaml" || ext_str == "yml" {
                    files.push(path);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_yaml() -> String {
        r#"
id: test-analyzer
name: Test Analyzer
description: A test agent for analysis
instruction: Analyze the provided input and return insights
tags:
  - analyzer
  - test
enabled: true
version: 1.0.0
input_schema:
  type: object
  properties:
    text:
      type: string
output_schema:
  type: object
  properties:
    insights:
      type: array
metadata:
  author: "test-user"
  category: "analysis"
"#
        .to_string()
    }

    #[test]
    fn test_parse_string() {
        let yaml = create_test_yaml();
        let config = YamlParser::parse_string(&yaml).expect("Failed to parse YAML");

        assert_eq!(config.id, "test-analyzer");
        assert_eq!(config.name, "Test Analyzer");
        assert!(config.enabled);
        assert_eq!(config.tags.len(), 2);
    }

    #[test]
    fn test_parse_string_minimal() {
        let yaml = r#"
id: minimal-agent
name: Minimal Agent
description: A minimal agent
instruction: Do something simple
input_schema: {}
output_schema: {}
"#;
        let config = YamlParser::parse_string(yaml).expect("Failed to parse minimal YAML");

        assert_eq!(config.id, "minimal-agent");
        assert!(config.enabled); // Should default to true
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let invalid_yaml = "id: test\ninvalid yaml: [";
        let result = YamlParser::parse_string(invalid_yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_required_field() {
        let yaml = r#"
id: incomplete-agent
name: Incomplete Agent
description: Missing instruction
input_schema: {}
output_schema: {}
"#;
        let result = YamlParser::parse_string(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_file() {
        let yaml = create_test_yaml();
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("agent.yaml");

        std::fs::write(&file_path, yaml).expect("Failed to write temp file");

        let config = YamlParser::parse_file(&file_path).expect("Failed to parse file");
        assert_eq!(config.id, "test-analyzer");
    }

    #[test]
    fn test_parse_directory() {
        let yaml1 = r#"
id: agent-1
name: Agent One
description: First agent
instruction: Do task one
input_schema: {}
output_schema: {}
"#;

        let yaml2 = r#"
id: agent-2
name: Agent Two
description: Second agent
instruction: Do task two
input_schema: {}
output_schema: {}
"#;

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file1 = temp_dir.path().join("agent1.yaml");
        let file2 = temp_dir.path().join("agent2.yml");

        std::fs::write(&file1, yaml1).expect("Failed to write file 1");
        std::fs::write(&file2, yaml2).expect("Failed to write file 2");

        let configs = YamlParser::parse_directory(temp_dir.path())
            .expect("Failed to parse directory");

        assert_eq!(configs.len(), 2);
        let ids: Vec<_> = configs.iter().map(|c| c.id.as_str()).collect();
        assert!(ids.contains(&"agent-1"));
        assert!(ids.contains(&"agent-2"));
    }

    #[test]
    fn test_parse_files() {
        let yaml1 = r#"
id: agent-1
name: Agent One
description: First agent
instruction: Do task one
input_schema: {}
output_schema: {}
"#;

        let yaml2 = r#"
id: agent-2
name: Agent Two
description: Second agent
instruction: Do task two
input_schema: {}
output_schema: {}
"#;

        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let file1 = temp_dir.path().join("agent1.yaml");
        let file2 = temp_dir.path().join("agent2.yaml");

        std::fs::write(&file1, yaml1).expect("Failed to write file 1");
        std::fs::write(&file2, yaml2).expect("Failed to write file 2");

        let configs = YamlParser::parse_files(&[&file1, &file2])
            .expect("Failed to parse files");

        assert_eq!(configs.len(), 2);
    }
}
