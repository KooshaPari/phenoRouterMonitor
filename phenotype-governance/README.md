# Phenotype Governance

Standardized templates and DRY propagation system for Phenotype ecosystem projects.

## Overview

This repository contains standardized configuration files, templates, and automation scripts to ensure consistency across all homegrown projects in the Phenotype ecosystem.

## Philosophy

**DRY (Don't Repeat Yourself)**: Common configuration patterns should be defined once and propagated to all relevant projects, not copy-pasted.

## Repository Structure

```
phenotype-governance/
├── templates/
│   ├── rust/           # Rust-specific templates
│   │   ├── codecov.yml.template
│   │   ├── rustfmt.toml.template
│   │   ├── .clippy.toml.template
│   │   └── Taskfile.yml.template
│   ├── ci/             # CI/CD templates
│   │   └── dependabot.yml.template
│   └── docs/           # Documentation templates
│       └── CONTRIBUTING.md.template
├── scripts/
│   └── propagate-templates.sh  # DRY propagation script
└── README.md
```

## Available Templates

### Rust Projects

| Template | Destination | Purpose |
|----------|-------------|---------|
| `codecov.yml.template` | `codecov.yml` | Code coverage configuration |
| `rustfmt.toml.template` | `rustfmt.toml` | Code formatting rules |
| `.clippy.toml.template` | `.clippy.toml` | Linting configuration |
| `Taskfile.yml.template` | `Taskfile.yml` | Task automation |

### CI/CD

| Template | Destination | Purpose |
|----------|-------------|---------|
| `dependabot.yml.template` | `.github/dependabot.yml` | Dependency updates |

### Documentation

| Template | Destination | Purpose |
|----------|-------------|---------|
| `CONTRIBUTING.md.template` | `CONTRIBUTING.md` | Contribution guidelines |

## Usage

### Propagate Templates to Projects

```bash
# Setup a specific project
cd phenotype-governance
./scripts/propagate-templates.sh Httpora

# Setup all P0 priority projects
./scripts/propagate-templates.sh all

# Dry run (see what would be changed)
./scripts/propagate-templates.sh -n all

# List projects needing attention
./scripts/propagate-templates.sh -l
```

### Manual Template Application

```bash
# Copy a specific template
cp templates/rust/codecov.yml.template /path/to/project/codecov.yml
cp templates/rust/rustfmt.toml.template /path/to/project/rustfmt.toml
cp templates/rust/.clippy.toml.template /path/to/project/.clippy.toml
```

## Project Priority Levels

### P0 - Critical (Immediate Attention Required)

Projects missing 5+ infrastructure components:
- **Httpora**: Missing tests, clippy, rustfmt, codecov, dependabot
- **Queris**: Missing tests, clippy, rustfmt, codecov, dependabot
- **Tossy**: Missing CONTRIBUTING, clippy, rustfmt, codecov, dependabot

### P1 - High Priority

Projects missing 3-4 infrastructure components:
- Profila, Cursora, Datamold, Docuverse, Duple, Flagward, Flowra, Guardis
- HexaGo, HexaType, Hexacore

### P2 - Medium Priority

Projects missing 1-2 infrastructure components.

## Infrastructure Checklist

Every project should have:

- [ ] `.github/workflows/` - CI/CD workflows
- [ ] `.github/dependabot.yml` - Automated dependency updates
- [ ] `codecov.yml` - Code coverage configuration
- [ ] `rustfmt.toml` - Code formatting rules
- [ ] `.clippy.toml` - Linting rules
- [ ] `CONTRIBUTING.md` - Contribution guidelines
- [ ] `README.md` - Project documentation
- [ ] `tests/` - Test infrastructure
- [ ] `Taskfile.yml` - Task automation
- [ ] `.pre-commit-config.yaml` - Pre-commit hooks

## Maintenance

### Adding New Templates

1. Create template in appropriate `templates/` subdirectory
2. Add entry to `scripts/propagate-templates.sh` arrays
3. Update this README
4. Test with dry run: `./scripts/propagate-templates.sh -n all`

### Updating Existing Templates

1. Modify the template file
2. Propagate to projects: `./scripts/propagate-templates.sh all`
3. Review changes before committing

## Contributing

When adding templates:
- Include clear comments explaining configuration options
- Provide usage examples in the template header
- Follow existing naming conventions
- Test templates before committing

## License

These templates are part of the Phenotype ecosystem and follow the same licensing as the projects they serve.
