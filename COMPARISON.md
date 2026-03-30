# AgilePlus Ecosystem Comparison Matrix

## Overview

This document provides a feature comparison matrix for repositories in the AgilePlus ecosystem. AgilePlus is a protocol-first architecture for AI agent development and orchestration, with Protocol Buffer definitions serving as the single source of truth for inter-service contracts.

## Repository Comparison

| Repository | Purpose | Key Features | Language/Framework | Maturity Level | Comparison to Similar Forks |
|------------|---------|--------------|-------------------|----------------|-----------------------------|
| **AgilePlus** (this repo) | Protocol Buffer definitions for AgilePlus gRPC API | • Single source of truth for inter-service contracts<br>• Defines 3 gRPC services (Core, Agents, Integrations)<br>• Shared message types (Feature, AuditEntry, etc.)<br>• buf v2 lint and breaking change configuration<br>• Rust (tonic/prost) and Python (grpcio) codegen | Protocol Buffers (proto3), Rust, Python | **Production** - Core contract definitions | **Primary source** - All other repos depend on these definitions |
| **agileplus-publish** | Published/distributed version of AgilePlus proto definitions | • Same core proto definitions<br>• Likely includes build artifacts and distribution packages<br>• May have different CI/CD pipeline for publishing | Protocol Buffers, Rust, Python | **Production** - Published artifacts | **Distribution fork** - Contains same proto definitions but optimized for publishing |
| **agileplus-agents** | Agent dispatch and orchestration service | • Implements `AgentDispatchService` from proto<br>• Agent spawning and lifecycle management<br>• Review loop implementation<br>• Integration with AI models | Rust (likely), Python | **Development** - Service implementation | **Consumer** - Consumes proto definitions from AgilePlus repo |
| **agileplus-mcp** | Model Context Protocol integration | • MCP server/client implementations<br>• Tool and resource discovery<br>• Context management for AI agents<br>• Integration with various data sources | Rust, TypeScript | **Development** - MCP integration | **Extension** - Extends AgilePlus with MCP capabilities |
| **agileplus** (subdirectory) | Core service implementation | • Likely implements `AgilePlusCoreService`<br>• Feature lifecycle management<br>• Governance and audit functionality<br>• Business logic layer | Rust, possibly others | **Development** - Core service | **Core implementation** - Primary service consuming proto contracts |
| **pheno-cli** | Command-line interface | • CLI tools for AgilePlus ecosystem<br>• Development and deployment utilities<br>• Local testing and debugging | Rust (CLI), Shell | **Development** - Tooling | **Tooling layer** - CLI utilities for ecosystem |

## Feature Breakdown

### Protocol Definition Features
| Feature | AgilePlus | agileplus-publish | agileplus-agents | agileplus-mcp |
|---------|-----------|-------------------|------------------|---------------|
| **gRPC Service Definitions** | ✅ Core, Agents, Integrations | ✅ Same as AgilePlus | ⚠️ Consumes only Agents service | ❌ Not applicable |
| **Shared Message Types** | ✅ Feature, AuditEntry, etc. | ✅ Same as AgilePlus | ⚠️ Consumes types | ❌ Not applicable |
| **buf Configuration** | ✅ buf.yaml, buf.gen.yaml | ✅ Likely similar | ❌ Not applicable | ❌ Not applicable |
| **Breaking Change Detection** | ✅ `buf breaking` checks | ✅ Likely similar | ❌ Not applicable | ❌ Not applicable |
| **Multi-language Codegen** | ✅ Rust (tonic/prost), Python (grpcio) | ✅ Same as AgilePlus | ❌ Not applicable | ❌ Not applicable |

### Service Implementation Features
| Feature | agileplus (core) | agileplus-agents | agileplus-mcp | pheno-cli |
|---------|------------------|------------------|---------------|-----------|
| **Service Implementation** | ✅ Core service | ✅ Agents service | ⚠️ MCP protocol | ❌ CLI only |
| **Database Integration** | ✅ Likely present | ✅ Agent state management | ⚠️ Context storage | ❌ Not applicable |
| **API Gateway** | ✅ HTTP/gRPC bridge | ✅ Likely present | ⚠️ MCP server | ❌ Not applicable |
| **Authentication/Authorization** | ✅ Likely present | ✅ Agent auth | ⚠️ MCP auth | ❌ Not applicable |
| **Monitoring & Metrics** | ✅ Likely present | ✅ Agent metrics | ⚠️ MCP metrics | ❌ Not applicable |

