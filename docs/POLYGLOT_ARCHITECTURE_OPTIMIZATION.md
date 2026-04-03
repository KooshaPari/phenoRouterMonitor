# Phenotype Ecosystem: Polyglot Architecture Optimization

**Analysis Date:** 2026-04-03  
**Scope:** Skills, Plugins, MCP Servers, NVMS Integration  
**Status:** Strategic Planning

---

## Executive Summary

The Phenotype ecosystem currently operates as a **polyglot, distributed system** with components written in Go, Python, Rust, TypeScript, and Zig. While this provides language-appropriate solutions, it creates **fragmentation in packaging, deployment, and isolation strategies**.

### Key Findings

| Component | Language | Packaging | Isolation | Status |
|-----------|----------|-----------|-----------|--------|
| phenotype-skills | Rust (crates) | Cargo workspace | bwrap/gVisor/WASM/Firecracker | Active |
| thegent-plugin-host | Rust | Cargo crate | WASM/Dynamic libs | Active |
| Dino MCP Server | C# | .NET + NuGet | Windows-focused | Active |
| heliosCLI MCP | Rust | Cargo workspace | Process-based | Active |
| nanovms | Go | Binary + Ops | Three-tier (WASM/gVisor/Firecracker) | Active |
| bare-cua | Rust | Binary | Native process | Active |

**Critical Gap:** Docker is used across 12+ projects for deployment, but nanovms provides a superior lightweight VM alternative that remains underutilized.

---

## 1. Current Architecture Analysis

### 1.1 Skills Ecosystem (phenotype-skills)

```
phenotype-skills/
├── crates/
│   ├── agileplus-p2p/          # P2P device mesh
│   └── ... (other skill crates)
├── skills/                     # Skill definitions
├── hexagonal/                  # Hexagonal architecture templates
└── docs/
```

**Strengths:**
- Hot-reloading support
- Multi-language skill runtime (Rust, Python, TS)
- Four-tier sandboxing: bwrap → gVisor → WASM → Firecracker
- Semantic versioning with dependency resolution

**Weaknesses:**
- Skill distribution fragmented across repos
- No unified skill registry
- Docker used for some deployments instead of native nanovms integration

### 1.2 Plugin Architecture

**Three Implementations:**

1. **thegent-plugin-host** (Rust, 41 dirs)
   - Hexagonal architecture (Ports & Adapters)
   - DDD with Entities, Value Objects, Domain Events
   - WASM + Dynamic library loading
   - CLI, API ports with storage adapters

2. **heliosCLI plugins** (Mixed)
   - Bazel-based build system
   - Integration with codex-mcp-server

