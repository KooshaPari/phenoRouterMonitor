# AgilePlus Harness: Claude Code

## Installation

Add to your Claude Code configuration (`~/.claude/projects.d/agileplus.json` or project-level):

```json
{
  "name": "AgilePlus",
  "description": "Spec-driven development engine with xDD methodologies",
  "command": "agileplus",
  "args": ["--project", "{PROJECT_PATH}"],
  "env": {
    "AGILEPLUS_HOME": "/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus",
    "AGILEPLUS_MCP": "true"
  }
}
```

## Commands

| Command | Description |
|---------|-------------|
| `agileplus specify <feature>` | Create feature specification |
| `agileplus research <feature>` | Research feature feasibility |
| `agileplus plan <feature>` | Generate implementation plan |
| `agileplus implement <feature>` | Implement work packages |
| `agileplus validate <feature>` | Validate governance compliance |
| `agileplus ship <feature>` | Ship feature to main |
| `agileplus cycle list` | List active cycles |
| `agileplus triage <item>` | Classify incoming item |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `AGILEPLUS_HOME` | `~/.agileplus` | AgilePlus data directory |
| `AGILEPLUS_MCP` | `false` | Enable MCP mode |
| `AGILEPLUS_CYCLE` | `current` | Active cycle ID |
| `AGILEPLUS_BRANCH_PREFIX` | `feat/` | Work package branch prefix |

## MCP Server

To use AgilePlus as an MCP server:

```json
{
  "mcpServers": {
    "agileplus": {
      "command": "npx",
      "args": ["-y", "@agileplus/mcp"]
    }
  }
}
```

## Skills Integration

Add to Claude Code `AGENTS.md`:

```markdown
## AgilePlus Workflow

Follow the AgilePlus spec-driven development workflow:

1. **Triage**: Classify incoming items with `agileplus triage`
2. **Specify**: Create feature specs with `agileplus specify`
3. **Research**: Research feasibility with `agileplus research`
4. **Plan**: Generate work packages with `agileplus plan`
5. **Implement**: Implement with `agileplus implement`
6. **Validate**: Validate with `agileplus validate`
7. **Ship**: Ship with `agileplus ship`

Reference: docs/governance/xdd-methodologies-encyclopedia.md
```

## Quick Start

```bash
# Install CLI
cp target/debug/agileplus ~/bin/
export PATH="$HOME/bin:$PATH"

# Create a feature
agileplus specify my-feature

# Research it
agileplus research my-feature

# Plan implementation
agileplus plan my-feature

# Validate before shipping
agileplus validate my-feature

# Ship to main
agileplus ship my-feature
```
