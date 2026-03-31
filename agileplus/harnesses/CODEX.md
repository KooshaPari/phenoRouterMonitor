# AgilePlus Harness: Codex

## Installation

Add to `~/.codex/AGENTS.md` or project-level `AGENTS.md`:

```markdown
## AgilePlus Integration

You have access to the AgilePlus spec-driven development engine.

### Commands

- `agileplus specify <feature>` - Create feature specification
- `agileplus research <feature>` - Research feature feasibility
- `agileplus plan <feature>` - Generate implementation plan
- `agileplus implement <feature>` - Implement work packages
- `agileplus validate <feature>` - Validate governance compliance
- `agileplus ship <feature>` - Ship feature to main
- `agileplus cycle list` - List active cycles
- `agileplus triage <item>` - Classify incoming item

### Workflow

For all feature development, follow this workflow:

1. **Triage**: Classify the item with `agileplus triage <description>`
2. **Specify**: Create SPEC.md with `agileplus specify <feature-name>`
3. **Research**: Research feasibility with `agileplus research <feature-name>`
4. **Plan**: Generate work packages with `agileplus plan <feature-name>`
5. **Implement**: Implement with `agileplus implement <feature-name>`
6. **Validate**: Validate with `agileplus validate <feature-name>`
7. **Ship**: Ship with `agileplus ship <feature-name>`

### Environment

```bash
export PATH="$HOME/bin:$PATH"
export AGILEPLUS_HOME="/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus"
```

### xDD Methodologies

Reference: `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/docs/governance/xdd-methodologies-encyclopedia.md`

Follow these methodologies:
- TDD: Test before implementation
- BDD: Behavior-driven specs
- DDD: Domain-driven design
- SpecDD: Specification-driven development
- CQRS: Command/query responsibility segregation
- Event Sourcing: Event-driven architecture
```

## Codex Agent Config

Add to `~/.codex/agents.toml`:

```toml
[agent.agileplus]
name = "AgilePlus Agent"
description = "Spec-driven development with xDD methodologies"
command = "agileplus"
args = ["--agent"]
env = { AGILEPLUS_HOME = "/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus" }
```

## Codex RC File

Add to `~/.codexrc`:

```bash
# AgilePlus integration
export PATH="$HOME/bin:$PATH"
export AGILEPLUS_HOME="/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus"

# Aliases
alias ap="agileplus"
alias ap-spec="agileplus specify"
alias ap-plan="agileplus plan"
alias ap-ship="agileplus ship"
```
