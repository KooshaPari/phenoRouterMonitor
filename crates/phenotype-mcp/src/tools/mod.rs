//! Built-in MCP tool definitions for Phenotype.

use crate::ToolDef;

/// Create the default set of Phenotype MCP tools.
pub fn default_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "phenotype_version".into(),
            description: "Get the current Phenotype version".into(),
            input_schema: serde_json::json!({"type": "object", "properties": {}}),
        },
        ToolDef {
            name: "phenotype_status".into(),
            description: "Get workspace health status".into(),
            input_schema: serde_json::json!({"type": "object", "properties": {}}),
        },
    ]
}
