---
audience: [sdk, agents]
---

# MCP Tools Catalog

AgilePlus exposes capabilities as [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) tools, implemented in the `agileplus-mcp` server. This allows AI agents to interact with the entire feature lifecycle programmatically.

## Tool Categories

### Feature Specification Tools

#### `agileplus_get_feature`

Retrieve full details of a feature.

```json
{
  "name": "agileplus_get_feature",
  "description": "Get feature details by slug",
  "inputSchema": {
    "type": "object",
    "properties": {
      "slug": {
        "type": "string",
        "description": "Feature slug (e.g., '001-user-login')"
      }
    },
    "required": ["slug"]
  }
}
```

Returns:
```json
{
  "id": 1,
  "slug": "001-user-login",
  "friendly_name": "User Login System",
  "state": "IMPLEMENT",
  "target_branch": "main",
  "created_at": "2025-01-15T10:30:00Z",
  "wp_count": 3,
  "wp_done": 1
}
```

#### `agileplus_list_features`

List all features, optionally filtered by state.

```json
{
  "name": "agileplus_list_features",
  "description": "List all features",
  "inputSchema": {
    "type": "object",
    "properties": {
      "state_filter": {
        "type": "string",
        "enum": ["SPECIFY", "PLAN", "IMPLEMENT", "REVIEW", "DONE"],
        "description": "Optional state filter"
      }
    }
  }
}
```

### Work Package Tools

#### `agileplus_list_work_packages`

List all work packages for a feature.

```json
{
  "name": "agileplus_list_work_packages",
  "description": "List work packages for a feature",
  "inputSchema": {
    "type": "object",
    "properties": {
      "feature_slug": {
        "type": "string",
        "description": "Feature slug"
      },
      "state_filter": {
        "type": "string",
        "enum": ["PLANNED", "DOING", "FOR_REVIEW", "DONE"],
        "description": "Optional state filter"
      }
    },
    "required": ["feature_slug"]
  }
}
```

Returns array of:
```json
{
  "id": 5,
  "title": "Implement login form",
  "state": "DOING",
  "sequence": 1,
  "agent_id": "claude-code",
  "pr_url": "https://github.com/org/repo/pull/42",
  "pr_state": "draft",
  "depends_on": [],
  "file_scope": ["src/auth/login.rs", "src/auth/session.rs"]
}
```

#### `agileplus_get_work_package_status`

Get detailed status of a single work package.

```json
{
  "name": "agileplus_get_work_package_status",
  "description": "Get work package status and metadata",
  "inputSchema": {
    "type": "object",
    "properties": {
      "feature_slug": { "type": "string" },
      "wp_sequence": { "type": "integer", "description": "WP number (1, 2, 3...)" }
    },
    "required": ["feature_slug", "wp_sequence"]
  }
}
```

### Governance & Audit Tools

#### `agileplus_check_governance_gate`

Validate whether a feature can transition to a new state.

```json
{
  "name": "agileplus_check_governance_gate",
  "description": "Check if a feature can transition to a new state",
  "inputSchema": {
    "type": "object",
    "properties": {
      "feature_slug": { "type": "string" },
      "transition": {
        "type": "string",
        "enum": ["PLAN", "IMPLEMENT", "REVIEW", "DONE"],
        "description": "Target state"
      }
    },
    "required": ["feature_slug", "transition"]
  }
}
```

Returns:
```json
{
  "passed": false,
  "violations": [
    {
      "rule_id": "FR-REVIEW-001",
      "message": "At least one approved review required",
      "remediation": "Request code review from a reviewer"
    }
  ]
}
```

#### `agileplus_get_audit_trail`

Retrieve the audit trail for a feature (immutable changelog).

```json
{
  "name": "agileplus_get_audit_trail",
  "description": "Get immutable audit trail for a feature",
  "inputSchema": {
    "type": "object",
    "properties": {
      "feature_slug": { "type": "string" },
      "after_id": {
        "type": "integer",
        "description": "Start from audit entry ID (for pagination)"
      }
    },
    "required": ["feature_slug"]
  }
}
```

Returns array of:
```json
{
  "id": 42,
  "feature_slug": "001-login",
  "wp_sequence": 1,
  "timestamp": "2025-01-16T14:22:00Z",
  "actor": "claude-code",
  "transition": "DOING -> FOR_REVIEW",
  "evidence_refs": ["pr/42", "commit/abc123"],
  "hash": "sha256:..."
}
```

### Command Dispatch

#### `agileplus_dispatch_command`

Execute arbitrary subcommands on the system.

```json
{
  "name": "agileplus_dispatch_command",
  "description": "Execute a subcommand",
  "inputSchema": {
    "type": "object",
    "properties": {
      "command": {
        "type": "string",
        "description": "Command name (e.g., 'branch:create', 'commit:create')"
      },
      "feature_slug": { "type": "string" },
      "args": {
        "type": "object",
        "additionalProperties": { "type": "string" },
        "description": "Command arguments as key-value pairs"
      }
    },
    "required": ["command"]
  }
}
```

Example: Create a branch

```json
{
  "command": "branch:create",
  "feature_slug": "001-login",
  "args": {
    "branch_name": "feat/001-login-WP01",
    "base": "main"
  }
}
```

Response:
```json
{
  "success": true,
  "message": "Branch created: feat/001-login-WP01",
  "outputs": {
    "branch_ref": "refs/heads/feat/001-login-WP01"
  }
}
```

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
        "AGILEPLUS_GRPC_HOST": "127.0.0.1",
        "AGILEPLUS_GRPC_PORT": "50051"
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
