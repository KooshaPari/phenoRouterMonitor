# Polyglot Architecture Implementation Roadmap

**Date:** 2026-04-03  
**Status:** Ready for Execution  
**Scope:** All components (phenotype-skills, phenotype-mcp, agileplus-mcp, nanovms, Dino MCP, thegent-plugin-host)

---

## Executive Summary

### Components to Package/Refactor

| Component | Current State | Target State | Priority |
|-----------|---------------|--------------|----------|
| phenotype-skills | Skill SDK (empty src/lib.rs) | NuGet crate + npm package + pyproject | P1 |
| phenotype-mcp | Asset MCP (not present) | Split: Asset MCP + Testing MCP | P1 |
| agileplus-mcp | Rust CLI + protocol | Reference implementation for plugin-mcp | P1 |
| thegent-plugin-host | Rust plugin host | Generic WASM plugin system | P2 |
| Dino MCP Server | C# game MCP | Reference: Testing MCP pattern | P2 |
| nanovms | TypeScript experimental | Unikernel service packaging | P3 |

---

## Phase 1: phenotype-skills Multi-Package (Week 1)

### 1.1 Rust Core (phenotype-skills)

**Location:** `phenotype-skills/`

**Tasks:**
1. Implement core skill interface:
   ```rust
   pub trait Skill {
       fn name(&self) -> &str;
       fn version(&self) -> &str;
       fn execute(&self, ctx: &Context) -> Result<Output>;
   }
   ```
2. Add skill registry with plugin loading
3. Implement CLI wrapper
4. Set up cargo packaging for crates.io

**Commands to run:**
```bash
cd phenotype-skills
cargo build --release
cargo test
cargo publish --dry-run
```

**Deliverable:** `phenotype-skills = "0.1.0"` on crates.io

---

### 1.2 C# Bindings (phenotype-skills-csharp)

**Location:** `phenotype-skills/csharp/`

**Tasks:**
1. Create .NET Standard 2.1 project
2. Implement C# skill interface matching Rust:
   ```csharp
   public interface ISkill {
       string Name { get; }
       string Version { get; }
       SkillOutput Execute(SkillContext ctx);
   }
   ```
3. Add native interop or gRPC bridge
4. NuGet packaging

**Commands to run:**
```bash
cd phenotype-skills/csharp
dotnet build
dotnet pack
```

**Deliverable:** `Phenotype.Skills` NuGet package

---

### 1.3 Python Package (phenotype-skills-python)

**Location:** `phenotype-skills/python/`

**Tasks:**
1. Create pyproject.toml with maturin
2. Use PyO3 for Rust bindings
3. Implement Python skill interface:
   ```python
   class Skill(Protocol):
       @property
       def name(self) -> str: ...
       def execute(self, ctx: Context) -> Output: ...
   ```
4. Set up PyPI publishing

**Commands to run:**
```bash
cd phenotype-skills/python
maturin develop
maturin build --release
twine upload --repository-url https://test.pypi.org/legacy/ target/wheels/*.whl
```

**Deliverable:** `phenotype-skills` on PyPI

---

### 1.4 Node.js Package (phenotype-skills-js)

**Location:** `phenotype-skills/js/`

**Tasks:**
1. Create package.json with napi-rs
2. Generate TypeScript declarations
3. Implement JavaScript skill interface:
   ```typescript
   interface Skill {
     name: string;
     version: string;
     execute(ctx: Context): Promise<Output>;
   }
   ```
4. npm publish

**Commands to run:**
```bash
cd phenotype-skills/js
npm run build
npm pack
npm publish --dry-run
```

**Deliverable:** `@phenotype/skills` on npm

---

## Phase 2: MCP Separation & Standardization (Week 1-2)

### 2.1 Create phenotype-mcp-asset

**Location:** `phenotype-mcp/asset/`

**Purpose:** Asset building, discovery, and compilation

**Features:**
- YAML pack validation
- Asset catalog generation
- Pack compilation
- Dependency resolution

**Implementation:**
- Copy from Dino PackCompiler logic
- Remove Unity-specific code
- Add generic asset interfaces

**Deliverable:** NuGet crate + npm package

---

### 2.2 Create phenotype-mcp-testing

**Location:** `phenotype-mcp/testing/`

**Purpose:** Game testing, validation, screenshot capture

