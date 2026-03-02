---
audience: [developers, agents]
---

# Environment Variables Reference

Complete reference of all environment variables affecting AgilePlus behavior.

## Core Configuration

### AGILEPLUS_PROJECT

```bash
AGILEPLUS_PROJECT=/path/to/project agileplus specify 001-feature
```

| Value | Default | Description |
|-------|---------|-------------|
| Path | `.` (current directory) | Project root directory |

Used by all commands to locate the git repository and `.agileplus/` configuration.

### AGILEPLUS_CONFIG

```bash
AGILEPLUS_CONFIG=/custom/path/config.toml agileplus plan 001-feature
```

| Value | Default | Description |
|-------|---------|-------------|
| Path | `.kittify/config.toml` | Configuration file location |

Allows using a custom configuration file instead of default location.

### AGILEPLUS_DATABASE or AGILEPLUS_DB

```bash
AGILEPLUS_DB=/tmp/agileplus.db agileplus specify 001-feature
```

| Value | Default | Description |
|-------|---------|-------------|
| Path | `.agileplus/agileplus.db` | SQLite database file path |

Overrides the `--db` CLI flag. Useful for in-memory testing (`sqlite://`) or remote databases.

### AGILEPLUS_LOG_LEVEL

```bash
AGILEPLUS_LOG_LEVEL=debug agileplus specify 001-feature
```

| Value | Description |
|-------|-------------|
| `trace` | Most verbose; all function calls and data |
| `debug` | Detailed diagnostic information |
| `info` | General operational information (default) |
| `warn` | Warning messages for potential issues |
| `error` | Only error messages |

Equivalent to CLI `-v` flags but takes precedence.

### AGILEPLUS_NO_COLOR

```bash
AGILEPLUS_NO_COLOR=1 agileplus queue list
```

| Value | Effect |
|-------|--------|
| `0` or unset | Colored output enabled |
| `1` or `true` | Colored output disabled |

Respects the `NO_COLOR` standard for Unix tools.

### AGILEPLUS_VERBOSE

```bash
AGILEPLUS_VERBOSE=2 agileplus implement 001-feature
```

| Value | Equivalent |
|-------|------------|
| `0` | `info` level |
| `1` | `debug` level |
| `2` | `trace` level |

Alternative to `AGILEPLUS_LOG_LEVEL` for programmatic control.

## Storage Backend

### Storage Type Selection

Default is SQLite, but can be overridden:

```bash
AGILEPLUS_STORAGE_TYPE=postgres \
AGILEPLUS_STORAGE_URL="postgresql://user:pass@localhost/agileplus" \
agileplus specify 001-feature
```

| Variable | Value | Purpose |
|----------|-------|---------|
| `AGILEPLUS_STORAGE_TYPE` | `sqlite` (default), `postgres`, `mysql` | Database backend |
| `AGILEPLUS_STORAGE_URL` | Connection string | Database connection details |

**SQLite:** `sqlite:///path/to/db.db` or `sqlite://` (in-memory)

**PostgreSQL:** `postgresql://user:pass@host:5432/dbname`

**MySQL:** `mysql://user:pass@host:3306/dbname`

## API & Server Configuration

### gRPC Server

```bash
AGILEPLUS_GRPC_HOST=0.0.0.0 \
AGILEPLUS_GRPC_PORT=9090 \
agileplus serve grpc
```

| Variable | Default | Description |
|----------|---------|-------------|
| `AGILEPLUS_GRPC_HOST` | `127.0.0.1` | Bind address (localhost-only by default) |
| `AGILEPLUS_GRPC_PORT` | `50051` | Listen port |
| `AGILEPLUS_GRPC_TLS_CERT` | — | Path to TLS certificate (for HTTPS) |
| `AGILEPLUS_GRPC_TLS_KEY` | — | Path to TLS private key |

### API Authentication

```bash
AGILEPLUS_API_TOKEN="sk-proj-abc123..." \
AGILEPLUS_API_TOKEN_EXPIRY=86400 \
agileplus serve grpc
```

| Variable | Required | Description |
|----------|----------|-------------|
| `AGILEPLUS_API_TOKEN` | ✓ for auth | Bearer token for gRPC requests |
| `AGILEPLUS_API_ALLOW_ANONYMOUS` | — | Allow unauthenticated requests (dev only) |
| `AGILEPLUS_API_TOKEN_EXPIRY` | — | Token lifetime in seconds (default: no expiry) |

### HTTP/REST API

```bash
AGILEPLUS_HTTP_HOST=0.0.0.0 \
AGILEPLUS_HTTP_PORT=8080 \
agileplus serve http
```

| Variable | Default | Description |
|----------|---------|-------------|
| `AGILEPLUS_HTTP_HOST` | `127.0.0.1` | Bind address |
| `AGILEPLUS_HTTP_PORT` | `8080` | Listen port |
| `AGILEPLUS_HTTP_CORS_ORIGIN` | `*` | CORS allowed origin |

## VCS Configuration

### Git Repository

```bash
AGILEPLUS_REPO=/path/to/repo agileplus plan 001-feature
```

