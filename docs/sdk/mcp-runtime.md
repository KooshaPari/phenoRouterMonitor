---
audience: [sdk, agents]
---

# MCP Runtime, Errors, and Supporting Catalogs

This companion document covers the MCP server configuration, invocation
examples, standardized error responses, the additional tool catalog, and the
full inventory for the AgilePlus MCP server.

## MCP Server Configuration

### Claude Code Integration

```json
{
  "mcpServers": {
    "agileplus": {
      "command": "agileplus",
      "args": ["mcp", "serve"],
      "env": {
        "AGILEPLUS_PROJECT": "/path/to/project",
        "AGILEPLUS_GRPC_ADDRESS": "127.0.0.1:50051"
      }
    }
  }
}
```

### Cursor Integration

```json
{
  "tools": [
    {
      "name": "agileplus",
      "enabled": true,
      "command": "agileplus mcp serve",
      "timeout": 30,
      "env": {
        "AGILEPLUS_PROJECT": "${workspaceFolder}"
      }
    }
  ]
}
```

## Tool Invocation Examples

### Python Example

```python
import subprocess
import json

# Call the MCP tool via stdio
proc = subprocess.Popen(
    ["agileplus", "mcp", "serve"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    env={"AGILEPLUS_PROJECT": "/path/to/project"}
)

# Send JSON-RPC request
request = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
        "name": "agileplus_get_feature",
        "arguments": {"slug": "001-login"}
    }
}

proc.stdin.write(json.dumps(request).encode() + b"\n")
response = json.loads(proc.stdout.readline().decode())
print(response)
```

### Bash Example

```bash
export AGILEPLUS_PROJECT="/path/to/project"

# Call a tool via JSON-RPC over stdio
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"agileplus_list_features"}}' \
  | agileplus mcp serve
```

## Error Responses