**Features:**
- Game launch orchestration
- UI automation
- Screenshot capture
- State validation

**Implementation:**
- Extract from Dino DesktopCompanion
- Extract from Dino MCP Server
- Add Playwright integration

**Deliverable:** NuGet crate + npm package

---

### 2.3 Unify agileplus-mcp

**Location:** `agileplus-mcp/`

**Actions:**
1. Refactor into library crate (lib.rs)
2. Separate CLI (main.rs) from core
3. Use phenotype-skills for skill loading
4. Add MCP protocol abstraction

**Code structure:**
```
agileplus-mcp/
├── src/
│   ├── lib.rs           # Library exports
│   ├── protocol.rs      # MCP protocol
│   ├── skills.rs         # Skill management
│   └── cli.rs           # CLI commands
└── src/bin/
    └── mcp-server.rs     # CLI entry
```

**Deliverable:** `agileplus-mcp` library + CLI binary

---

## Phase 3: Plugin Architecture Unification (Week 2)

### 3.1 Refactor thegent-plugin-host

**Location:** `thegent/crates/thegent-plugin-host/`

**Actions:**
1. Add WASM plugin support (wasmtime)
2. Create plugin manifest schema
3. Implement plugin lifecycle management
4. Add hot-reload capability

**Interface:**
```rust
pub trait PluginHost {
    fn load(&self, manifest: &PluginManifest) -> Result<Plugin>;
    fn unload(&self, id: &str) -> Result<()>;
    fn call(&self, id: &str, method: &str, args: Args) -> Result<Output>;
}
```

**Deliverable:** `thegent-plugin-host v0.2.0`

---

### 3.2 Create phenotype-plugin-cli

**Location:** `phenotype-plugin-cli/`

**Purpose:** Generic CLI for plugin management

**Features:**
- `plugin init` - Create new plugin from template
- `plugin build` - Compile plugin
- `plugin test` - Run plugin tests
- `plugin publish` - Publish to registry

**Templates:**
- `rust-wasm` - Rust WASM plugin
- `csharp-dll` - C# DLL plugin
- `python-wheel` - Python wheel plugin
- `typescript-napi` - Node.js N-API plugin

**Deliverable:** `phenotype-plugin` CLI

---

## Phase 4: nanovms Integration (Week 3)

### 4.1 Create phenotype-service-unikernel

**Location:** `nanovms/phenotype-service/`

**Purpose:** Package phenotype services as unikernels

**Actions:**
1. Create nanovms/ directory structure
2. Add Rust service examples:
   - phenotype-mcp-asset as unikernel
   - phenotype-mcp-testing as unikernel
3. Create Dockerfile for building
4. Add ops.json configuration

**Example:**
```rust
// nanovms/phenotype-service/src/main.rs
use phenotype_mcp_asset::{AssetMcpServer, Config};

fn main() {
    let config = Config::from_env();
    let server = AssetMcpServer::new(config);
    server.run().expect("Server failed");
}
```

**Deliverable:** Unikernel images for phenotype services

---

### 4.2 Docker Service Decoupling

**Location:** All service components

**Actions:**
1. Identify Docker-dependent services
2. Create unikernel alternatives for each
3. Update docker-compose.yml to include unikernel options
4. Document migration path

**Services to decouple:**
- [ ] phenotype-security-aggregator
- [ ] phenotype-telemetry
- [ ] phenotype-registry

---

## Phase 5: DINOForge Integration (Week 3-4)

### 5.1 Replace Dino SDK with Phenotype.Packs

**Location:** `Dino/src/SDK/`

**Actions:**
1. Add `Phenotype.Packs` NuGet reference
2. Remove ContentLoader.cs (341 lines) → Use library
3. Remove PackManifest.cs (206 lines) → Use library
4. Update using statements
5. Run full Dino test suite

**Commands to run:**
```bash
cd Dino/src/SDK
dotnet add package Phenotype.Packs --version 0.1.0
dotnet build
# Fix any API mismatches
dotnet test ../../Tests/
```

**Verification:** All 1,017 tests pass

---

### 5.2 Dino MCP Server Migration

**Location:** `Dino/src/Tools/McpServer/`

**Actions:**
1. Reference `phenotype-mcp-testing`
2. Remove duplicate screenshot/validation code
3. Keep only Dino-specific game logic
4. Update to unified MCP protocol