| Variable | Default | Description |
|----------|---------|-------------|
| `AGILEPLUS_REPO` | current directory | Git repository root |

Equivalent to CLI `--repo` flag. Useful for CI/CD environments.

### Worktree Directory

```bash
AGILEPLUS_WORKTREE_DIR=.worktrees agileplus implement 001-feature
```

| Variable | Default | Description |
|----------|---------|-------------|
| `AGILEPLUS_WORKTREE_DIR` | `.worktrees` | Directory for git worktrees |
| `AGILEPLUS_WORKTREE_AUTO_CLEANUP` | `true` | Auto-remove worktrees on completion |

### Git Author (for Commits)

```bash
AGILEPLUS_GIT_AUTHOR_NAME="Claude Code" \
AGILEPLUS_GIT_AUTHOR_EMAIL="claude@anthropic.com" \
agileplus implement 001-feature
```

| Variable | Default | Description |
|----------|---------|-------------|
| `AGILEPLUS_GIT_AUTHOR_NAME` | `git config user.name` | Commit author name |
| `AGILEPLUS_GIT_AUTHOR_EMAIL` | `git config user.email` | Commit author email |

## Integration & Sync

### GitHub

```bash
GITHUB_TOKEN="ghp_..." \
GITHUB_OWNER="phenotype" \
GITHUB_REPO="agileplus" \
agileplus ship 001-feature
```

| Variable | Required | Description |
|----------|----------|-------------|
| `GITHUB_TOKEN` | ✓ for sync | Personal access token (scope: `repo, workflow`) |
| `GITHUB_OWNER` | ✓ for sync | Repository owner (user or org) |
| `GITHUB_REPO` | ✓ for sync | Repository name |
| `GITHUB_API_BASE` | — | Custom GitHub API URL (for GitHub Enterprise) |

### GitHub Actions (CI/CD)

Automatically set by GitHub Actions runner:

```bash
GITHUB_ACTIONS=true              # Running in Actions
GITHUB_REPOSITORY=owner/repo     # Repo identifier
GITHUB_SHA=abc123...             # Current commit
GITHUB_REF=refs/heads/main       # Current branch
```

### Plane.so Integration

```bash
PLANE_API_KEY="..." \
PLANE_WORKSPACE="my-workspace" \
agileplus sync plane 001-feature
```

| Variable | Required | Description |
|----------|----------|-------------|
| `PLANE_API_KEY` | ✓ | API key from Plane settings |
| `PLANE_WORKSPACE` | ✓ | Workspace slug (from URL) |
| `PLANE_API_URL` | — | Custom Plane instance URL |
| `PLANE_SYNC_BIDIRECTIONAL` | — | Enable two-way sync (default: one-way) |

### Jira Integration

```bash
JIRA_URL="https://company.atlassian.net" \
JIRA_USER="user@company.com" \
JIRA_API_TOKEN="..." \
JIRA_PROJECT_KEY="AP" \
agileplus sync jira 001-feature
```

| Variable | Required | Description |
|----------|----------|-------------|
| `JIRA_URL` | ✓ | Jira instance URL |
| `JIRA_USER` | ✓ | Jira email/username |
| `JIRA_API_TOKEN` | ✓ | API token (from Jira account settings) |
| `JIRA_PROJECT_KEY` | ✓ | Project key (e.g., AP, INFRA) |

## Agent Dispatch

### Agent Selection

```bash
AGILEPLUS_AGENT="claude-code" \
AGILEPLUS_AGENT_TIMEOUT="3600" \
agileplus implement 001-feature
```

| Variable | Default | Description |
|----------|---------|-------------|
| `AGILEPLUS_AGENT` | `claude-code` | Default agent harness to use |
| `AGILEPLUS_AGENT_TIMEOUT` | `1800` | Agent session timeout in seconds |
| `AGILEPLUS_AGENT_MAX_RETRIES` | `5` | Max review/fix cycles |

### Claude Code

```bash
CLAUDE_CODE_PATH="/usr/local/bin/claude" \
CLAUDE_API_KEY="..." \
agileplus implement 001-feature
```

| Variable | Default | Description |
|----------|---------|-------------|
| `CLAUDE_CODE_PATH` | `claude` (on PATH) | Path to Claude Code binary |
| `CLAUDE_API_KEY` | — | API key for Claude (if needed) |

### Codex

```bash
CODEX_API_KEY="..." \
CODEX_ENDPOINT="https://api.openai.com/v1" \
agileplus implement 001-feature --agent codex
```

| Variable | Default | Description |
|----------|---------|-------------|
| `CODEX_API_KEY` | — | OpenAI API key |
| `CODEX_ENDPOINT` | `https://api.openai.com/v1` | Custom endpoint |
| `CODEX_MODEL` | `gpt-4` | Model version |

### Cursor

```bash
CURSOR_PATH="/Applications/Cursor.app/Contents/MacOS/Cursor" \
agileplus implement 001-feature --agent cursor
```

| Variable | Default | Description |
|----------|---------|-------------|
| `CURSOR_PATH` | `cursor` (on PATH) | Path to Cursor binary |