MCP tools return standardized error responses:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32600,
    "message": "Invalid Request",
    "data": {
      "reason": "Feature not found",
      "slug": "nonexistent"
    }
  }
}
```

Common error codes:
- `-32600`: Invalid request (bad tool name, missing args)
- `-32601`: Method not found (unknown command)
- `-32602`: Invalid params (missing required field)
- `-32700`: Parse error (malformed JSON)
- `-1`: Internal error (storage/VCS failure)

## Additional Tool Catalog

### Transition Tools

#### `agileplus_transition_feature`

Trigger a feature state transition (subject to governance checks).

```json
{
  "name": "agileplus_transition_feature",
  "description": "Transition a feature to a new state",
  "inputSchema": {
    "type": "object",
    "properties": {
      "feature_slug": { "type": "string" },
      "target_state": {
        "type": "string",
        "enum": ["Specified", "Researched", "Planned", "Implementing", "Validated", "Shipped", "Cancelled"]
      },
      "actor": {
        "type": "string",
        "description": "Actor identifier: 'human:alice' or 'agent:claude-code'"
      },
      "evidence": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "fr_id": { "type": "string" },
            "evidence_type": { "type": "string" },
            "artifact_path": { "type": "string" }
          }
        }
      }
    },
    "required": ["feature_slug", "target_state", "actor"]
  }
}
```

Returns:
```json
{
  "success": true,
  "new_state": "Planned",
  "audit_entry_id": 5,
  "audit_hash": "sha256:0x9c0d...",
  "governance_checks_passed": 3,
  "timestamp": "2026-03-01T16:00:00Z"
}
```

#### `agileplus_transition_work_package`

Transition a work package state.

```json
{
  "name": "agileplus_transition_work_package",
  "description": "Transition a work package state",
  "inputSchema": {
    "type": "object",
    "properties": {
      "feature_slug": { "type": "string" },
      "wp_id": { "type": "string" },
      "target_state": {
        "type": "string",
        "enum": ["Planned", "Doing", "ForReview", "Done", "Blocked"]
      },
      "actor": { "type": "string" },
      "block_reason": {
        "type": "string",
        "description": "Required when target_state is Blocked"
      }
    },
    "required": ["feature_slug", "wp_id", "target_state", "actor"]
  }
}
```

### Artifact Tools

#### `agileplus_read_artifact`

Read a text artifact from the feature's artifact store.

```json
{
  "name": "agileplus_read_artifact",
  "description": "Read a feature artifact (spec.md, plan.md, research.md, etc.)",
  "inputSchema": {
    "type": "object",
    "properties": {
      "feature_slug": { "type": "string" },
      "relative_path": {
        "type": "string",
        "description": "Path relative to feature dir: 'spec.md', 'WP01/prompt.md'"
      }
    },
    "required": ["feature_slug", "relative_path"]
  }
}
```

Returns:
```json
{
  "content": "# User Authentication\n\n## Functional Requirements...",
  "size_bytes": 2147,
  "sha256": "0x3f7e..."
}
```

#### `agileplus_write_artifact`

Write or update a feature artifact.

```json
{
  "name": "agileplus_write_artifact",
  "description": "Write content to a feature artifact",
  "inputSchema": {
    "type": "object",
    "properties": {
      "feature_slug": { "type": "string" },
      "relative_path": { "type": "string" },
      "content": { "type": "string" },
      "commit_message": {
        "type": "string",
        "description": "Git commit message for this artifact write"
      }
    },
    "required": ["feature_slug", "relative_path", "content"]
  }
}
```

### Sync Tools

#### `agileplus_sync_push`

Push AgilePlus state to external trackers.

```json
{
  "name": "agileplus_sync_push",
  "description": "Push feature/WP state to Plane.so or GitHub",
  "inputSchema": {
    "type": "object",
    "properties": {
      "feature_slug": {
        "type": "string",
        "description": "Optional: sync only this feature. Omit for all."
      },
      "platform": {
        "type": "string",
        "enum": ["plane", "github", "all"],
        "default": "all"
      },
      "dry_run": {
        "type": "boolean",
        "description": "If true, simulate without making changes"
      }
    }
  }
}
```

Returns:
```json
{
  "pushed_features": 3,
  "pushed_wps": 12,
  "created_issues": 5,
  "updated_issues": 7,
  "conflicts_detected": 0,
  "duration_ms": 1240
}
```

#### `agileplus_sync_status`

Get current sync status and last sync time.

```json
{
  "name": "agileplus_sync_status",
  "description": "Get sync status with external trackers",
  "inputSchema": {
    "type": "object",
    "properties": {
      "platform": {
        "type": "string",
        "enum": ["plane", "github", "all"]
      }
    }
  }
}
```

Returns:
```json
{
  "platforms": {
    "plane": {
      "connected": true,
      "last_sync": "2026-03-01T15:55:00Z",
      "pending_pushes": 2,
      "pending_pulls": 0
    },
    "github": {
      "connected": true,
      "last_sync": "2026-03-01T15:50:00Z",
      "pending_pushes": 0,
      "pending_pulls": 1
    }
  }
}
```

## Full Tool Inventory

| Tool | Category | Description |
|------|----------|-------------|
| `agileplus_get_feature` | Features | Get feature details |
| `agileplus_list_features` | Features | List features, filtered by state |
| `agileplus_transition_feature` | Features | Trigger state transition |
| `agileplus_list_work_packages` | WPs | List WPs for a feature |
| `agileplus_get_work_package_status` | WPs | Get WP detail |
| `agileplus_transition_work_package` | WPs | Transition WP state |
| `agileplus_check_governance_gate` | Governance | Check if transition is allowed |
| `agileplus_get_audit_trail` | Audit | Get immutable audit log |
| `agileplus_verify_audit_chain` | Audit | Verify hash chain integrity |
| `agileplus_read_artifact` | Artifacts | Read spec, plan, etc. |
| `agileplus_write_artifact` | Artifacts | Write artifact content |
| `agileplus_sync_push` | Sync | Push to external trackers |
| `agileplus_sync_pull` | Sync | Pull from external trackers |
| `agileplus_sync_status` | Sync | Get sync health |
| `agileplus_dispatch_command` | Commands | Execute agent subcommands |

## Next Steps

- [Storage Port](storage-port.md) — StoragePort API reference
- [VCS Port](vcs-port.md) — VcsPort API reference
- [Prompt Format](../agents/prompt-format.md) — How agents use MCP tools
- [Agent Integration Example](../examples/agent-integration.md) — End-to-end walkthrough
- [Environment Variables](../reference/env-vars.md) — MCP server configuration