**Deliverable:** Dino MCP Server using phenotype-mcp-testing

---

### 5.3 Add Phenotype.Skills Integration

**Location:** `Dino/src/Tools/McpServer/`

**Actions:**
1. Add skill loading from phenotype-skills
2. Create Dino-specific skill implementations:
   - `GameLaunchSkill`
   - `ScreenshotSkill`
   - `StateQuerySkill`
3. Register skills with MCP server

**Deliverable:** Dino skills integrated with phenotype-skills

---

## Build Order & Dependencies

```
Week 1:
├── phenotype-skills (Rust) ──────┐
│   ├── C# bindings               │
│   ├── Python bindings           │
│   └── Node.js bindings          │
└── phenotype-mcp-asset ──────────┤
    └── phenotype-mcp-testing ────┤
                                 │
Week 2:                          │
├── thegent-plugin-host           │
│   └── phenotype-plugin-cli      │
├── agileplus-mcp (refactor) ─────┤
└── phenotype-service-unikernel ──┘
                                 │
Week 3-4:                        │
└── Dino Integration ─────────────┘
```

---

## Commands Reference

### Build All Components
```bash
#!/bin/bash
# build-all.sh

# 1. phenotype-skills
cd phenotype-skills
cargo build --release
cd csharp && dotnet build && cd ..
cd python && maturin develop && cd ..
cd js && npm run build && cd ..
cd ..

# 2. phenotype-mcp
cd phenotype-mcp/asset && dotnet build && cd ..
cd phenotype-mcp/testing && dotnet build && cd ..

# 3. agileplus-mcp
cd agileplus-mcp
cargo build --release
cd ..

# 4. thegent-plugin-host
cd thegent/crates/thegent-plugin-host
cargo build --release
cd ../../..

echo "All components built successfully!"
```

### Run All Tests
```bash
#!/bin/bash
# test-all.sh

# Rust tests
find . -name "Cargo.toml" -exec dirname {} \; | while read dir; do
    if [ -d "$dir/src" ]; then
        echo "Testing $dir..."
        (cd "$dir" && cargo test 2>&1 | tail -5)
    fi
done

# .NET tests
find . -name "*.Tests.csproj" | while read proj; do
    echo "Testing $proj..."
    dotnet test "$proj" --verbosity quiet
done

echo "All tests complete!"
```

---

## Verification Checklist

- [ ] phenotype-skills on crates.io
- [ ] Phenotype.Skills on NuGet
- [ ] phenotype-skills on PyPI
- [ ] @phenotype/skills on npm
- [ ] phenotype-mcp-asset NuGet package
- [ ] phenotype-mcp-testing NuGet package
- [ ] agileplus-mcp library crate
- [ ] thegent-plugin-host WASM support
- [ ] phenotype-plugin-cli tool
- [ ] nanovms unikernel builds
- [ ] Dino builds with Phenotype.Packs
- [ ] All Dino tests pass (1,017)
- [ ] Docker services have unikernel alternatives

---

## Documentation to Update

| Document | Changes Needed |
|----------|---------------|
| Dino/README.md | Add "Built on Phenotype" section |
| phenotype-skills/README.md | Add multi-language quickstart |
| phenotype-mcp/README.md | Document asset vs testing split |
| agileplus-mcp/README.md | Update for library usage |
| docs/POLYGLOT_ARCHITECTURE_OPTIMIZATION.md | Mark phases complete |
| nanovms/README.md | Add phenotype service examples |

---

## Immediate Next Steps (This Session)

1. **Build phenotype-skills core** - Run cargo commands
2. **Verify Dino integration works** - Test with NuGet package
3. **Create phenotype-mcp-asset skeleton** - Copy from Dino
4. **Document current state** - Update READMEs

---

## Success Criteria

**Week 1:** All phenotype-skills packages published to registries  
**Week 2:** MCP components separated and working independently  
**Week 3:** Plugin architecture unified with WASM support  
**Week 4:** Dino fully integrated, all 1,017 tests passing  

---

## Notes

- Focus on trait/interface compatibility across languages
- Use protobuf/gRPC for cross-language communication
- WASM for plugins enables language-agnostic extensibility
- nanovms unikernels provide security + performance for services
- Dino integration proves the generic abstraction actually works
