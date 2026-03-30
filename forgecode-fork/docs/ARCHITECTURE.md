# forgecode Fork: Architecture & Integration Points

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Forgecode Fork Ecosystem                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                      AgilePlus Dashboard UI                         │   │
│  │  ┌─────────────────────┐  ┌──────────────────┐  ┌──────────────┐   │   │
│  │  │ Provider Gallery    │  │ Evidence Timeline│  │ Work Distrib.│   │   │
│  │  │ (hover-expand)      │  │ (clickable links)│  │ (real-time)  │   │   │
│  │  └─────────────────────┘  └──────────────────┘  └──────────────┘   │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                    ▲                                         │
│                                    │ REST API                                │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                    Forgecode Core (Upstream)                         │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐   │   │
│  │  │ CLI Engine   │  │ Plugin System│  │ Configuration Management │   │   │
│  │  └──────────────┘  └──────────────┘  └──────────────────────────┘   │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                    ▲                                         │
│                                    │ Plugin API                              │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                  Custom Extensions (Fork-Specific)                   │   │
│  │                                                                       │   │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐         │   │
│  │  │ Providers      │  │ Subagent Exts. │  │ Utilities      │         │   │
│  │  │                │  │                │  │                │         │   │
│  │  │• Rust Crate    │  │• Coordinator   │  │• Schema Vers.  │         │   │
│  │  │• AgilePlus WP  │  │• Evidence Gen. │  │• Migration Gen.│         │   │
│  │  │• xDD Tests     │  │• Conflict Res. │  │                │         │   │
│  │  └────────────────┘  └────────────────┘  └────────────────┘         │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                    Generated Artifacts Layer                         │   │
│  │                                                                       │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐   │   │
│  │  │ Source Code  │  │ Tests        │  │ Evidence & Artifacts     │   │   │
│  │  │              │  │              │  │                          │   │   │
│  │  │• Crates      │  │• Unit Tests  │  │• Test Logs              │   │   │
│  │  │• WP Specs    │  │• Integration │  │• Coverage Reports       │   │   │
│  │  │• Config      │  │• xDD BDD     │  │• Lint Summaries         │   │   │
│  │  └──────────────┘  └──────────────┘  └──────────────────────────┘   │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Core Design: Hexagonal Architecture

Forgecode fork follows Hexagonal Architecture with clear separation:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            Hexagonal Model                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌─────────────────┐          ┌──────────────────┐     ┌─────────────┐    │
│   │  Inbound Ports  │◄────────▶│  Forgecode Core  │◄────▶│   Adapters  │    │
│   │  (User Input)   │          │   (Generation    │     │ (Providers, │    │
│   │                 │          │    Logic)        │     │  Storage)   │    │
│   │ • CLI           │          │                  │     │             │    │
│   │ • REST API      │          │ • Template       │     │ • Providers │    │
│   │ • Dashboard UI  │          │   Engine         │     │ • File I/O  │    │
│   │ • Subagent API  │          │ • Validation     │     │ • Git ops   │    │
│   └─────────────────┘          │ • Composition    │     │ • CI/CD API │    │
│                                │                  │     │             │    │
│                                │                  │     └─────────────┘    │
│                                └──────────────────┘                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Integration Points

### 1. Provider Plugin Interface

**Location**: `src/extensions/providers/`

**Base Interface**:
```typescript
interface Provider {
  name: string;
  description: string;
  version: string;
  supportedLanguages: string[];

  generate(config: ProviderConfig): Promise<GeneratedArtifacts>;
  validate(input: any): Promise<ValidationResult>;
  preview(config: ProviderConfig): Promise<PreviewContent>;
}
```

**Implementations**:
- `PhenotypeRustCrateProvider` — Hexagonal crate scaffolding
- `AgilePlusWorkPackageProvider` — WP YAML templates with FR traceability
- `XDDTestGenerator` — Test scaffolds with #[Traces to: FR-*] markers

### 2. Subagent Coordination API

**Location**: `src/extensions/subagent-coordinator/`

**Endpoints**:
```
POST /api/extensions/coordinate-agents
  Input: { workPackages: WP[], agents: Agent[] }
  Output: { assignment: Assignment[], strategy: Strategy, conflicts: Conflict[] }

POST /api/extensions/merge-results
  Input: { results: AgentResult[], strategy: Strategy }
  Output: { merged: MergedResult, conflicts: Conflict[], report: Report }

GET /api/extensions/agent-status/:agentId
  Output: { status: Status, progress: Progress, blockers: Issue[] }
```

### 3. Evidence & Audit Trail

**Location**: `src/extensions/audit-trail/`

**Scope**:
- Capture test execution logs, coverage reports, lint summaries
- Embed git commit links, CI/CD pipeline links
- Generate evidence bundles per work package
- Gallery view in AgilePlus dashboard

**Data Model**:
```typescript
interface EvidenceBundle {
  workPackageId: string;
  timestamp: ISO8601;
  testResults: {
    passed: number;
    failed: number;
    skipped: number;
    logs: string[];
  };
  coverage: {
    lines: number;
    branches: number;
    functions: number;
    report: string; // link to HTML report
  };
  linting: {
    errors: number;
    warnings: number;
    summary: LintSummary[];
  };
  artifacts: {
    gitCommit: string;
    ciPipelineLink: string;
    artifactLinks: string[];
  };
}
```

### 4. Dashboard REST API

**Location**: `src/api/dashboard/`

