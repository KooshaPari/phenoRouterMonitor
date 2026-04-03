# Phenotype Registry Documentation

This directory contains detailed documentation for the Phenotype Registry system.

## Contents

### Overview

The Phenotype Registry is the central catalog for all repositories in the Phenotype ecosystem. It provides:

1. **Automated Discovery** - Scans all git repositories automatically
2. **Metadata Extraction** - Detects languages, documentation status, project types
3. **Health Tracking** - Identifies repos that need documentation improvements
4. **Query Interface** - JSON-based querying via `jq` or programmatic access

## Registry Files

### Main Registry (`registry.json`)

Contains all repository entries with metadata:

```json
{
  "version": "1.0.0",
  "generated_at": "2026-04-03T08:37:33Z",
  "total_repos": 101,
  "repos": [...]
}
```

### Summary Statistics (`summary.json`)

Aggregated metrics for quick analysis:

```json
{
  "total_repos": 101,
  "by_language": { "rust": 34, "python": 31, ... },
  "by_status": { "complete": 73, "bare": 16, ... },
  "by_type": { "core": 71, "utility": 10, ... }
}
```

### Project Files (`projects/*.json`)

Individual metadata files for each repository, useful for:
- Tooling that needs single-file access
- CI/CD integration
- External registry consumers

## Language Detection

The registry detects languages based on file presence:

| Language | Detected By |
|----------|-------------|
| Rust | `Cargo.toml` |
| Python | `pyproject.toml`, `setup.py` |
| Go | `go.mod` |
| TypeScript | `package.json` |
| Zig | `build.zig` |
| PHP | `composer.json` |
| Elixir | `mix.exs` |
| Java/Kotlin | `pom.xml`, `build.gradle` |

## Status Calculation

Repository status is determined by documentation completeness:

```
complete  = README.md + SPEC.md + PLAN.md
partial   = Missing 1 document
minimal   = 1-2 documents
bare      = No documents
```

## Type Classification

Type is inferred from repository name patterns:

| Pattern | Type |
|---------|------|
| `template-*` | template |
| `*cli*`, `*tool*`, `*kit*`, `*cmd*` | tool |
| `*lib*`, `*core*`, `*shared*`, `*api*` | core |
| `*app*`, `*ui*`, `*web*`, `*portal*` | app |
| `*plugin*`, `*extension*` | plugin |
| No language detected | utility |
| Default | core |

## Query Examples

### Using jq

```bash
# Count repos by language
jq '.summary.by_language' summary.json

# List all TypeScript repos
jq -r '.repos[] | select(.primary_language == "typescript") | .name' registry.json

# Find repos missing README
jq -r '.repos[] | select(.has_readme == false) | .name' registry.json

# Get repos needing attention (not complete)
jq -r '.repos[] | select(.status != "complete") | .name' registry.json

# Multi-language repos
jq -r '.repos[] | select(.languages | length > 1) | "\(.name): \(.languages)"' registry.json
```

### Using Python

```python
import json

with open('registry.json') as f:
    registry = json.load(f)

# Get all Rust repos
rust_repos = [r for r in registry['repos'] if r['primary_language'] == 'rust']

# Get bare repos needing docs
bare_repos = [r for r in registry['repos'] if r['status'] == 'bare']
```

## Integration

### CI/CD Integration

Check documentation completeness in CI:

```yaml
- name: Check Registry Status
  run: |
    bare_count=$(jq '.summary.by_status.bare // 0' phenotype-registry/summary.json)
    if [ "$bare_count" -gt 0 ]; then
      echo "Warning: $bare_count repos without documentation"
      jq -r '.repos[] | select(.status == "bare") | .name' phenotype-registry/registry.json
    fi
```

### Tooling Integration

Projects can reference the registry for discovery:

```bash
# Find all plugins
jq -r '.repos[] | select(.type == "plugin") | .name' registry.json

# Find all templates
jq -r '.repos[] | select(.type == "template") | .name' registry.json
```

## Regeneration Script

The registry can be regenerated using:

```python
import os
import json
from pathlib import Path

REPOS_DIR = "/Users/kooshapari/CodeProjects/Phenotype/repos"
REGISTRY_DIR = "/Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-registry"

def detect_language(repo_path):
    languages = []
    if os.path.exists(f"{repo_path}/Cargo.toml"): languages.append("rust")
    if os.path.exists(f"{repo_path}/go.mod"): languages.append("go")
    if os.path.exists(f"{repo_path}/pyproject.toml"): languages.append("python")
    if os.path.exists(f"{repo_path}/package.json"): languages.append("typescript")
    if os.path.exists(f"{repo_path}/build.zig"): languages.append("zig")
    return languages if languages else ['unknown']

# See full script in /tmp/generate_registry.py
```

## Future Enhancements

Potential improvements to the registry:

1. **Dependency Tracking** - Map inter-repo dependencies
2. **Activity Metrics** - Track commit recency, PR activity
3. **Test Coverage** - Include test pass/fail status
4. **Version Tracking** - Track published versions for each repo
5. **Ownership** - Map repos to teams/owners
6. **Health Score** - Composite health metric

---

For the main registry documentation, see [../README.md](../README.md).
