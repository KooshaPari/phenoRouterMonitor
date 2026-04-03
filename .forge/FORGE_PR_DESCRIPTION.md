# PR Description: Governance Files and Workspace Compilation Fixes

## Summary

This PR adds standardized governance files across all Rust projects in the phenotype ecosystem and fixes workspace compilation errors to ensure consistent code quality, security auditing, and agent context.

## Changes

### Governance Files Added (7 files)

| File | Purpose |
|------|---------|
| `.gitignore` | Standard git ignore patterns for Rust/Cargo projects |
| `.gitmodules` | Submodule configuration for project dependencies |
| `AGENTS.md` | Agent rules and guidelines for AI-assisted development |
| `CLAUDE.md` | Project context and architecture documentation for Claude Code |
| `_typos.toml` | Typos check configuration for documentation and code |
| `clippy.toml` | Clippy lint configuration with strict rules |
| `deny.toml` | Cargo deny configuration for dependency auditing |

### Workspace Dependencies Added (Cargo.toml)

Added missing workspace dependencies required by member crates:
- `axum` - Web framework for API crates
- `opentelemetry-otlp` - OpenTelemetry OTLP exporter

### Compilation Fixes

#### Missing Crate Manifests Created
Created minimal `Cargo.toml` files for workspace crates that were missing them:
- `crates/phenotype-mock/Cargo.toml`
- `crates/phenotype-event-sourcing/Cargo.toml`
- `crates/phenotype-port-traits/Cargo.toml`

#### Missing Source Files Created
Created `lib.rs` files with proper module structure:
- `crates/phenotype-event-sourcing/src/lib.rs`
- `crates/phenotype-mock/src/lib.rs`
- `crates/phenotype-port-traits/src/lib.rs`
- `crates/phenotype-string/src/lib.rs`

#### Type and Error Fixes
- Fixed `StringError::InvalidUtf8` → `Error::InvalidUtf8` in `phenotype-string`
- Fixed type annotations in `phenotype-mock/src/stub.rs`
- Corrected feature flags for `tokio` dev-dependencies

### Governance Standards Applied

1. **Linting & Code Quality** (`clippy.toml`)
   - Enabled all pedantic lints
   - Strict documentation requirements
   - Performance and correctness checks

2. **Security Auditing** (`deny.toml`)
   - License compliance checking
   - Vulnerability scanning via advisory DB
   - Dependency ban list for known problematic crates

3. **Documentation Quality** (`_typos.toml`)
   - Automated typo detection
   - Project-specific word exceptions

4. **Agent Context** (`AGENTS.md`, `CLAUDE.md`)
   - Development workflow guidelines
   - Project architecture overview
   - Testing and quality standards

## Testing

- [x] Files are valid TOML format
- [x] Clippy configuration loads without errors
- [x] Cargo deny can parse the configuration
- [x] Typos checker runs successfully
- [x] Workspace compiles with `cargo check --workspace`
- [x] All crates have valid manifests

## Commits

```
2fb0e49170 chore: add standardized governance files (clippy, deny, typos, AGENTS, CLAUDE)
61dc693ac fix: resolve workspace compilation errors
b171f5569 fix: add minimal Cargo.toml for phenotype-mock, phenotype-event-sourcing, phenotype-port-traits
```

## Related

Part of the larger effort to standardize governance across all phenotype projects and ensure the workspace compiles cleanly.
