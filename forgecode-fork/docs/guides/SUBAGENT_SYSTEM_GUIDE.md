# Forgecode Fork: Subagent System Guide

## Overview

The Forgecode Fork Subagent System provides a **zero-code agent definition system** that enables agents to be loaded from YAML configuration files without requiring Rust code changes. This system is built on three core components:

1. **SubagentConfig Trait** — Defines the interface for agent configurations
2. **YAML Parser** — Loads agent definitions from YAML files with automatic validation
3. **Agent Discovery** — Automatically finds and registers agents from the filesystem with hot-reload support
4. **Agent Registry** — Central registry for agent lookup, filtering, and management

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Forgecode Fork Subagent System                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────┐  ┌──────────────────┐  ┌─────────────┐  │
│  │   YAML Files     │  │  YAML Parser     │  │   Trait     │  │
│  │  (agents/*.yaml) │─→│  (serde_yaml)    │─→│SubagentConfig│ │
│  └──────────────────┘  └──────────────────┘  └─────────────┘  │
│                              ↓                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │         Agent Discovery (File System Watcher)            │  │
│  │  • Recursive directory scanning                          │  │
│  │  • .yaml and .yml file detection                         │  │
│  │  • Hot reload on file changes (create/modify/delete)     │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              ↓                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │             Agent Registry (Central Store)               │  │
│  │  • HashMap<String, AgentConfig> backed by RwLock         │  │
│  │  • Async-safe operations (tokio::sync::RwLock)           │  │
│  │  • List by tag, enabled status, or all agents            │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Quick Start

### 1. Basic Agent Definition

Create a YAML file in `agents/my-agent.yaml`:

```yaml
id: my-agent
name: My Custom Agent
description: A description of what this agent does
instruction: |
  You are an expert at...
  Your task is to...

input_schema:
  type: object
  properties:
    input_field:
      type: string

output_schema:
  type: object
  properties:
    result:
      type: string

tags:
  - my-category
  - analysis

version: "1.0.0"
enabled: true
```

### 2. Initialize Discovery

```rust
use std::sync::Arc;
use forgecode_fork::{AgentDiscovery, AgentRegistry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create registry
    let registry = Arc::new(AgentRegistry::new());

    // Create discovery system pointing to agents directory
    let discovery = AgentDiscovery::new("./agents", registry.clone())?;

    // Discover and register all agents
    let agents = discovery.discover().await?;
    println!("Discovered {} agents", agents.len());

    // List all registered agents
    let all = registry.list_all().await?;
    for agent in all {
        println!("- {} ({})", agent.name, agent.id);
    }

    Ok(())
}
```

### 3. Enable Hot Reload

```rust
// Initialize file watcher for automatic agent reloading
discovery.initialize_hot_reload()?;

// Continue running application
// Changes to YAML files will be detected and agents reloaded
tokio::signal::ctrl_c().await?;
```

## Agent Configuration Schema

### Required Fields

```yaml
id: unique-agent-identifier
name: Human Readable Name
description: What this agent does
instruction: |
  The actual prompt/instruction for the agent
input_schema:
  type: object
  # JSON Schema for expected input
output_schema:
  type: object
  # JSON Schema for expected output
```

### Optional Fields

```yaml
tags:                    # Array of category tags
  - analysis
  - validation

metadata:                # Custom key-value pairs
  author: "Team"
  requires_model: "opus"

enabled: true            # Enable/disable agent without deletion
version: "1.0.0"         # Agent configuration version
```

## Usage Examples

### Example 1: Initialize Registry and Discover Agents

```rust
use std::sync::Arc;
use forgecode_fork::AgentRegistry;
use forgecode_fork::AgentDiscovery;

#[tokio::main]
async fn main() -> forgecode_fork::Result<()> {
    let registry = Arc::new(AgentRegistry::new());
    let discovery = AgentDiscovery::new("./agents", registry.clone())?;

    // Discover all agents
    let agents = discovery.discover().await?;
    assert!(agents.len() > 0);

    println!("Registered {} agents", agents.len());
    Ok(())
}
```

### Example 2: Query Agents by Tag

```rust
// Find all agents with "analysis" tag
let analysis_agents = registry.list_by_tag("analysis").await?;

for agent in analysis_agents {
    println!("{}: {}", agent.id, agent.description);
}
```

### Example 3: List Only Enabled Agents

```rust
// Get only enabled agents
let enabled = registry.list_enabled().await?;

for agent in enabled {
    if agent.enabled {
        println!("Active: {}", agent.name);
    }
}
```

### Example 4: Get Agent by ID

```rust
// Get specific agent with required validation
let agent = registry.get_required("analyzer-agent").await?;
println!("Input schema: {}", agent.input_schema);
println!("Instruction: {}", agent.instruction);
```

### Example 5: Check if Agent Exists

```rust
if registry.exists("my-agent").await? {
    println!("Agent found!");
} else {
    println!("Agent not found");
}
```

## File Structure

```
forgecode-fork/
├── src/
│   ├── lib.rs                 # Main module exports
│   ├── config.rs              # SubagentConfig trait & AgentConfig struct
│   ├── parser.rs              # YAML parsing functionality
│   ├── error.rs               # Error types
│   ├── discovery.rs           # Agent discovery & hot reload
│   └── registry.rs            # Agent registry & storage
├── agents/
│   ├── analyzer.yaml          # Sample analyzer agent
│   ├── validator.yaml         # Sample validator agent
│   └── reporter.yaml          # Sample reporter agent
├── docs/
│   └── guides/
│       └── SUBAGENT_SYSTEM_GUIDE.md  # This file
├── Cargo.toml                 # Project dependencies
└── tests/
    └── integration_tests.rs    # Integration test examples
```

## API Reference

### SubagentConfig Trait

```rust
pub trait SubagentConfig: Send + Sync + Debug {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn instruction(&self) -> &str;
    fn input_schema(&self) -> &serde_json::Value;
    fn output_schema(&self) -> &serde_json::Value;
    fn metadata(&self) -> HashMap<String, String>;
    fn validate(&self) -> Result<()>;
    async fn initialize(&self) -> Result<()>;
    fn tags(&self) -> Vec<&str>;
    fn is_enabled(&self) -> bool;
}
```

### AgentConfig Struct

```rust
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub instruction: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub metadata: HashMap<String, String>,
    pub tags: Vec<String>,
    pub enabled: bool,
    pub version: String,
    pub extra: serde_json::Value,  // Extensible properties
}
```

### YamlParser Methods

```rust
// Parse from string
pub fn parse_string(yaml_content: &str) -> Result<AgentConfig>

// Parse from file
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<AgentConfig>

// Parse multiple files
pub fn parse_files<P: AsRef<Path>>(paths: &[P]) -> Result<Vec<AgentConfig>>

// Parse directory (recursive)
pub fn parse_directory<P: AsRef<Path>>(dir: P) -> Result<Vec<AgentConfig>>
```

### AgentRegistry Methods

```rust
// Registration
pub async fn register(&self, agent: AgentConfig) -> Result<()>
pub async fn unregister(&self, agent_id: &str) -> Result<()>

// Lookup
pub async fn get(&self, agent_id: &str) -> Result<Option<AgentConfig>>
pub async fn get_required(&self, agent_id: &str) -> Result<AgentConfig>
pub async fn exists(&self, agent_id: &str) -> Result<bool>

// Listing
pub async fn list_all(&self) -> Result<Vec<AgentConfig>>
pub async fn list_by_tag(&self, tag: &str) -> Result<Vec<AgentConfig>>
pub async fn list_enabled(&self) -> Result<Vec<AgentConfig>>

// Management
pub async fn count(&self) -> Result<usize>
pub async fn clear(&self) -> Result<()>
```

### AgentDiscovery Methods

```rust
// Create discovery instance
pub fn new<P: AsRef<Path>>(discovery_path: P, registry: Arc<AgentRegistry>)
    -> Result<Self>

// Discover agents
pub async fn discover(&self) -> Result<Vec<AgentConfig>>

// Initialize hot reload (file watcher)
pub fn initialize_hot_reload(&self) -> Result<()>

// Accessors
pub fn discovery_path(&self) -> &Path
pub fn registry(&self) -> &AgentRegistry
```

## Validation

Agents are automatically validated when:
1. Parsed from YAML
2. Registered in the registry
3. Initialized for use

Validation checks:
- ✓ Agent ID is not empty
- ✓ Agent name is not empty
- ✓ Instruction is not empty
- ✓ Agent ID contains only alphanumeric, hyphen, or underscore characters
- ✓ Input schema is valid JSON
- ✓ Output schema is valid JSON

## Hot Reload

The system supports automatic agent reloading when files change:

```rust
discovery.initialize_hot_reload()?;
```

This watches for:
- **Create**: New agent files are automatically registered
- **Modify**: Changes to agent YAML are reloaded
- **Delete**: Removed agents are unregistered

Hot reload runs in a background tokio task and does not block the main application.

## Error Handling

The system uses a custom `ForgeError` type for comprehensive error reporting:

```rust
pub enum ForgeError {
    YamlError(String),           // YAML parsing failed
    ConfigError(String),          // Config validation failed
    IoError(std::io::Error),      // File system error
    AgentNotFound(String),        // Agent ID not in registry
    InvalidAgent(String),         // Agent definition invalid
    DiscoveryError(String),       // Discovery system error
    RegistryError(String),        // Registry operation failed
    SchemaError(String),          // JSON Schema error
    Other(String),                // Generic error
}
```

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Register agent | O(1) | HashMap insertion |
| Get agent | O(1) | HashMap lookup |
| List all | O(n) | Returns all n agents |
| List by tag | O(n) | Filters n agents |
| Discover directory | O(m) | Parses m YAML files |
| Hot reload initialization | O(1) | Starts background watcher |

**Concurrency**: All registry operations are fully async-safe using `tokio::sync::RwLock`. Multiple readers can access agents simultaneously without blocking.

## Testing

The system includes 28 comprehensive tests covering:
- ✓ Agent configuration creation and validation
- ✓ YAML parsing (strings, files, directories)
- ✓ Agent registration and unregistration
- ✓ Registry queries (list, filter, exists)
- ✓ Hot reload discovery
- ✓ Error handling

Run tests with:

```bash
cargo test --lib
```

## Best Practices

### 1. Agent ID Naming

Use lowercase alphanumeric with hyphens:

```yaml
id: my-analyzer-agent    # ✓ Good
id: MyAnalyzerAgent      # ✗ Invalid (uppercase)
id: my_analyzer_agent    # ✓ Also valid (underscores OK)
id: my analyzer agent    # ✗ Invalid (spaces)
```

### 2. Schema Design

Make schemas as specific as possible:

```yaml
input_schema:
  type: object
  required:
    - code_content
  properties:
    code_content:
      type: string
      minLength: 1
      description: The code to analyze
    language:
      type: string
      enum: [rust, python, go, typescript]
```

### 3. Instructions

Write clear, actionable instructions:

```yaml
instruction: |
  You are a code review expert. Analyze the provided code for:
  1. Security vulnerabilities
  2. Performance issues
  3. Code style violations

  Output structured findings with severity levels.
```

### 4. Metadata

Include operational metadata:

```yaml
metadata:
  author: "Team Name"
  requires_model: "claude-opus"  # Minimum model required
  estimated_tokens: "2000-5000"
  owner: "team@company.com"
  support_url: "https://docs.example.com"
```

### 5. Tags

Use consistent tag naming across agents:

```yaml
tags:
  - analysis      # What it does
  - code-quality  # Category
  - async-safe    # Special properties
```

## Common Patterns

### Pattern: Agent Factory

```rust
pub struct AgentFactory {
    registry: Arc<AgentRegistry>,
}

impl AgentFactory {
    pub async fn create(id: &str) -> Result<AgentConfig> {
        self.registry.get_required(id).await
    }

    pub async fn create_for_task(task_type: &str)
        -> Result<Vec<AgentConfig>> {
        self.registry.list_by_tag(task_type).await
    }
}
```

### Pattern: Agent Resolver

```rust
pub async fn resolve_agent(
    name: impl AsRef<str>,
    registry: &AgentRegistry
) -> Result<AgentConfig> {
    registry.get_required(name.as_ref()).await
}
```

## Troubleshooting

### Agent Not Found

Check that:
1. Agent file exists in `agents/` directory
2. File has `.yaml` or `.yml` extension
3. Agent `id` field is unique
4. Discovery was called: `discovery.discover().await?`

### YAML Parse Error

Verify:
1. YAML syntax is valid (use `yamllint`)
2. Required fields are present (id, name, description, instruction)
3. JSON schemas are valid JSON
4. Indentation is consistent (use 2 spaces)

### Hot Reload Not Working

Ensure:
1. `initialize_hot_reload()` was called
2. Application is still running
3. File notifications are supported on your OS
4. Watch directory has read permissions

## Future Enhancements

- [ ] Agent versioning and migrations
- [ ] Remote agent registries
- [ ] Agent dependency resolution
- [ ] Performance metrics collection
- [ ] Agent caching layer
- [ ] Distributed registry support
- [ ] Agent update scheduling

## Contributing

When adding new agents, ensure:
1. Valid YAML syntax
2. Unique agent ID following naming conventions
3. Comprehensive input/output schemas
4. Clear, actionable instructions
5. Appropriate tags for discoverability
6. Metadata for operational context

---

**Last Updated**: 2026-03-30
**Version**: 1.0.0
**Maintainer**: Phenotype Team