## Observability & Monitoring

### Logging

```bash
AGILEPLUS_LOG_FORMAT="json" \
AGILEPLUS_LOG_FILE="/var/log/agileplus.log" \
agileplus specify 001-feature
```

| Variable | Default | Description |
|----------|---------|-------------|
| `AGILEPLUS_LOG_FORMAT` | `text` | `text` or `json` |
| `AGILEPLUS_LOG_FILE` | stdout | File path for logs |
| `AGILEPLUS_LOG_MAX_SIZE` | `100mb` | Rotate log when this size reached |
| `AGILEPLUS_LOG_MAX_AGE` | `30` | Keep logs for N days |

### Metrics & Telemetry

```bash
AGILEPLUS_METRICS_ENABLED="true" \
AGILEPLUS_METRICS_ENDPOINT="http://prometheus:9090" \
agileplus serve grpc
```

| Variable | Default | Description |
|----------|---------|-------------|
| `AGILEPLUS_METRICS_ENABLED` | `false` | Enable Prometheus metrics |
| `AGILEPLUS_METRICS_PORT` | `9091` | Metrics server port |
| `AGILEPLUS_METRICS_ENDPOINT` | — | Push metrics to Prometheus |

### Tracing (OpenTelemetry)

```bash
AGILEPLUS_TRACING_ENABLED="true" \
AGILEPLUS_JAEGER_ENDPOINT="http://jaeger:6831" \
agileplus implement 001-feature
```

| Variable | Default | Description |
|----------|---------|-------------|
| `AGILEPLUS_TRACING_ENABLED` | `false` | Enable distributed tracing |
| `AGILEPLUS_JAEGER_ENDPOINT` | `http://localhost:6831` | Jaeger agent endpoint |
| `AGILEPLUS_TRACE_SAMPLE_RATE` | `0.1` | Sampling rate (0.0–1.0) |

## Development & Testing

### Debug Mode

```bash
AGILEPLUS_DEBUG="1" \
AGILEPLUS_NO_CLEANUP="1" \
agileplus implement 001-feature
```

| Variable | Effect |
|----------|--------|
| `AGILEPLUS_DEBUG=1` | Increase verbosity; show detailed errors |
| `AGILEPLUS_NO_CLEANUP=1` | Don't delete worktrees/artifacts after completion |
| `AGILEPLUS_DRY_RUN=1` | Simulate commands without executing |

### Testing

```bash
AGILEPLUS_TEST_MODE="1" \
AGILEPLUS_TEST_STORAGE="memory" \
cargo test
```

| Variable | Description |
|----------|-------------|
| `AGILEPLUS_TEST_MODE=1` | Enable test mode (in-memory storage, no git) |
| `AGILEPLUS_TEST_STORAGE` | `sqlite` (file), `memory`, `mock` |
| `AGILEPLUS_TEST_SEED=12345` | Seed for reproducible randomness |

## CI/CD & Automation

### Continuous Integration

```bash
AGILEPLUS_CI="1" \
AGILEPLUS_CI_TIMEOUT="600" \
agileplus validate 001-feature
```

| Variable | Default | Description |
|----------|---------|-------------|
| `AGILEPLUS_CI` | — | Set to `1` to enable CI mode (stricter checks) |
| `AGILEPLUS_CI_TIMEOUT` | `600` | CI job timeout in seconds |
| `AGILEPLUS_CI_STRICT` | — | Fail on any warning (not just errors) |

### GitHub Pages (Docs)

```bash
GITHUB_PAGES="true" \
GITHUB_PAGES_BASEURL="/agileplus" \
agileplus docs build
```

| Variable | Effect |
|----------|--------|
| `GITHUB_PAGES=true` | Enable GitHub Pages base path rewriting |
| `GITHUB_PAGES_BASEURL=/repo-name` | Set docs base URL |

## Quick Reference for Common Scenarios

### Local Development

```bash
export AGILEPLUS_PROJECT="$(pwd)"
export AGILEPLUS_LOG_LEVEL="debug"
export AGILEPLUS_NO_CLEANUP="1"
```

### CI/CD Pipeline (GitHub Actions)

```bash
export AGILEPLUS_DB=".agileplus/agileplus.db"
export AGILEPLUS_LOG_LEVEL="info"
export GITHUB_TOKEN="${{ secrets.GITHUB_TOKEN }}"
export AGILEPLUS_CI="1"
```

### Docker Container

```bash
docker run \
  -e AGILEPLUS_PROJECT=/workspace \
  -e AGILEPLUS_GRPC_HOST=0.0.0.0 \
  -e GITHUB_TOKEN="${GITHUB_TOKEN}" \
  -v /path/to/repo:/workspace \
  agileplus:latest
```

### MCP Server (Agent Integration)

```bash
export AGILEPLUS_PROJECT="/path/to/project"
export AGILEPLUS_GRPC_HOST="127.0.0.1"
export AGILEPLUS_GRPC_PORT="50051"
export AGILEPLUS_API_TOKEN="$(openssl rand -hex 32)"
agileplus mcp serve
```
