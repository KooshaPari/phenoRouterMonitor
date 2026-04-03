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
├── registry.json           # Master registry (all repos)
├── summary.json            # Aggregated statistics
├── projects/               # Individual project metadata
│   ├── phenotype-infrakit.json
│   └── ... (121 files)
└── docs/
    └── index.md           # This documentation
```

## Quick Stats

| Metric | Count |
|--------|-------|
| **Total Repositories** | 121 |
| **Complete** (all docs) | 93 |
| **Partial** (missing 1 doc) | 5 |
| **Minimal** (1-2 docs) | 4 |
| **Bare** (no docs) | 19 |

### By Language

| Language | Count |
|----------|-------|
| Rust | 39 |
| Python | 33 |
| TypeScript | 23 |
| Go | 12 |
| Unknown | 14 |

### By Type

| Type | Count |
|------|-------|
| Core | 79 |
| Utility | 13 |
| Tool | 12 |
| Template | 11 |
| App | 3 |
| Plugin | 3 |

## Using the Registry

### Find Repos by Language

```bash
# Get all Rust repos
jq '.repos[] | select(.primary_language == "rust") | .name' registry.json

# Get all Python repos
jq '.repos[] | select(.primary_language == "python") | .name' registry.json
```

### Find Repos Needing Attention

```bash
# Find repos without README
jq '.repos[] | select(.has_readme == false) | .name' registry.json

# Find bare repos (no docs)
jq '.repos[] | select(.status == "bare") | .name' registry.json
```

### Get Project Metadata

```bash
# Read individual project file
cat projects/phenotype-infrakit.json | jq .
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
# Re-scan all repos and regenerate
python3 scripts/regenerate_registry.py
```

### Add New Repository

When a new repo is added to `repos/`:
1. Re-scan will automatically pick it up
2. Individual project file will be created
3. Registry statistics will be updated

## Related Documentation

- [PHENOTYPE_INDEX.md](../PHENOTYPE_INDEX.md) - Master project index
- [PROJECT_CLASSIFICATION.md](../PROJECT_CLASSIFICATION.md) - Classification criteria
- [ADR_REGISTRY.md](../ADR_REGISTRY.md) - Architecture decisions

---

*Registry generated: 2025-04-03*
*Version: 1.0.0*
