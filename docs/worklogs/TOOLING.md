
---

## 2026-03-30 - Dev Container & Environment Standardization (Wave 140)

**Project:** [cross-repo]
**Category:** tooling, devops
**Status:** in_progress
**Priority:** P1

### Dev Container Tools

| Tool | Use Case | Phenotype Status |
|------|----------|------------------|
| **Dev Container (VS Code)** | Local dev | ⚠️ Partial |
| **Docker Compose** | Multi-service | ✅ Adopted |
| **Nix** | Reproducible builds | ❌ Not used |
| **Devbox** | Nix wrapper | ❌ Not used |
| **Flakes** | Nix v2 | ❌ Not used |

### Recommended Configuration

```jsonc
// .devcontainer/devcontainer.json
{
  "name": "Phenotype Development",
  "image": "mcr.microsoft.com/devcontainers/rust:1.75",
  "features": {
    "ghcr.io/devcontainers/features/docker-in-docker:2": {},
    "ghcr.io/devcontainers/features/node-18:1": {}
  },
  "postCreateCommand": "cargo fetch && npm install",
  "portsAttributes": {
    "3000": {"label": "API"},
    "5432": {"label": "Postgres"},
    "6379": {"label": "Redis"}
  }
}
```

### Nix Flake for Reproducible Builds

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }: {
    devShells.x86_64-linux.default = let
      pkgs = import nixpkgs {
        overlays = [ rust-overlay ];
      };
    in pkgs.mkShell {
      buildInputs = with pkgs; [
        rust-bin.stable.latest.default
        cargo-nextest
        sccache
        protobuf
        nodejs_20
      ];
    };
  };
}
```

---

## 2026-03-30 - CI/CD Pipeline Optimization (Wave 141)

**Project:** [cross-repo]
**Category:** tooling, CI/CD
**Status:** in_progress
**Priority:** P1

### CI Pipeline Comparison

| Aspect | GitHub Actions | GitLab CI | Buildkite |
|--------|----------------|-----------|-----------|
| **Cost** | Minutes-based | Included | Pay-per-minute |
| **Caching** | Good | Good | Excellent |
| **Matrix** | Native | Native | Native |
| **Artifacts** | Limited | Unlimited | Unlimited |

### Optimized GitHub Actions

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup sccache
        uses: mozilla-actions/sccache-action@v0.0.3

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Cache cargo registry
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: "crates"

      - name: Clippy
        run: cargo clippy --workspace -- -D warnings

      - name: Test
        run: cargo nextest run --workspace

      - name: Build docs
        run: cargo doc --no-deps

  python:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: astral-sh/setup-uv@v3

      - name: Install
        run: uv sync --all-extras

      - name: Type check
        run: uv pyright

      - name: Test
        run: uv pytest

  sccache-stats:
    needs: rust
    runs-on: ubuntu-latest
    steps:
      - uses: mozilla-actions/sccache-action@v0.0.3
        with:
          action: upload
```

---

## 2026-03-30 - Code Generation Tools (Wave 142)

**Project:** [cross-repo]
**Category:** tooling, codegen
**Status:** proposed
**Priority:** P2

### Code Generation Landscape

| Tool | Language | Use Case | Phenotype |
|------|----------|----------|-----------|
| **cargo-expand** | Rust | Macro debugging | ✅ |
| **proc_macro_hack** | Rust | Compile-time eval | ❌ |
| **derive_builder** | Rust | Builder pattern | ✅ |
| **serde_generate** | Multi | Schema → types | ✅ (buf) |
| **axum-gen** | Rust | Route generation | ❌ |

### Recommended Codegen Stack

| Use Case | Tool | Config |
|----------|------|--------|
| Protobuf | buf | `buf.gen.yaml` |
| gRPC | tonic-build | `build.rs` |
| SQL | sqlx-cli | `sqlx-data.json` |
| OpenAPI | utoipa | `#[derive(ToSchema)]` |
| Builders | derive_builder | `#[derive(Builder)]` |

---

## 2026-03-30 - Version Management (Wave 143)

**Project:** [cross-repo]
**Category:** tooling, versioning
**Status:** in_progress
**Priority:** P1

### Version Tools

| Tool | Type | Phenotype | Status |
|------|------|-----------|--------|
| **cargo-release** | Rust | ✅ | Adopted |
| **release-please** | GitHub | ✅ | Adopted |
| **changesets** | Monorepo | ❌ | Evaluate |
| **semantic-release** | GitHub | ❌ | Evaluate |

### Recommended: changesets for Monorepo

```bash
# Install
npm install -D @changesets/cli

# Init
npx changeset init
```

```jsonc
// .changeset/config.json
{
  "$schema": "https://unpkg.com/@changesets/config@3.0.0/schema.json",
  "changelog": "@changesets/cli/changelog",
  "commit": false,
  "fixed": [
    "packages/phenotype-event-sourcing",
    "packages/phenotype-policy-engine"
  ],
  "linked": [],
  "access": "restricted",
  "baseBranch": "main",
  "updateInternalDependencies": "patch",
  "ignore": []
}
```

### Conventional Commits Enforcement

```yaml
# .github/workflows/commitlint.yml
- name: Validate commit messages
  uses: wagoid/commitlint-github-action@v5
  with:
    configFile: .commitlintrc.yml
```

```yaml
# .commitlintrc.yml
rules:
  body-leading-blank: [2, always]
  body-max-line-length: [2, always, 100]
  footer-leading-blank: [2, always]
  header-max-length: [2, always, 72]
  type-enum:
    - 2
    - always
    - [feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert]
```

---

_Last updated: 2026-03-30 (Wave 143)_
