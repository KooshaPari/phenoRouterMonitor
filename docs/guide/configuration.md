# Configuration

## Project Config

`agileplus init` generates `.agileplus/config.toml`:

```toml
[project]
name = "my-project"
type = "brownfield"  # or "greenfield"

[vcs]
provider = "git"

[agents]
enabled = ["claude", "cursor"]

[conventions]
languages = ["rust", "typescript"]
frameworks = ["axum", "vitepress"]
```

## CLI Flags

Global flags available on all commands:

| Flag | Description |
|------|-------------|
| `-v`, `--verbose` | Increase verbosity (-v, -vv, -vvv) |
| `--db <path>` | SQLite database path (default: `.agileplus/agileplus.db`) |
| `--repo <path>` | Git repository root (default: current directory) |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `AGILEPLUS_DB` | Override default database path |
| `PLANE_API_KEY` | Plane.so API key for sync |
| `GITHUB_TOKEN` | GitHub token for issue sync |