3. **Dino Runtime** (C#)
   - BepInEx plugin system
   - Unity ECS integration
   - Windows-centric

**Fragmentation Issue:** Each plugin system uses different loading mechanisms, security models, and packaging formats.

### 1.3 MCP Server Landscape

**Four Separate Implementations:**

| Project | Location | Language | Purpose |
|---------|----------|----------|---------|
| Dino | `Dino/src/Tools/McpServer/` | C# | Game automation (screenshot, UI, input) |
| heliosCLI | `heliosCLI/codex-rs/mcp-server/` | Rust | Shell command MCP |
| thegent | `thegent/` (MCP tests) | Python | Agent management |
| Tooling | `tooling/browser-agent-mcp/` | ? | Browser automation |

**Problem:** No shared MCP infrastructure. Each implements the protocol independently with different transport layers (stdio, SSE, WebSocket).

### 1.4 Container vs VM Strategy

**Current Docker Usage:**

```bash
# 20+ Dockerfiles across ecosystem
AgilePlus/Dockerfile.rust          # Multi-stage Rust build
Dino/Dockerfile                    # Windows game modding
evalora/Dockerfile                 # Evaluation runner
heliosCLI/Dockerfile               # CLI container
Kogito/Dockerfile                  # Go microservice
PolicyStack/Dockerfile             # Policy engine
Profila/docker-compose.yml         # Multi-service
# ... and 12 more
```

**nanovms Capability (Underutilized):**

```
nanovms/
├── Three-tier isolation:
│   ├── Tier 1: WASM (~1ms, ~1MB)      # Tool execution
│   ├── Tier 2: gVisor (~90ms, ~20MB)  # Syscall filtering
│   └── Tier 3: MicroVMs (~125ms, <5MB) # Full isolation
├── Firecracker integration
├── OCI-compatible
└── Go-based orchestration
```

---

## 2. Optimal Packaging Strategy

### 2.1 Unified Component Model

Proposed **Phenotype Component Standard** — All components package as:

```
Phenotype Component Package (PCP)
├── manifest.pcm          # Component metadata
├── schema/               # JSON schemas
│   ├── input.schema.json
│   └── output.schema.json
├── runtime/              # Language-specific
│   ├── rust/             # Cargo workspace member
│   ├── python/           # Wheel + requirements
│   ├── go/               # Binary + go.mod
│   └── csharp/           # NuGet package
├── sandbox/              # Isolation config
│   ├── wasm/             # wasm32-wasi target
│   ├── gvisor/           # seccomp profile
│   └── microvm/          # Firecracker config
└── skills/               # Embedded skills (if applicable)
```

### 2.2 Skill + Plugin + MCP Unification

**Current State (Fragmented):**

```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│   Skills    │  │   Plugins   │  │MCP Servers  │
│  (phenotype-│  │  (thegent-  │  │ (scattered) │
│   skills)   │  │plugin-host) │  │             │
├─────────────┤  ├─────────────┤  ├─────────────┤
│ Hot reload  │  │ Dynamic lib │  │ Stdio/SSE   │
│ WASM/gVisor │  │ WASM        │  │ JSON-RPC    │
│ bwrap       │  │ Hexagonal   │  │ Tools       │
└─────────────┘  └─────────────┘  └─────────────┘
```

**Proposed Unified Architecture:**

```
┌─────────────────────────────────────────────────────────────┐
│              Phenotype Runtime (Unified)                    │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │ Skill Host  │  │ Plugin Host │  │ MCP Router  │         │
│  │             │  │             │  │             │         │
│  │ • Registry  │  │ • Loader    │  │ • Transport │         │
│  │ • Lifecycle │  │ • Sandbox   │  │ • Routing   │         │
│  │ • Events    │  │ • Events    │  │ • Auth      │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         │                │                │               │
│  ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐        │
│  │Skill Loader │  │Plugin Loader│  │MCP Handlers │        │
│  │             │  │             │  │             │        │
│  │ • WASM      │  │ • Dynamic   │  │ • Assets    │        │
│  │ • Python    │  │ • WASM      │  │ • Testing   │        │
│  │ • Rust      │  │ • Process   │  │ • Runtime   │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
├─────────────────────────────────────────────────────────────┤
│              nanovms Isolation Layer                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │ WASM (1ms)  │  │ gVisor(90ms)│  │MicroVM(125ms)        │
│  │ ~1MB        │  │ ~20MB       │  │ <5MB        │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
└─────────────────────────────────────────────────────────────┘
```

### 2.3 Recommended Packaging by Language

**Rust Components (phenotype-skills, thegent-plugin, heliosCLI):**

```toml
# Unified Cargo.toml pattern
[package]
name = "phenotype-{component}"
version = "0.1.0"
edition = "2021"

[features]
default = ["std"]
std = []
wasm = ["wasm-bindgen"]
gvisor = ["seccomp"]
microvm = ["firecracker"]

[[bin]]
name = "phenotype-{component}"
path = "src/main.rs"
required-features = ["cli"]

[lib]
name = "phenotype_{component}"
path = "src/lib.rs"

[dependencies]
phenotype-core = { workspace = true }  # Shared abstractions
nanovms = { workspace = true, optional = true }
```

**Go Components (nanovms, Kogito):**

```go
// Unified binary + library pattern
package main

import (
    "github.com/phenotype-org/core"
    "github.com/phenotype-org/nanovms"
)

// Binary entrypoint for nanovms
func main() {
    runtime := nanovms.NewRuntime()
    runtime.RegisterComponent(&MyComponent{})
    runtime.Run()
}
```

**Python Components (phenotype-middleware, tooling):**

```toml
# pyproject.toml
[project]
name = "phenotype-{component}"
version = "0.1.0"
dependencies = ["phenotype-core>=0.1.0"]

[project.optional-dependencies]
nanovms = ["phenotype-nanovms>=0.1.0"]

[project.scripts]
phenotype-{component} = "phenotype_{component}.cli:main"
```

**C# Components (Dino):**

```xml
<!-- .csproj -->
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
    <PackageId>Phenotype.{Component}</PackageId>
    <Version>0.1.0</Version>
  </PropertyGroup>
  
  <ItemGroup>
    <PackageReference Include="Phenotype.Core" Version="0.1.0" />
    <PackageReference Include="Phenotype.NanoVMS" Version="0.1.0" />
  </ItemGroup>
</Project>
```

---

## 3. MCP Server Separation Strategy

### 3.1 Current Problem

MCP servers are **monolithic** within projects:

```
Dino/src/Tools/McpServer/
├── BareCua.cs              # CUA integration
├── GameProcessManager.cs   # Game lifecycle
├── Program.cs              # Entrypoint
└── Tools/                  # 23 game tools
    ├── AssetScreenshot.cs
    ├── GameStateQuery.cs
    ├── UIAutomation.cs
    └── ...
```

All tools bundled together = tight coupling, hard to test, can't scale independently.

### 3.2 Proposed Separation

**Two MCP Server Categories:**

#### Category A: Asset Building & Discovery

```
phenotype-mcp-assets/
├── manifest.json          # MCP manifest
├── tools/
│   ├── pack_discovery/    # Find packs in directories
│   ├── asset_indexing/    # Catalog 3D models, textures
│   ├── dependency_scan/   # Analyze pack dependencies
│   ├── validation/        # Schema validation
│   └── build_pipeline/    # Trigger asset builds
├── runtime/
│   ├── rust/              # High-performance indexing
│   └── python/            # Asset processing scripts
└── nanovms/
    └── microvm.conf       # Tier 3 isolation for untrusted packs
```

**Use Case:** Dino pack development, content discovery, CI/CD asset validation

#### Category B: Game Testing & Validation

```
phenotype-mcp-testing/
├── manifest.json          # MCP manifest
├── tools/
│   ├── game_launch/       # Start game instances
│   ├── state_capture/     # Screenshot + UI state
│   ├── input_injection/   # Send clicks/keys
│   ├── performance/       # FPS, memory profiling
│   ├── scenario_runner/   # Automated test scenarios
│   └── regression_detect/ # Compare state snapshots
├── runtime/
│   └── csharp/            # Dino bridge integration
└── nanovms/
    ├── gvisor.conf        # Tier 2 for game isolation
    └── microvm.conf       # Tier 3 for clean state
```

**Use Case:** Automated game testing, regression detection, scenario validation

### 3.3 MCP Router Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     MCP Router (Rust)                        │
├─────────────────────────────────────────────────────────────┤
│  Transport Layer                                            │
│  ├── stdio (local)                                         │
│  ├── SSE (HTTP)                                            │
│  └── WebSocket (real-time)                                 │
├─────────────────────────────────────────────────────────────┤
│  Tool Registry                                              │
│  ├── Asset Discovery: phenotype-mcp-assets               │
│  ├── Game Testing: phenotype-mcp-testing                 │
│  ├── Shell: codex-mcp-server                              │
│  └── Browser: browser-agent-mcp                          │
├─────────────────────────────────────────────────────────────┤
│  Isolation Router                                           │
│  ├── WASM: phenotype-mcp-assets (Tier 1)                 │
│  ├── gVisor: phenotype-mcp-testing (Tier 2)                │
│  └── MicroVM: Clean game state (Tier 3)                  │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. Docker → NVMS Migration Path

### 4.1 Current Docker Footprint

```bash
# Representative Dockerfile (AgilePlus)
FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/agileplus /usr/local/bin/
ENTRYPOINT ["agileplus"]
```

**Issues:**
- Base image: ~100MB+ (debian:bookworm-slim)
- Build time: Multi-stage with full Rust toolchain
- Startup: Container initialization overhead
- Security: Full Linux syscall surface

### 4.2 NVMS Equivalent

```yaml
# nanovms manifest (ops.json)
{
  "ProjectName": "agileplus",
  "BaseVolumeSz": "50m",
  "RunConfig": {
    "Memory": "512m",
    "CPUs": 2
  },
  "ManifestPassthrough": {
    "sandbox": "gvisor",  # or "wasm", "microvm"
    " isolation_tier": 2
  }
}
```

**Advantages:**
- Image size: <5MB (vs 100MB+ Docker)
- Startup: ~90ms (vs 2-5s container)
- Security: Syscall filtering (gVisor) or full VM (Firecracker)
- Density: 100x more instances per host

### 4.3 Migration Priority Matrix

| Component | Current | Target | Priority | Effort |
|-----------|---------|--------|----------|--------|
| AgilePlus | Docker | nanovms gVisor | High | Medium |
| phenotype-skills | bwrap | nanovms WASM/Tier 1 | High | Low |
| Kogito | Docker | nanovms microVM | Medium | Medium |
| PolicyStack | Docker | nanovms gVisor | Medium | Medium |
| Dino | Native | Keep native + NVMS for testing | Low | High |
| heliosCLI | Process | nanovms WASM | High | Low |
| Evalora | Docker | nanovms gVisor | Medium | Medium |

### 4.4 Unified Deployment Spec

```yaml
# phenotype-deployment.yaml
apiVersion: phenotype.org/v1
kind: Component
metadata:
  name: agileplus-p2p
spec:
  runtime: rust
  binary: target/release/agileplus-p2p
  
  isolation:
    tier: 2  # gVisor
    memory: 256Mi
    cpu: 1
    
  sandbox:
    seccomp: default
    network: restricted
    filesystem: readonly
    
  skills:  # Embedded skill registry
    - name: device-discovery
      version: 1.2.0
    - name: p2p-mesh
      version: 2.0.1
      
  mcp:
    enabled: true
    transport: stdio
    tools:
      - device-list
      - mesh-status
      
  nanovms:
    base: phenotype/rust-gvisor:latest
    size: 20m
```

---

## 5. Component Integration Architecture

### 5.1 Proposed Unified Stack

```
┌──────────────────────────────────────────────────────────────────────┐
│                        Agent Layer                                    │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌────────────┐     │
│  │  Claude    │  │   Goose    │  │  Codex     │  │  Other     │     │
│  │  Desktop   │  │  CLI       │  │  Agent     │  │  Agents    │     │
│  └──────┬─────┘  └──────┬─────┘  └──────┬─────┘  └──────┬─────┘     │
└─────────┼──────────────┼──────────────┼──────────────┼───────────────┘
          │              │              │              │
          └──────────────┴──────┬──────┴──────────────┘
                                │
┌─────────────────────────────────┴──────────────────────────────────┐
│                      MCP Router (Unified)                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐     │
│  │ Asset MCP       │  │ Testing MCP     │  │ Shell MCP       │     │
│  │ (phenotype-mcp  │  │ (phenotype-mcp  │  │ (codex-mcp      │     │
│  │  -assets)       │  │  -testing)      │  │  -server)       │     │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘     │
└───────────┼───────────────────┼───────────────────┼───────────────┘
            │                   │                   │
┌───────────┴───────────────────┴───────────────────┴───────────────┐
│                    Skill + Plugin Runtime                        │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │              Phenotype Runtime Core                        │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │ │
│  │  │ Skill Host  │  │ Plugin Host │  │ Component   │        │ │
│  │  │             │  │             │  │ Registry    │        │ │
│  │  │ • WASM      │  │ • Dynamic   │  │             │        │ │
│  │  │ • Python    │  │ • WASM      │  │ • Discovery │        │ │
│  │  │ • Rust      │  │ • Process   │  │ • Lifecycle │        │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘        │ │
│  └────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────┬──┘
                                                                 │
┌────────────────────────────────────────────────────────────────┴──┐
│                      nanovms Isolation Layer                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │
│  │ Tier 1: WASM│  │ Tier 2:     │  │ Tier 3:     │                  │
│  │             │  │ gVisor      │  │ MicroVM     │                  │
│  │ • Skills    │  │             │  │             │                  │
│  │ • Tools     │  │ • Plugins   │  │ • Full apps │                  │
│  │ • 1ms       │  │ • MCP       │  │ • 125ms     │                  │
│  │ • ~1MB      │  │ • 90ms      │  │ • <5MB      │                  │
│  │             │  │ • ~20MB     │  │             │                  │
│  └─────────────┘  └─────────────┘  └─────────────┘                  │
└─────────────────────────────────────────────────────────────────────┘
```

### 5.2 Language-Specific Bindings

All components expose the same interface across languages:

**Rust (Primary):**
```rust
use phenotype::{Skill, Plugin, McpHandler};

#[derive(Skill)]
struct MySkill;

#[derive(Plugin)]
struct MyPlugin;
```

**Go:**
```go
import "github.com/phenotype-org/sdk"

skill := phenotype.NewSkill("my-skill")
plugin := phenotype.NewPlugin("my-plugin")
```

**Python:**
```python
from phenotype import Skill, Plugin

class MySkill(Skill):
    pass
```

**C#:**
```csharp
using Phenotype;

[Skill("my-skill")]
public class MySkill : ISkill { }
```

---

## 6. Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)

1. **Create phenotype-core crate/package**
   - Shared abstractions (Skill, Plugin, Component)
   - Trait definitions
   - Serialization schemas

2. **Refactor phenotype-skills**
   - Extract generic runtime from current implementation
   - Integrate nanovms as default isolation
   - Remove bwrap-only limitation

3. **Create MCP Router**
   - Rust-based stdio/SSE/WebSocket multiplexer
   - Tool registry with capability filtering
   - Isolation tier routing

### Phase 2: Component Extraction (Weeks 3-4)

1. **phenotype-mcp-assets**
   - Extract from Dino: pack discovery, asset indexing
   - Extract from heliosCLI: file operations
   - Package as WASM + gVisor variants

2. **phenotype-mcp-testing**
   - Extract from Dino: game process management, screenshot
   - Extract from thegent: test scenario runner
   - C# bridge for Unity games, Rust for generic

3. **thegent-plugin-host v2**
   - Refactor on phenotype-core
   - Add WASM-first loading
   - nanovms integration

### Phase 3: Docker Migration (Weeks 5-6)

1. **High-priority containers → nanovms**
   - AgilePlus: Dockerfile.rust → ops.json
   - phenotype-skills: bwrap → nanovms WASM
   - heliosCLI: process → nanovms WASM

2. **Build unified deployment tooling**
   - `phenotype deploy` CLI
   - Generate ops.json from Cargo.toml/pyproject.toml
   - Multi-tier isolation selection

### Phase 4: Ecosystem Integration (Weeks 7-8)

1. **Update all MCP implementations**
   - Dino: Refactor to use phenotype-mcp-testing
   - heliosCLI: Refactor to use MCP Router
   - thegent: Add MCP transport

2. **Documentation & Migration Guide**
   - "Moving from Docker to NVMS"
   - "Packaging Skills for Phenotype"
   - Component integration patterns

---

## 7. Success Metrics

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| Container startup | 2-5s | <100ms | Average cold start |
| Container size | 100MB+ | <10MB | Compressed image size |
| Memory per instance | 100MB+ | <20MB | RSS at idle |
| Density per host | 10-50 | 500+ | Concurrent instances |
| MCP tool count (duplicated) | ~50 | ~15 | Unique tool implementations |
| Skill packaging formats | 4+ | 1 | PCP format adoption |
| Isolation tier usage | ~5% WASM | 60% Tier 1/2 | nanovms adoption |

---

## 8. Immediate Actions

### This Week:

1. **Create phenotype-core repository**
   - Define `Skill`, `Plugin`, `McpHandler` traits
   - JSON schema for component manifests

2. **Draft phenotype-deployment.yaml spec**
   - Unify Cargo.toml, pyproject.toml, .csproj concepts
   - Include nanovms isolation tier selection

3. **Prototype MCP Router**
   - Stdio multiplexer in Rust
   - Route to existing MCP servers (Dino, heliosCLI)

### Next Week:

4. **Refactor phenotype-skills**
   - Integrate nanovms as default
   - Test WASM skill loading

5. **Extract phenotype-mcp-assets**
   - Move Dino asset tools to standalone
   - Package as WASM component

Would you like me to proceed with any of these immediate actions?
