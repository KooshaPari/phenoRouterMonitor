# Testing Infrastructure

Comprehensive testing setup for all Phenotype repos using modern free tooling.

## Overview

| Language | Runtime | Test Runner | Linter | Coverage |
|----------|---------|-------------|--------|----------|
| TypeScript | Bun | Vitest | ESLint | v8 |
| Python | uv | pytest | ruff | coverage.py |
| Go | go | go test | golangci-lint | gocov |
| Rust | cargo | cargo test | clippy | tarpaulin |
| Zig | zig | zig test | zig fmt | builtin |

## Repositories

### nanovms
- **Branch:** docs
- **Files:** `vitest.config.ts`, `playwright.config.ts`, `.golangci.yaml`, `Makefile.go`

### AgilePlus  
- **Branch:** feat/add-testing-workflow (PR needed)
- **Files:** `vitest.config.ts`, `playwright.config.ts`, `.github/workflows/ai-testing.yml`

### thegent
- **Branch:** refactor/cleanup-error-variants
- **Files:** `vitest.config.ts`, `playwright.config.ts`, `.github/workflows/ai-testing.yml`

### phenotype-logging-zig
- **Branch:** main
- **Files:** `.zigtest.toml`, `Makefile.zig`

## Usage

### Bun (TypeScript)
```bash
bun install
bun run vitest run
bun run playwright test
```

### uv (Python)
```bash
uv pip install pytest pytest-cov --system
pytest --cov=. --cov-report=html
```

### Go
```bash
go test ./...
golangci-lint run ./
```

### Rust
```bash
cargo test --workspace
cargo clippy -- -D warnings
```

### Zig
```bash
zig test .
zig fmt --check .
```

## Infrastructure

### Docker
```bash
docker compose up -d
```

### Kubernetes
```bash
kubectl apply -f infrastructure/kubernetes/deployment.yaml
```

### Load Testing
```bash
k6 run testing-configs/k6-load-test.js
```

## Paid Services Removed

- ~~Qodo~~ → Vitest + cargo test
- ~~TestRigor~~ → Cucumber/BDD
- ~~Applitools~~ → Playwright screenshots

## CI/CD

GitHub Actions workflows configured for:
- Scheduled runs (weekly)
- Push triggers
- Multi-browser Playwright tests
- Code coverage reporting
