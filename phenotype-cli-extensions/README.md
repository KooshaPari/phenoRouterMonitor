# Phenotype CLI Extensions

[![CI](https://github.com/KooshaPari/phenotype-cli-extensions/actions/workflows/ci.yml/badge.svg)](https://github.com/KooshaPari/phenotype-cli-extensions/actions/workflows/ci.yml)
[![Quality Gate](https://github.com/KooshaPari/phenotype-cli-extensions/actions/workflows/quality-gate.yml/badge.svg)](https://github.com/KooshaPari/phenotype-cli-extensions/actions/workflows/quality-gate.yml)
[![Security Scan](https://github.com/KooshaPari/phenotype-cli-extensions/actions/workflows/security.yml/badge.svg)](https://github.com/KooshaPari/phenotype-cli-extensions/actions/workflows/security.yml)
[![Cargo Deny](https://github.com/KooshaPari/phenotype-cli-extensions/actions/workflows/cargo-deny.yml/badge.svg)](https://github.com/KooshaPari/phenotype-cli-extensions/actions/workflows/cargo-deny.yml)
[![Codespell](https://github.com/KooshaPari/phenotype-cli-extensions/actions/workflows/codespell.yml/badge.svg)](https://github.com/KooshaPari/phenotype-cli-extensions/actions/workflows/codespell.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)

> CLI extensions for the Phenotype ecosystem - Kitty graphics, MCP shell integration, and TypeScript SDK bindings.

## Features

- **Kitty Graphics Protocol**: Display images directly in Kitty-compatible terminals
- **MCP Shell Integration**: Model Context Protocol server for secure shell command execution  
- **TypeScript SDK**: Automatic type-safe bindings generated from Rust code

## Quick Start

```bash
# Clone the repository
git clone https://github.com/KooshaPari/phenotype-cli-extensions.git
cd phenotype-cli-extensions

# Build the project
cargo build --release

# Run tests
cargo test
```

## Project Structure

```
phenotype-cli-extensions/
├── .agileplus/specs/          # AgilePlus specifications
│   ├── architecture-decisions/  # ADRs 001-006
│   ├── functional-requirements/ # FRs 001-006  
│   ├── user-stories/           # USs 001-006
│   ├── index.yaml              # Spec index
│   └── traceability-matrix.yaml # Traceability
├── .github/workflows/          # CI/CD workflows
│   ├── ci.yml                  # Multi-platform CI
│   ├── release.yml             # Automated releases
│   ├── quality-gate.yml        # Code quality
│   ├── cargo-deny.yml          # Dependency audit
│   ├── codespell.yml           # Spell checking
│   └── security.yml            # Security scanning
├── Cargo.toml                  # Package manifest
└── deny.toml                   # Cargo-deny config
```

## Specifications (AgilePlus)

This project follows the [AgilePlus methodology](https://agileplus.dev).

### Architecture Decision Records

| ID | Title | Status |
|:---|:------|:-------|
| ADR-001 | Kitty Graphics Protocol Implementation | ✅ Accepted |
| ADR-002 | Model Context Protocol (MCP) Shell Integration | ✅ Accepted |
| ADR-003 | TypeScript SDK Code Generation | ✅ Accepted |
| ADR-004 | Kitty Keyboard Protocol | ✅ Accepted |
| ADR-005 | Terminal Window Management | 📝 Proposed |
| ADR-006 | Desktop Notifications | 📝 Proposed |

## License

MIT - See [LICENSE](LICENSE) for details.
