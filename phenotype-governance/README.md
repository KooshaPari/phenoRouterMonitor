# Phenotype Governance

> Policy-as-code governance engine for the Phenotype ecosystem

## Overview

Phenotype Governance provides a policy-as-code system for managing organizational standards, compliance requirements, and automated enforcement across the Phenotype ecosystem.

## Features

- **Policy Definition**: Define policies in TOML, YAML, or JSON
- **Policy Evaluation**: Real-time evaluation with context awareness
- **Enforcement Modes**: Audit, warn, or block
- **Compliance Tracking**: Continuous compliance monitoring
- **Audit Logging**: Complete audit trail for all policy decisions

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       Governance Engine Architecture                         │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      Policy Definitions                                │   │
│  │                                                                      │   │
│  │   ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐         │   │
│  │   │ Security │ │  Coding  │ │  Testing │ │  Deploy  │         │   │
│  │   │ Policies │ │ Standards│ │ Standards│ │  Policies│         │   │
│  │   └──────────┘ └──────────┘ └──────────┘ └──────────┘         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                       Policy Engine                                    │   │
│  │                                                                      │   │
│  │   ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐         │   │
│  │   │  Parser  │ │Validator │ │Evaluator │ │ Enforcer │         │   │
│  │   └──────────┘ └──────────┘ └──────────┘ └──────────┘         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                       Compliance Layer                                 │   │
│  │                                                                      │   │
│  │   ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐         │   │
│  │   │  Audit   │ │   Warn   │ │   Block  │ │  Report  │         │   │
│  │   └──────────┘ └──────────┘ └──────────┘ └──────────┘         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Policy Types

| Category | Description | Examples |
|----------|-------------|----------|
| Security | Security-related policies | No secrets in code, MFA required |
| Quality | Code quality standards | Test coverage >80%, No TODOs |
| Process | Workflow policies | PR requires review, CI must pass |
| Compliance | Regulatory compliance | GDPR, SOC2 requirements |

## Quick Start

### Define a Policy

```toml
# .phenotype/policies/security.toml
[policy]
id = "no-secrets-in-code"
name = "No Secrets in Code"
description = "Prevent secrets from being committed to repository"
enforcement = "block"  # audit | warn | block
severity = "critical"

[[rules]]
name = "detect-api-keys"
pattern = "[a-zA-Z0-9]{32,}"  # Regex pattern
message = "Potential API key detected"

[[rules]]
name = "detect-passwords"
pattern = "(?i)(password|passwd|pwd)\s*=\s*['\"][^'\"]+['\"]"
message = "Hardcoded password detected"
```

### Evaluate Policy

```bash
# Evaluate against current directory
pheno-governance check

# Evaluate specific file
pheno-governance check --file src/main.rs

# Evaluate with specific policy
pheno-governance check --policy security

# Generate compliance report
pheno-governance report --format json
```

### Programmatic Usage

```rust
use phenotype_governance::{PolicyEngine, Policy, EnforcementLevel};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let engine = PolicyEngine::new();
    
    // Load policies
    engine.load_policies_from_dir("./policies").await?;
    
    // Evaluate file
    let result = engine.evaluate_file("./src/main.rs").await?;
    
    for violation in result.violations {
        println!("{}: {}", violation.rule, violation.message);
    }
    
    Ok(())
}
```

## Policy Schema

```rust
pub struct Policy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: PolicyCategory,
    pub enforcement: EnforcementLevel,
    pub severity: Severity,
    pub rules: Vec<Rule>,
}

pub enum EnforcementLevel {
    Audit,   // Log only
    Warn,    // Warning but allow
    Block,   // Prevent action
}

pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

pub struct Rule {
    pub name: String,
    pub description: String,
    pub condition: Condition,
    pub message: String,
}
```

## Integration

### Pre-commit Hook

```yaml
# .pre-commit-hooks.yaml
- repo: local
  hooks:
    - id: phenotype-governance
      name: Phenotype Governance Check
      entry: pheno-governance check
      language: system
      pass_filenames: true
```

### CI/CD Integration

```yaml
# .github/workflows/governance.yml
name: Governance Check
on: [push, pull_request]

jobs:
  governance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run Governance Check
        run: |
          cargo install pheno-governance
          pheno-governance check --fail-on violation
```

## Configuration

```toml
# .phenotype/governance.toml
[governance]
strict_mode = false
fail_on_warning = false
policy_dirs = ["./policies", "~/.config/phenotype/policies"]

[enforcement]
default = "warn"
security = "block"
compliance = "block"

[output]
format = "table"  # table | json | yaml
quiet = false
color = true
```

## Documentation

- [Specification](SPEC.md) - Detailed technical specification
- [Policy Reference](docs/policies.md) - Built-in policy reference
- [Contributing](CONTRIBUTING.md) - Development guidelines

## License

MIT License - see [LICENSE](LICENSE) for details.
