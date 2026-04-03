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

# Run examples
cargo run --example kitty_graphics
cargo run --example mcp_server
```

## Usage

### Kitty Graphics

```rust
use phenotype_cli_extensions::kitty::graphics;

// Display an image in the terminal
let image_data = std::fs::read("image.png")?;
graphics::display_image(&image_data, Some(800), Some(600))?;

// Clear all graphics
graphics::clear_graphics()?;
```

### MCP Shell Server

```rust
use phenotype_cli_extensions::shell_tool_mcp::server::McpServer;

let server = McpServer::new();
let output = server.execute("ls", &["-la"])?;
println!("{}", output);
```

## Project Structure

```
phenotype-cli-extensions/
├── .agileplus/specs/          # AgilePlus specifications
│   ├── architecture-decisions/  # ADRs
│   ├── functional-requirements/ # FRs
│   ├── user-stories/           # USs
│   ├── index.yaml              # Spec index
│   └── traceability-matrix.yaml # Traceability
├── .github/workflows/          # CI/CD workflows
│   ├── ci.yml                  # Multi-platform CI
│   ├── release.yml             # Automated releases
│   ├── quality-gate.yml        # Code quality
│   ├── cargo-deny.yml          # Dependency audit
│   ├── codespell.yml           # Spell checking
│   └── security.yml            # Security scanning
├── src/
│   ├── lib.rs                  # Library root
│   ├── kitty/                  # Kitty graphics protocol
│   ├── shell_tool_mcp/         # MCP server implementation
│   └── sdk_typescript/         # TypeScript SDK bindings
├── examples/                   # Usage examples
├── tests/                      # Integration tests
├── Cargo.toml                  # Package manifest
└── deny.toml                   # Cargo-deny config
```

## CI/CD Workflows

| Workflow | Description |
|:---------|:------------|
| [CI](.github/workflows/ci.yml) | Multi-platform Rust builds with caching |
| [Quality Gate](.github/workflows/quality-gate.yml) | Formatting, clippy, docs, audit |
| [Security](.github/workflows/security.yml) | Trivy vulnerability scanning |
| [Cargo Deny](.github/workflows/cargo-deny.yml) | License and dependency auditing |
| [Codespell](.github/workflows/codespell.yml) | Spell checking |
| [Release](.github/workflows/release.yml) | Automated releases on tags |

## Specifications (AgilePlus)

This project follows the [AgilePlus methodology](https://agileplus.dev) for specification management.

### Architecture Decision Records

| ID | Title | Status |
|:---|:------|:-------|
| [ADR-001](.agileplus/specs/architecture-decisions/adr_001.yaml) | Kitty Graphics Protocol Implementation | ✅ Accepted |
| [ADR-002](.agileplus/specs/architecture-decisions/adr_002.yaml) | Model Context Protocol (MCP) Shell Integration | ✅ Accepted |
| [ADR-003](.agileplus/specs/architecture-decisions/adr_003.yaml) | TypeScript SDK Code Generation | ✅ Accepted |
| [ADR-004](.agileplus/specs/architecture-decisions/adr_004.yaml) | Kitty Keyboard Protocol | ✅ Accepted |
| [ADR-005](.agileplus/specs/architecture-decisions/adr_005.yaml) | Terminal Window Management | 📝 Proposed |
| [ADR-006](.agileplus/specs/architecture-decisions/adr_006.yaml) | Desktop Notifications | 📝 Proposed |

### Functional Requirements

| ID | Title | Status | Priority |
|:---|:------|:-------|:---------|
| [FR-001](.agileplus/specs/functional-requirements/fr_001.yaml) | Kitty Graphics Protocol Support | ✅ Implemented | High |
| [FR-002](.agileplus/specs/functional-requirements/fr_002.yaml) | MCP Shell Integration | ✅ Implemented | High |
| [FR-003](.agileplus/specs/functional-requirements/fr_003.yaml) | TypeScript SDK Type-Safe Bindings | ✅ Implemented | Medium |
| [FR-004](.agileplus/specs/functional-requirements/fr_004.yaml) | Kitty Keyboard Protocol | 📝 Proposed | Medium |
| [FR-005](.agileplus/specs/functional-requirements/fr_005.yaml) | Terminal Window Management | 📝 Proposed | Low |
| [FR-006](.agileplus/specs/functional-requirements/fr_006.yaml) | Desktop Notifications | 📝 Proposed | Low |

### Traceability Matrix

All requirements, decisions, and stories are linked in the [traceability matrix](.agileplus/specs/traceability-matrix.yaml).

## Development

### Prerequisites

- Rust 1.70 or later
- Cargo
- (Optional) Kitty terminal for graphics testing

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# With specific features
cargo build --features kitty-graphics,mcp-shell
```

### Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_kitty_module_exists
```

### Code Quality

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings

# Generate documentation
cargo doc --no-deps

# Check dependencies
cargo deny check
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Please read our [Contributing Guide](CONTRIBUTING.md) for information on:
- Code of Conduct
- Development workflow
- Submitting pull requests
- Reporting issues

## Related Projects

- [helios-cli](https://github.com/KooshaPari/helios-cli) - Reference CLI implementation
- [template-commons](https://github.com/KooshaPari/template-commons) - Reusable CI/CD workflows
- [phenotype-forge](https://github.com/KooshaPari/phenotype-forge) - Core Phenotype library

---

<p align="center">
  Built with ❤️ by the <a href="https://phenotype.dev">Phenotype Team</a>
</p>