### Development & Tooling Features
| Feature | AgilePlus | pheno-cli | All Service Repos |
|---------|-----------|-----------|-------------------|
| **Local Development Setup** | ✅ Makefile, docker-compose | ✅ CLI tools | ✅ Individual setups |
| **Testing Framework** | ✅ Proto validation tests | ✅ CLI tests | ✅ Service tests |
| **CI/CD Pipeline** | ✅ GitHub Actions | ✅ Likely present | ✅ Individual pipelines |
| **Documentation** | ✅ README, CONTRIBUTING | ✅ CLI docs | ✅ Service docs |
| **Dependency Management** | ✅ Cargo.toml, package.json | ✅ Cargo.toml | ✅ Individual configs |

## Architecture Relationships

```
┌─────────────────────────────────────────────────────────────┐
│                    AgilePlus (Proto Definitions)            │
│  • Single source of truth for contracts                     │
│  • Protocol Buffer definitions only                         │
└───────────────┬─────────────────────────────────────────────┘
                │
                ├─────────────────────────────────────────────┐
                │                                             │
        ┌───────▼───────┐                             ┌───────▼───────┐
        │ agileplus     │                             │ agileplus-    │
        │ (core service)│                             │ publish       │
        │ • Implements  │                             │ • Distribution│
        │   CoreService │                             │   artifacts   │
        └───────┬───────┘                             └───────────────┘
                │
        ┌───────▼───────┐                             ┌───────────────┐
        │ agileplus-    │                             │ pheno-cli     │
        │ agents        │                             │ • CLI tools   │
        │ • Implements  │                             │ • Dev utilities│
        │   AgentService│                             └───────────────┘
        └───────┬───────┘
                │
        ┌───────▼───────┐
        │ agileplus-mcp │
        │ • MCP protocol│
        │   integration │
        └───────────────┘
```

## Maturity Assessment

### Production Ready
- **AgilePlus**: Core protocol definitions with strict breaking change policy
- **agileplus-publish**: Distribution mechanism for proto artifacts

### Active Development
- **agileplus** (core service): Implementing business logic layer
- **agileplus-agents**: Agent orchestration service
- **agileplus-mcp**: MCP protocol integration
- **pheno-cli**: Development tooling

### Dependencies
All service implementations depend on **AgilePlus** for:
1. Protocol Buffer definitions
2. Message type schemas  
3. Service interface contracts
4. Breaking change coordination

## Recommendations

1. **For protocol changes**: Always modify **AgilePlus** first, then regenerate stubs in dependent repos
2. **For service development**: Use **pheno-cli** for local development and testing
3. **For agent orchestration**: Implement against **agileplus-agents** service
4. **For MCP integration**: Use **agileplus-mcp** for tool/resource discovery
5. **For distribution**: Use **agileplus-publish** for published artifacts

## Version Compatibility

| Component | Current Version | Compatibility Notes |
|-----------|----------------|---------------------|
| **Protocol Definitions** | v1 | Breaking changes require v2 module path |
| **Rust Crate** | Follows proto version | Must regenerate on proto changes |
| **Python Package** | Follows proto version | Must regenerate on proto changes |
| **Service Implementations** | Independent versions | Must update to match proto changes |

## Contributing Guidelines

When contributing to the AgilePlus ecosystem:
1. **Protocol changes**: Submit to **AgilePlus** with breaking change analysis
2. **Service implementations**: Submit to respective service repo
3. **Tooling improvements**: Submit to **pheno-cli**
4. **Documentation**: Update relevant README files in each repo

---

*Last updated: 2026-03-25*  
*This matrix helps developers understand the AgilePlus ecosystem structure and choose the right repository for their needs.*