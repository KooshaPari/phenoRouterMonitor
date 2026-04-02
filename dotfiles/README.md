# dotfiles — Governance and Hook Templates

## Overview

- **Type**: Template Library
- **Purpose**: Consolidated governance and configuration templates for the Phenotype ecosystem
- **Contents**: Pre-commit hooks, CLAUDE.md base templates, quality gate patterns

## Structure

```
dotfiles/
├── governance/          # Governance templates
│   ├── CLAUDE.base.md  # Base template for all CLAUDE.md files
│   └── README.md       # Governance template documentation
├── hooks/              # Pre-commit hook configurations
│   ├── .pre-commit-config.base.yaml
│   └── README.md       # Hook template documentation
├── README.md           # This file
├── AGENTS.md           # Agent guidance
├── .gitignore          # Git ignore patterns
└── .agileplus/        # AgilePlus worklog
```

## Usage

### CLAUDE.md Base Template

Copy `governance/CLAUDE.base.md` to any project as `CLAUDE.md`.

### Pre-commit Hooks

Copy `hooks/.pre-commit-config.base.yaml` to any project.

## Maintenance

This template library is maintained by the Phenotype Governance Team.
