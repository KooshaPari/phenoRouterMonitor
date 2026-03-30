# AgilePlus Harness: Cursor

## Installation

Add to Cursor's `~/.cursor/mcp.json` or project-level `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "agileplus": {
      "command": "npx",
      "args": ["-y", "@agileplus/mcp"],
      "env": {
        "AGILEPLUS_HOME": "/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus"
      }
    }
  }
}
```

## Alternative: Direct CLI

Add to Cursor's terminal configuration:

```bash
# In ~/.zshrc or ~/.bashrc
export PATH="$HOME/bin:$PATH"
export AGILEPLUS_HOME="/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus"
alias ap="agileplus"
```

## Agent Mode

Enable in Cursor settings:

1. Open Cursor Settings → AI → Agents
2. Enable "Agent Mode"
3. Configure custom commands for AgilePlus

## Custom Commands

Add to `.cursor/commands.json`:

```json
{
  "specify": "agileplus specify {selection}",
  "research": "agileplus research {selection}",
  "plan": "agileplus plan {selection}",
  "validate": "agileplus validate {selection}",
  "ship": "agileplus ship {selection}"
}
```

## Prompt Integration

Add to `.cursor/system_prompt.md`:

```markdown
# AgilePlus Integration

You have access to the AgilePlus spec-driven development engine.
Use the following workflow for all feature development:

1. Triage: `agileplus triage <description>`
2. Specify: `agileplus specify <feature-name>`
3. Research: `agileplus research <feature-name>`
4. Plan: `agileplus plan <feature-name>`
5. Implement: `agileplus implement <feature-name>`
6. Validate: `agileplus validate <feature-name>`
7. Ship: `agileplus ship <feature-name>`

Reference: AgilePlus/docs/governance/xdd-methodologies-encyclopedia.md
```

## Keyboard Shortcuts

Add to Cursor keybindings:

```json
{
  "key": "cmd+shift+a",
  "command": "agileplus specify",
  "when": "editorTextFocus"
}
```