**Endpoints**:
```
GET /api/providers
  → List all available providers with metadata

POST /api/providers/:providerName/generate
  Input: { config: ProviderConfig }
  Output: { jobId: string, estimatedTime: number }

GET /api/jobs/:jobId
  → Long-poll job status, streaming output

GET /api/evidence/:workPackageId
  → Retrieve evidence bundle for timeline view

GET /api/agents/status
  → Real-time status of coordinated subagents
```

## Modification Strategy

### Fork Maintenance

**Upstream Sync**: Monthly cherry-pick of bugfixes/features

**Branch Strategy**:
```
main
├── upstream/main (tracking official forgecode)
├── features/
│   ├── providers/* (custom provider implementations)
│   ├── subagent-extensions/* (multi-agent coordination)
│   └── dashboard-integration/* (AgilePlus widgets)
└── worktrees/
    ├── provider-dev/<feature>
    ├── subagent-dev/<feature>
    └── integration-test/<feature>
```

**Cherry-Pick Workflow**:
```bash
# Pull latest upstream
git fetch upstream main

# Find commits to cherry-pick (bugfixes, security patches)
git log upstream/main --since="1 month ago" --oneline

# Cherry-pick into main
git cherry-pick <commit-hash>

# Test custom extensions against new upstream version
cargo test --workspace
npm test
```

### Extension Points

**Provider API**:
- All providers inherit from `Provider` interface
- Each provider is independent (no inter-provider dependencies)
- Providers compose with core engine for templating, validation, file I/O

**Subagent Registry**:
- Fork exposes `SubagentRegistry` trait
- Agents can register capabilities (code generation, testing, audit)
- Orchestrator queries registry to assign work

**Configuration**:
- fork-specific config in `.agileplus/config.toml`
- Provider configs in `providers/<provider-name>/config.toml`
- Subagent configs in `subagent-extensions/config.toml`

## Development Workflow

### Adding a New Provider

1. **Create provider module**:
   ```bash
   mkdir src/extensions/providers/<provider-name>
   touch src/extensions/providers/<provider-name>/mod.rs
   ```

2. **Implement Provider trait**:
   ```rust
   pub struct MyProvider;

   impl Provider for MyProvider {
       async fn generate(&self, config: ProviderConfig) -> Result<Artifacts> {
           // Implementation
       }
   }
   ```

3. **Register in provider registry**:
   ```rust
   // src/extensions/providers/mod.rs
   pub mod my_provider;

   pub fn get_provider(name: &str) -> Option<Box<dyn Provider>> {
       match name {
           "my-provider" => Some(Box::new(MyProvider)),
           _ => None,
       }
   }
   ```

4. **Create tests**:
   ```rust
   #[cfg(test)]
   mod tests {
       // Traces to: FR-FORGE-NNN
       #[test]
       fn test_generate() { ... }
   }
   ```

5. **Document in kitty-specs/**:
   ```markdown
   # Provider: My Provider

   **FR Mapping**: FR-FORGE-NNN
   **Expected Output**: [example]
   **Integration**: POST /api/providers/my-provider/generate
   ```

### Adding a Subagent Extension

1. **Create extension module**:
   ```bash
   mkdir src/extensions/subagent-extensions/<extension-name>
   ```

2. **Implement SubagentExtension trait**:
   ```rust
   pub trait SubagentExtension {
       async fn execute(&self, context: ExecutionContext) -> Result<ExecutionResult>;
   }
   ```

3. **Register in subagent registry** (same pattern as providers)

4. **Document in kitty-specs/**

## Testing Strategy

### Unit Tests
- Provider behavior: `cargo test --package forgecode-providers`
- Extension logic: `cargo test --package forgecode-extensions`
- Each test annotated: `// Traces to: FR-FORGE-NNN`

### Integration Tests
- End-to-end provider pipeline
- Multi-agent coordination workflows
- Dashboard API contract tests

### Evidence Collection
- Capture test execution logs
- Generate coverage reports (via tarpaulin/coverage.py)
- Run clippy/eslint/pylint and capture summaries
- Archive to `evidence/` directory

## Configuration Management

**Fork-Level Config** (`forgecode-fork/.agileplus/config.toml`):
```toml
[fork]
name = "forgecode-fork"
version = "0.1.0"
upstream = "https://github.com/forgecode/forgecode"

[agileplus]
specs_location = "../kitty-specs/forgecode-fork/"

[providers]
enabled = ["phenotype-rust-crate", "agileplus-wp", "xdd-test"]

[subagent]
registry_url = "http://localhost:9999/registry"
coordinator_timeout_ms = 30000
```

**Provider Config** (`src/extensions/providers/<provider>/config.toml`):
```toml
[provider]
name = "phenotype-rust-crate"
description = "Hexagonal Rust crate scaffolding"

[templates]
default = "hexagonal"

[validation]
required_fields = ["crate_name", "template"]
```

## CI/CD Integration

**GitHub Actions Workflow**:
```yaml
name: Test & Evidence

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run tests
        run: cargo test --workspace
      - name: Generate evidence
        run: forgecode audit:generate --include tests,coverage,lint
      - name: Upload evidence
        uses: actions/upload-artifact@v3
        with:
          name: evidence-bundle
          path: evidence/
```

## References

- **Upstream**: https://github.com/forgecode/forgecode
- **AgilePlus**: /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
- **phenotype-infrakit**: /Users/kooshapari/CodeProjects/Phenotype/repos (Rust crates)
- **Hexagonal Architecture**: https://en.wikipedia.org/wiki/Hexagonal_architecture_(software)
- **xDD**: https://en.wikipedia.org/wiki/Behavior-driven_development
