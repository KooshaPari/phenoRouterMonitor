# Phenotype Registry

> Central registry for all Phenotype ecosystem repositories

## Overview

The Phenotype Registry catalogs all repositories in the `repos` workspace, providing:
- **Metadata tracking** - Languages, status, documentation completeness
- **Project discovery** - Find repos by type, language, or status
- **Health monitoring** - Identify repos needing attention

## Registry Structure

```
phenotype-registry/
├── README.md              # This file
├── registry.json          # Master registry (all repos)
├── summary.json           # Aggregated statistics
├── projects/              # Individual project metadata
│   ├── AgilePlus.json
│   ├── heliosCLI.json
│   └── ... (101 files)
└── docs/
    └── index.md           # Detailed documentation
```

## Quick Stats

| Metric | Count |
|--------|-------|
| **Total Repositories** | 101 |
| **Complete** (all docs) | 73 |
| **Partial** (missing 1 doc) | 5 |
| **Minimal** (1-2 docs) | 7 |
| **Bare** (no docs) | 16 |

### By Language

| Language | Count |
|----------|-------|
| Rust | 34 |
| Python | 31 |
| TypeScript | 16 |
| Unknown | 11 |
| Go | 8 |
| Zig | 1 |

### By Type

| Type | Count |
|------|-------|
| Core | 71 |
| Tool | 11 |
| Utility | 10 |
| Template | 5 |
| App | 3 |
| Plugin | 1 |

## Using the Registry

### Find Repos by Language

```bash
# Get all Rust repos
jq -r '.repos[] | select(.primary_language == "rust") | .name' registry.json

# Get all Python repos
jq -r '.repos[] | select(.primary_language == "python") | .name' registry.json
```

### Find Repos Needing Attention

```bash
# Find repos without README
jq -r '.repos[] | select(.has_readme == false) | .name' registry.json

# Find bare repos (no docs)
jq -r '.repos[] | select(.status == "bare") | .name' registry.json
```

### Get Project Metadata

```bash
# Read individual project file
cat projects/AgilePlus.json | jq .
```

## Registry Schema

### Repository Entry

```json
{
  "name": "repo-name",
  "languages": ["rust", "typescript"],
  "primary_language": "rust",
  "status": "complete|partial|minimal|bare",
  "has_readme": true,
  "has_spec": true,
  "has_plan": true,
  "type": "core|utility|tool|template|app|plugin",
  "path": "repos/repo-name"
}
```

### Status Definitions

| Status | Criteria |
|--------|----------|
| `complete` | Has README.md + SPEC.md + PLAN.md |
| `partial` | Missing 1 document |
| `minimal` | 1-2 documents present |
| `bare` | No documentation |

### Type Definitions

| Type | Description | Examples |
|------|-------------|----------|
| `core` | Core libraries and APIs | phenotype-* repos |
| `utility` | Support/utility repos | artifacts, docs |
| `tool` | CLI tools and kits | heliosCLI, KodeVibeGo |
| `template` | Project templates | template-* repos |
| `app` | Applications | heliosApp, cloud |
| `plugin` | Plugin systems | agileplus-plugin-* |

## Maintenance

### Regenerate Registry

```bash
# Run the generation script
python3 /tmp/generate_registry.py
```

### Add New Repository

When a new repo is added to `repos/`:
1. Re-scan will automatically pick it up
2. Individual project file will be created
3. Registry statistics will be updated

## Repos Needing Attention

The following 16 repositories have `bare` status (no documentation):

| Repository | Language | Type |
|------------|----------|------|
| artifacts | unknown | utility |
| crates | rust | core |
| docs | typescript | utility |
| koosha-portfolio | unknown | utility |
| phenGovernance | unknown | core |
| phenoProc | rust | core |
| phenoRouterMonitor | unknown | utility |
| phenoVCS | rust | core |
| phenotype-hub | unknown | core |
| platforms | rust | utility |
| repos | unknown | utility |
| scripts | python | utility |
| src | rust | utility |
| tests | python | utility |
| tooling | unknown | utility |
| tools | python | utility |

## Related Documentation

- [PHENOTYPE_INDEX.md](../PHENOTYPE_INDEX.md) - Master project index
- [PROJECT_CLASSIFICATION.md](../PROJECT_CLASSIFICATION.md) - Classification criteria
- [ADR_REGISTRY.md](../ADR_REGISTRY.md) - Architecture decisions

---

*Registry generated: 2026-04-03*  
*Version: 1.0.0*
