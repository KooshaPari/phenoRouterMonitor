# HexaKit

Multi-language hexagonal architecture templates and scaffolding registry.

## Overview

HexaKit is a domain registry providing hexagonal architecture patterns across multiple programming languages. Each language implementation is independently importable.

## Structure

```
HexaKit/
├── go/           # Go hexagonal implementation
├── python/       # Python hexagonal implementation
├── rust/         # Rust hexagonal implementation
└── typescript/   # TypeScript hexagonal implementation
```

## Usage

```bash
# Rust
cargo install hexakit-rust

# Go
go get github.com/KooshaPari/HexaKit/go

# Python
pip install hexakit

# TypeScript
npm install @hexakit/typescript
```

## Language Packages

| Language | Package | Purpose |
|----------|---------|---------|
| Rust | `hexakit` | Hexagonal scaffolding for Rust |
| Go | `github.com/KooshaPari/HexaKit/go` | Ports & Adapters in Go |
| Python | `hexakit` | Hexagonal patterns for Python |
| TypeScript | `@hexakit/typescript` | Ports & Adapters for TS/Node |

## License

MIT
