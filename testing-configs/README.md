# Testing Configurations

Modern, free testing infrastructure for all languages.

## Tools

| Type | Tool | Language |
|------|------|----------|
| **Unit Tests** | Vitest | TypeScript/JS |
| **Unit Tests** | pytest | Python |
| **Unit Tests** | cargo test | Rust |
| **Unit Tests** | go test | Go |
| **Unit Tests** | zig test | Zig |
| **Linting** | ESLint/Prettier | TypeScript/JS |
| **Linting** | golangci-lint | Go |
| **Linting** | clippy | Rust |
| **Linting** | zig fmt | Zig |
| **Linting** | ruff | Python |
| **Load Testing** | k6 | Any HTTP |
| **Mutation Testing** | mutmut | Python |
| **Mutation Testing** | cargo-mutants | Rust |
| **Fuzzing** | cargo-fuzz | Rust |
| **Fuzzing** | go-fuzz | Go |
| **Fuzzing** | zig test --fuzz | Zig |
| **E2E Testing** | Playwright | Any |
| **BDD** | Cucumber | Any |
| **Coverage** | coverage.py | Python |
| **Coverage** | tarpaulin | Rust |
| **Coverage** | gocov | Go |

## Usage

### Bun (TypeScript/JS)
```bash
bun add -d vitest @vitest/coverage-v8
bun run vitest
```

### uv (Python)
```bash
uv pip install pytest pytest-cov ruff mypy
uv run pytest
```

### Go
```bash
go install github.com/gotestsum/gotestsum@latest
go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
gotestsum -- -race ./...
```

### Rust
```bash
cargo test --workspace
cargo clippy -- -D warnings
cargo tarpaulin --workspace
```

### Zig
```bash
zig test .
zig fmt --check .
```
