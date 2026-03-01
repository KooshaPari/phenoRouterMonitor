---
audience: [developers, agents]
---

# Environment Variables

Configuration options available through environment variables.

## Core

| Variable | Default | Description |
|----------|---------|-------------|
| `AGILEPLUS_PROJECT` | `.` | Path to project root |
| `AGILEPLUS_CONFIG` | `.kittify/config.toml` | Config file path |
| `AGILEPLUS_LOG_LEVEL` | `info` | Log verbosity: `trace`, `debug`, `info`, `warn`, `error` |
| `AGILEPLUS_NO_COLOR` | `false` | Disable colored output |

## API & Server

| Variable | Default | Description |
|----------|---------|-------------|
| `AGILEPLUS_API_TOKEN` | — | Authentication token for gRPC API |
| `AGILEPLUS_GRPC_PORT` | `50051` | gRPC server listen port |
| `AGILEPLUS_GRPC_HOST` | `127.0.0.1` | gRPC server bind address |

## Sync

| Variable | Default | Description |
|----------|---------|-------------|
| `PLANE_API_KEY` | — | Plane.so API key |
| `PLANE_WORKSPACE` | — | Plane.so workspace slug |
| `GITHUB_TOKEN` | — | GitHub personal access token |

## Agent Dispatch

| Variable | Default | Description |
|----------|---------|-------------|
| `AGILEPLUS_AGENT_TIMEOUT` | `30m` | Default agent session timeout |
| `AGILEPLUS_AGENT_HARNESS` | `claude-code` | Default agent harness |
| `CLAUDE_CODE_PATH` | `claude` | Path to Claude Code binary |

## Build & CI

| Variable | Default | Description |
|----------|---------|-------------|
| `GITHUB_ACTIONS` | — | Set by GitHub Actions runner |
| `GITHUB_PAGES` | — | Enables Pages base path for docs |
| `GITHUB_REPOSITORY` | — | `owner/repo` for link generation |
