# phenotype-mcp-asset

> MCP Server for Asset Building and Discovery

This MCP server provides tools for building, discovering, and managing assets for the Phenotype ecosystem. It handles:

- Asset discovery and indexing
- Pack manifest creation and validation
- Content building and compilation
- Dependency resolution for packs
- Asset hot-reloading

## Tools

### Asset Discovery

- `discover_assets` - Discover assets in a directory
- `index_assets` - Index discovered assets for fast lookup
- `query_assets` - Query the asset index

### Pack Management

- `create_pack` - Create a new pack manifest
- `validate_pack` - Validate a pack manifest
- `build_pack` - Build a pack from source
- `resolve_dependencies` - Resolve pack dependencies

### Content Building

- `compile_wasm` - Compile WASM modules
- `build_shaders` - Compile shader assets
- `bundle_assets` - Bundle assets for distribution

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                     phenotype-mcp-asset                              │
│                                                                       │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
│  │ Asset Discovery  │  │ Pack Management  │  │ Content Building │  │
│  │                  │  │                  │  │                  │  │
│  │ - discover       │  │ - create_pack    │  │ - compile_wasm   │  │
│  │ - index          │  │ - validate_pack  │  │ - build_shaders  │  │
│  │ - query          │  │ - build_pack     │  │ - bundle         │  │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘  │
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │                    Phenotype.Packs Library                      │ │
│  │                                                                   │ │
│  │  - PackManifest, ContentType, Validation                         │ │
│  │  - FileDiscovery, Compatibility, HotReload                     │ │
│  │                                                                   │ │
│  └──────────────────────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────────────────┘
```

## Installation

```bash
cargo install phenotype-mcp-asset
```

## Usage

```json
{
  "mcpServers": {
    "phenotype-asset": {
      "command": "phenotype-mcp-asset",
      "args": ["--packs-dir", "./packs"]
    }
  }
}
```

## License

MIT
