---
audience: [sdk, agents]
---

# MCP Tools

AgilePlus exposes its capabilities as [Model Context Protocol](https://modelcontextprotocol.io/) tools, allowing AI agents to interact with the system programmatically.

## Available Tools

| Tool | Description |
|------|-------------|
| `agileplus_specify` | Create a feature specification from a description |
| `agileplus_plan` | Generate work packages for a feature |
| `agileplus_implement` | Start implementation of a work package |
| `agileplus_move_task` | Transition a work package between lanes |
| `agileplus_status` | Get kanban board status |
| `agileplus_sync` | Sync with external trackers |

## Tool Schemas

### agileplus_specify

```json
{
  "name": "agileplus_specify",
  "description": "Create a feature specification",
  "inputSchema": {
    "type": "object",
    "properties": {
      "description": {
        "type": "string",
        "description": "Natural language feature description"
      },
      "slug": {
        "type": "string",
        "description": "Kebab-case feature slug"
      }
    },
    "required": ["description"]
  }
}
```

### agileplus_move_task

```json
{
  "name": "agileplus_move_task",
  "description": "Move a work package to a new lane",
  "inputSchema": {
    "type": "object",
    "properties": {
      "wp_id": { "type": "string" },
      "to": {
        "type": "string",
        "enum": ["planned", "doing", "for_review", "done"]
      },
      "note": { "type": "string" }
    },
    "required": ["wp_id", "to"]
  }
}
```

## Configuration

Add AgilePlus as an MCP server in your agent config:

```json
{
  "mcpServers": {
    "agileplus": {
      "command": "agileplus",
      "args": ["mcp", "serve"],
      "env": {
        "AGILEPLUS_PROJECT": "/path/to/project"
      }
    }
  }
}
```
