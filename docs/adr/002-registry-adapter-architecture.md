# ADR 002: Registry Adapter Architecture

**Date:** 2026-03-01
**Status:** Accepted
**Deciders:** Phenotype Platform Team

## Context

pheno-cli must publish packages to 7 different package registries:

1. **npm** — Node.js / TypeScript packages
2. **PyPI** — Python packages
3. **crates.io** — Rust crates
4. **Go module proxy** — Go modules (tag-based, proxy propagation)
5. **Hex.pm** — Elixir packages
6. **Zig package index** — Zig packages
7. **Mojo package registry** — Mojo packages

Each registry has materially different conventions:
- **Authentication**: API tokens (npm, PyPI, crates), git tags (Go), API keys (Hex)
- **Version format**: SemVer with dashes (npm, crates), PEP 440 (PyPI), tag-only (Go)
- **Build artifacts**: tarballs (npm), wheels/sdists (PyPI), no artifact (Go), .tar.gz (crates)
- **Publish mechanism**: HTTP upload (npm, PyPI, crates), git push + tag (Go), CLI tool (hex, zig)
- **Verification**: HTTP polling, CLI query, DNS propagation

A naive approach of a single monolithic publish function would result in a deeply branched, untestable module that couples unrelated registry concerns.

## Decision

We adopt a **RegistryAdapter interface pattern** in `internal/adapters/`. Each registry is implemented as a self-contained adapter that satisfies the `RegistryAdapter` interface:

```go
type RegistryAdapter interface {
    Detect(repoPath string) ([]Package, error)
    Version(baseVersion string, channel Channel, increment int) (string, error)
    Build(pkg Package) (*BuildResult, error)
    Publish(pkg Package, version string, creds map[string]string) (*PublishResult, error)
    Verify(pkg Package, version string) (bool, error)
    Name() Registry
}
```

### Interface Method Responsibilities

| Method    | Responsibility                                                    |
|-----------|-------------------------------------------------------------------|
| `Detect`  | Scan a repo path for manifest files; return publishable packages  |
| `Version` | Format a pre-release version string per this registry's convention|
| `Build`   | Execute the registry-specific build step; return artifact path    |
| `Publish` | Upload artifact to the registry using provided credentials        |
| `Verify`  | Confirm the published version is accessible on the registry       |
| `Name`    | Return the canonical registry identifier                          |

### Orchestrator

The publish orchestrator (`internal/publish/orchestrator.go`) holds a registry of `RegistryAdapter` implementations. It:

1. Calls `Detect` on each adapter to find packages in the repo.
2. Calls `Version` to compute the channel-specific version string.
3. Calls `Build` to produce the artifact.
4. Calls `Publish` to upload.
5. Calls `Verify` to confirm availability.

The orchestrator is registry-agnostic; all registry-specific logic lives in the adapter.

### Stub Adapters

Registries not yet fully supported (Hex, Zig, Mojo) are represented by stub adapters that implement `Detect` (so packages are discovered) but return `ErrNotSupported` from `Build` and `Publish`. This allows partial rollout without blocking the orchestrator.

## Alternatives Considered

### 1. Switch statement in a single publish function

A single `publishPackage(pkg Package, registry Registry)` function with a switch on `registry`. Rejected because:
- Adding a new registry requires modifying the central function (violates open/closed principle).
- Testing one registry's logic requires mocking or skipping all other branches.
- Registry-specific state (e.g., HTTP client, auth tokens) bleeds into a shared scope.

### 2. Plugin-based dynamic loading

Load adapter implementations as Go plugins (`.so` files) at runtime. Rejected because:
- Go plugins have significant platform restrictions (Linux-only, CGO dependency).
- Plugin versioning and compatibility is fragile.
- Overkill for 7 known registries with a stable interface.

### 3. Separate CLI binary per registry

One binary each: `pheno-npm`, `pheno-pypi`, etc. Rejected because:
- Fragmenting the UX across binaries complicates CI pipelines.
- Shared logic (channel governance, gate evaluation) would need to be duplicated or extracted to a library.

## Consequences

**Positive:**
- Each adapter is independently testable in isolation.
- Adding a new registry requires only creating a new file that satisfies `RegistryAdapter` — no changes to existing code.
- Stub adapters allow partial rollout without breaking the orchestrator.
- The interface is narrow (5 methods + Name) and stable; adapters are unlikely to need interface changes.

**Negative:**
- New adapters must be explicitly registered in the orchestrator; there is no auto-discovery.
- The `Version` method encodes registry-specific pre-release format knowledge inside each adapter rather than in a shared version module. This is intentional (each registry has genuinely different conventions) but means version format tests are distributed across adapter test files.
- Credentials are passed as `map[string]string` to keep the interface generic. Adapters must document which credential keys they expect.

## Implementation Notes

- Sentinel errors (`ErrRateLimited`, `ErrAuth`, `ErrAlreadyPublished`, etc.) are defined in `adapter.go` and shared across all adapters for consistent error handling in the orchestrator.
- The `Channel` type and `ChannelOrdinal` helper are also defined in `adapter.go` so adapters can make channel-aware decisions without importing a separate governance package (avoiding import cycles).
