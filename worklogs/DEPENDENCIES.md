# DEPENDENCIES Worklogs

External dependency management, fork candidates, package modernization, and 3rd party integration work.

---

## 2026-03-29 - Fork Candidates Analysis (CRITICAL)

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P0

### Summary

Comprehensive analysis of fork candidates for library extraction. Identified 4 CRITICAL forks needed.

### Fork Candidates Summary

| ID | Source | Target | LOC | Priority | Risk |
|----|--------|--------|-----|----------|------|
| FORK-001 | `utils/pty` | `phenotype-process` | ~750 | CRITICAL | Low |
| FORK-002 | `error.rs` pattern | `phenotype-error` | ~400 | CRITICAL | Low |
| FORK-003 | `utils/git` | `phenotype-git` | ~300 | MEDIUM | Medium |
| FORK-004 | `utils/config` | `phenotype-config` | ~200 | MEDIUM | Low |

### Decision Framework

| Factor | Consideration |
|--------|-------------|
| Usage count | >3 repos = fork candidate |
| LOC | >100 LOC = consider fork |
| Coupling | Low external coupling = good fork candidate |
| Maintenance burden | High = prioritize fork |

### Related
- `plans/2026-03-29-FORK_EXECUTION_PLAN-v1.md`

---

## 2026-03-29 - FORK-001: phenotype-process (PTY)

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P0

### Summary

Fork `utils/pty` to create `phenotype-process` library for process/PTY management.

### Source Analysis

**Location:** `utils/pty/`
**Lines:** ~750
**Current Users:** heliosCLI

### Components

```rust
// Core modules
pub mod pty;
pub mod reader;
pub mod writer;
pub mod error;

pub struct PtyProcess {
    master_fd: RawFd,
    child_pid: pid_t,
    pty_width: u16,
    pty_height: u16,
}

impl PtyProcess {
    pub fn spawn(cmd: &[&str], cwd: Option<&Path>) -> Result<Self>;
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()>;
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    pub fn write(&mut self, buf: &[u8]) -> Result<usize>;
    pub fn kill(&mut self) -> Result<()>;
    pub fn wait(&mut self) -> Result<i32>;
}
```

### Fork Strategy

1. Create `libs/phenotype-process/`
2. Copy source from `utils/pty/`
3. Update dependencies
4. Add tests
5. Publish to internal registry
6. Update heliosCLI to depend on new crate
7. Deprecate `utils/pty/`

### Timeline

- **Week 1:** Fork and initial cleanup
- **Week 2:** Add documentation and tests
- **Week 3:** Migrate first consumer
- **Week 4:** Deprecate old location

### Related
- FORK-001 in main plan

---

## 2026-03-29 - FORK-002: phenotype-error (Error Types)

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P0

### Summary

Fork error pattern into `phenotype-error` to eliminate 36+ duplicate error enums.

### Source Analysis

**Pattern:** `error.rs` files with thiserror
**Files:** 36+
**LOC:** ~400

### Common Error Variants

```rust
pub enum CoreError {
    NotFound(String),
    Conflict(String),
    InvalidInput(String),
    Serialization(String),
    Io(String),
    Timeout,
    Unauthorized,
    Forbidden,
    Storage(String),
    Config(String),
}
```

### Fork Strategy

1. Create `libs/phenotype-error/`
2. Define `CoreError` enum with thiserror
3. Add conversion traits
4. Migrate one crate at a time
5. Add blanket implementations

### Timeline

- **Week 1:** Create crate, define enum
- **Week 2:** Migrate agileplus-domain
- **Week 3:** Migrate dependent crates
- **Week 4:** Remove duplicates

### Related
- FORK-002 in main plan

---

## 2026-03-29 - FORK-003: phenotype-git (Git Operations)

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P1

### Summary

Evaluate and potentially fork git operations to `phenotype-git`.

### Source Analysis

| Location | Pattern | LOC |
|----------|---------|-----|
| `utils/git/` | Raw commands | ~300 |
| `agileplus-git/` | Full library | N/A |

### Options

1. **Fork agileplus-git** - Already a full library
2. **Use gix directly** - Already a dependency
3. **Create lightweight wrapper** - Around existing utilities

### Decision Criteria

- `agileplus-git` provides more functionality
- `gix` is async-first and modern
- `utils/git` is basic and duplicative

### Recommendation

1. Evaluate `agileplus-git` for library quality
2. Consider using `gix` as foundation
3. Deprecate `utils/git/`

### Next Steps
- [ ] Audit `agileplus-git` API surface
- [ ] Compare with `gix` capabilities
- [ ] Decide on approach

---

## 2026-03-29 - FORK-004: phenotype-config (Config Loading)

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P1

### Summary

Fork config loading patterns to `phenotype-config`.

### Source Analysis

**Pattern:** Configuration loading with `dirs_next::home_dir()`
**Files:** 4+ crates
**LOC:** ~200

### Pattern to Extract

```rust
pub trait ConfigLoader: Sized {
    fn load() -> Result<Self, ConfigError>;
    fn load_from(path: &Path) -> Result<Self, ConfigError>;
    fn config_path() -> PathBuf;
}

pub fn home_dir() -> PathBuf {
    dirs_next::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".phenotype")
}

pub fn ensure_config_dir() -> Result<PathBuf, ConfigError> {
    let dir = home_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
```

### Fork Strategy

1. Create `libs/phenotype-config/`
2. Define `ConfigError` enum
3. Implement `ConfigLoader` trait
4. Add TOML/JSON/YAML support
5. Migrate from agileplus-domain first

---

## 2026-03-29 - Modern Tooling Integration (uv, ruff, gix)

**Project:** [cross-repo]
**Category:** dependencies
**Status:** completed
**Priority:** P1

### Summary

Integrated modern tooling (uv, ruff, gix) across projects.

### Tool Status

| Tool | Purpose | Status |
|------|---------|--------|
| **uv** | Python package manager | ✅ Integrated |
| **ruff** | Python linter/formatter | ✅ Integrated |
| **gix** | Git operations (async) | ✅ Using (migrating from git2) |
| **buf** | Protocol buffers | ✅ Integrated |
| **just** | Command runner | ✅ Integrated |

### uv Benefits

```bash
# Fast installs
uv pip install -r requirements.txt  # 10-100x faster than pip

# Lock files
uv pip compile requirements.in > requirements.txt

# Virtual envs
uv venv
uv sync
```

### ruff Configuration

```toml
[tool.ruff]
line-length = 100
target-version = "py311"

[tool.ruff.lint]
select = ["E", "F", "I", "N", "W", "UP"]
ignore = ["E501"]
```

### gix Migration

```toml
# Replace git2 with gix
git2 = "0.18"  # Old
gix = "0.71"   # New (async-first)
```

### Related
- Migration: git2 -> gix (in ARCHITECTURE.md)

---

## 2026-03-29 - Security Advisory Tracking (RUSTSEC)

**Project:** [cross-repo]
**Category:** dependencies
**Status:** in_progress
**Priority:** P0

### Summary

Track and remediate Rust security advisories.

### Active Advisories

| ID | Crate | Issue | Severity | Status |
|----|-------|-------|----------|--------|
| RUSTSEC-2025-0140 | gix 0.71 | Pinned old version | Medium | Pending fix |
| RUSTSEC-2025-0134 | rustls-pemfile | Deprecated | Low | Awaiting async-nats |
| RUSTSEC-2026-0049 | rustls-webpki | Via async-nats | Medium | Awaiting async-nats |

### Remediation Plan

1. **gix:** Update to latest version when available
2. **async-nats:** Update when new version releases with fixed deps
3. **rustls-pemfile:** Remove direct dependency, use async-nats bundled

### CI Integration

```yaml
- name: Security audit
  run: |
    cargo audit --deny warnings
    cargo outdated --exit-code 1
```

### Timeline

- Weekly: Run `cargo audit`
- Monthly: Review `cargo outdated`
- On-release: Full advisory scan

---

## 2026-03-29 - Dependency Update Strategy

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P1

### Summary

Implement automated dependency update strategy.

### Update Cadence

| Update Type | Frequency | Automation |
|-------------|-----------|------------|
| Patch | Weekly | Renovate |
| Minor | Monthly | Renovate |
| Major | Quarterly | Manual |
| Security | ASAP | Dependabot |

### Configuration

```json
{
  "packageRules": [
    {
      "matchDatasources": ["crate"],
      "groupName": "all-minor-patches",
      "schedule": ["every weekend"],
      "automerge": true,
      "automergeType": "pr"
    },
    {
      "matchPackagePatterns": ["tokio", "serde"],
      "separateMinorPatch": true
    }
  ]
}
```

### Workflow

1. Renovate creates PR with updates
2. CI runs tests
3. Maintainer reviews
4. Auto-merge on green

### Related
- Dependency Update Automation (ARCHITECTURE.md)

---

## 2026-03-29 - External Repository Analysis (harbor-framework/skills)

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P1

### Summary

Analyzed harbor-framework/skills for potential integration or fork.

### Key Features

| Feature | Description | Phenotype Fit |
|---------|-------------|---------------|
| Skill registry | Centralized skill definitions | High |
| Tool definitions | Standardized tool schemas | High |
| Skill versioning | Versioned skill releases | Medium |
| Runtime adapter | Pluggable runtime | Medium |

### Potential Fork

**Option A: Fork and Customize**
- Fork `harbor-framework/skills`
- Add Phenotype-specific skills
- Maintain custom runtime

**Option B: Use as Reference**
- Study patterns
- Implement similar in `platforms/thegent`
- Don't fork

**Option C: Integrate**
- Add as dependency
- Wrap with Phenotype adapters

### Recommendation

Fork for maximum customization:
- Custom skill types for Phenotype domain
- Integration with spec-driven development
- Tool definitions matching Phenotype APIs

---

## 2026-03-29 - External Repository Analysis (khoj-ai/khoj)

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P1

### Summary

Analyzed khoj-ai/khoj for local AI knowledge base capabilities.

### Key Features

| Feature | Description | Phenotype Fit |
|---------|-------------|---------------|
| Local embeddings | Run locally | High |
| Multiple backends | PostgreSQL, SQLite | High |
| RAG pipeline | Retrieval augmented gen | High |
| API-first | REST + GraphQL | Medium |

### Potential Integration

**Use Case:** Local knowledge base for agent context

```python
# Potential usage
from khoj import Khoj

khoj = Khoj()
results = khoj.search(
    query="project architecture",
    filters={"project": "agileplus"}
)
```

### Recommendation

Evaluate as `platforms/knowledge-base` foundation:
1. Use Khoj's embedding pipeline
2. Add Phenotype document loaders
3. Integrate with agent dispatch

---

## 2026-03-29 - External Repository Analysis (pathwaycom/pathway)

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P1

### Summary

Analyzed pathwaycom/pathway for real-time ML data processing.

### Key Features

| Feature | Description | Phenotype Fit |
|---------|-------------|---------------|
| Stream processing | Real-time data | High |
| Connectors | 40+ data sources | High |
| LLM xpack | RAG pipelines | High |
| MCP server | Model Context Protocol | High |

### Pathway MCP Pattern

```python
# Pathway's MCP server approach
from pathway.xpacks.llm import MCP_server

server = MCP_server(
    processors=[retriever, ranker],
    model=embedder
)
server.run()
```

### Recommendation

Create `platforms/pathway-xpack`:
1. Fork Pathway's xpack pattern
2. Add Phenotype connectors
3. Build custom MCP server

---

## 2026-03-29 - External Repository Analysis (nitrojs/nitro)

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Analyzed nitrojs/nitro for edge/serverless agent runtime.

### Key Features

| Feature | Description | Phenotype Fit |
|---------|-------------|---------------|
| Edge deployment | Cloudflare, Vercel | Medium |
| Hybrid rendering | SSR + API | Low |
| Smart defaults | Zero-config | Medium |
| AI routes | Built-in LLM support | Medium |

### Nitro AI Routes

```typescript
// Nitro's AI route pattern
export default defineEventHandler(async (event) => {
  const body = await readBody(event);
  return.ai.complete('gpt-4', {
    messages: body.messages
  });
});
```

### Recommendation

Evaluate for `platforms/nitro-agent`:
- Lightweight agent runtime
- Edge deployment support
- Auto-scaling

---

## 2026-03-29 - External Repository Analysis (lightdash/lightdash)

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Analyzed lightdash/lightdash for BI tool patterns.

### Key Features

| Feature | Description | Phenotype Fit |
|---------|-------------|---------------|
| dbt integration | SQL-first BI | Medium |
| YAML config | Declarative | High |
| Semantic layer | Metrics as code | Medium |
| MCP server | AI integration | High |

### YAML-First Approach

```yaml
# Lightdash YAML pattern
version: 2
metrics:
  - name: daily_active_users
    meta:
      format: compact
    sql: user_id
    type: count_distinct
```

### Recommendation

Study for metrics infrastructure:
- YAML-first configuration
- Semantic layer patterns
- MCP integration for AI queries

---

## 2026-03-29 - External Repository Analysis (great-expectations)

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Analyzed great-expectations for data validation patterns.

### Key Features

| Feature | Description | Phenotype Fit |
|---------|-------------|---------------|
| Expectation suites | Declarative validation | High |
| Checkpoints | Automated validation runs | High |
| Data connectors | Multiple sources | Medium |
| Profiling | Auto-generate expectations | Medium |

### Expectation Suite Pattern

```python
# Great Expectations pattern
expectations = [
    expect_column_values_to_be_unique("user_id"),
    expect_column_values_to_not_be_null("email"),
    expect_column_value_lengths_to_be_between("name", 1, 100),
]

suite = ExpectationSuite("user_validation", expectations)
checkpoint = Checkpoint("user_checkpoint", suite=suite)
result = checkpoint.run()
```

### Recommendation

Create `platforms/llm-eval` with similar patterns:
- Expectation suites for agent outputs
- Checkpoints for validation automation
- Profiling for test generation

---

## 2026-03-29 - Rust Crate Modernization

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P1

### Summary

Modernize Rust crate dependencies across workspace.

### Current State

| Category | Crate | Version | Status |
|----------|-------|---------|--------|
| Serialization | serde | 1.0 | ✅ Current |
| Async | tokio | 1.x | ✅ Current |
| HTTP | reqwest | 0.12 | ⚠️ Update available |
| Database | sqlx | 0.8 | ✅ Current |
| CLI | clap | 4.x | ✅ Current |
| Error | thiserror | 2.x | ✅ Current |
| Logging | tracing | 0.1 | ✅ Current |

### Update Priorities

1. **reqwest 0.11 -> 0.12** - Breaking changes
2. **async-nats** - Waiting on security fixes
3. **sqlx 0.7 -> 0.8** - Test required

### MSRV Strategy

| Version | Crates | Policy |
|---------|--------|--------|
| 1.75 | Most | Standard |
| 1.70 | heliosCLI | Legacy support |

### Recommendation

1. Set MSRV to 1.75 across workspace
2. Update reqwest in next release
3. Pin async-nats until security fix

---

## 2026-03-29 - Python Package Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Manage Python package dependencies using uv.

### Current Tools

| Tool | Purpose | Status |
|------|---------|--------|
| **uv** | Package manager | ✅ Primary |
| pip | Fallback | Deprecated |
| poetry | Legacy projects | Migrating |
| pip-tools | Lock files | Replacing |

### Workflow

```bash
# Install dependencies
uv pip install -r requirements.txt

# Update lock file
uv pip compile requirements.in --output-file requirements.txt

# Sync environment
uv sync

# Add package
uv add package-name
```

### Requirements Files

```text
# requirements.in
fastapi>=0.109.0
httpx>=0.27.0
pydantic>=2.0.0

# requirements.txt (generated)
fastapi==0.109.2
httpx==0.27.0
pydantic==2.6.0
```

### Recommendation

1. Migrate all Python projects to uv
2. Use `requirements.in` + `requirements.txt` pattern
3. Add to CI/CD pipeline

---

## 2026-03-29 - Protocol Buffer Dependencies (buf)

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Manage proto dependencies using buf CLI.

### Current Setup

| File | Purpose |
|------|---------|
| `schemas/agileplus.proto` | Main API |
| `schemas/event.proto` | Events |
| `schemas/sync.proto` | Sync |

### buf Configuration

```yaml
# buf.yaml
version: v2
lint:
  use:
    - DEFAULT
  except:
    - PACKAGE_VERSION_SUFFIX
breaking:
  use:
    - FILE
```

### Dependencies

```yaml
# buf.lock (generated)
deps:
  - remote: buf.build
    owner: googleapis
    repository: googleapis
    version: v1.0.0
```

### Recommendation

1. Set up buf workspace for all protos
2. Add buf.build dependencies for Google protos
3. Enable buf lint in CI

---

## 2026-03-29 - Workspace Dependency Management

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P1

### Summary

Manage workspace-level dependencies efficiently.

### Workspace Structure

```toml
# Cargo.toml (workspace)
[workspace]
members = [
    "crates/*",
    "libs/*",
]

[workspace.dependencies]
tokio = { version = "1.36", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
thiserror = "2.0"
```

### Usage in Crates

```toml
# crates/example/Cargo.toml
[dependencies]
tokio.workspace = true
serde.workspace = true
```

### Benefits

1. Single source of truth
2. Consistent versions
3. Easier updates

### Recommendation

1. Migrate all shared deps to workspace
2. Add workspace.edition
3. Document workspace dependency policy

---

## 2026-03-29 - Docker Dependency Management

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Manage Docker base images and dependencies.

### Base Images

| Image | Usage | Update Policy |
|-------|-------|---------------|
| rust:1.75-slim | Rust binaries | Monthly |
| python:3.11-slim | Python services | Monthly |
| debian:bookworm | Runtime base | Quarterly |

### Security Scanning

```dockerfile
# Multi-stage build for minimal images
FROM rust:1.75-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/binary /usr/local/bin/
```

### Trivy Scanning

```bash
# Scan image
trivy image --severity HIGH,CRITICAL myimage:latest

# Scan in CI
trivy fs --severity HIGH,CRITICAL .
```

### Recommendation

1. Pin base image versions
2. Add Trivy scanning to CI
3. Use multi-stage builds

---

## 2026-03-29 - Node.js Dependencies (for MCP)

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Manage Node.js dependencies for MCP server.

### Current Setup

```json
{
  "name": "agileplus-mcp",
  "version": "1.0.0",
  "dependencies": {
    "@modelcontextprotocol/sdk": "^1.0.0",
    "fastify": "^4.26.0"
  }
}
```

### Package Manager Options

| Manager | Pros | Cons |
|---------|------|------|
| pnpm | Fast, strict | Different from Python |
| npm | Standard | Slower |
| bun | Fastest | Newer |

### Recommendation

Use pnpm for consistency:
```bash
pnpm install
pnpm add @modelcontextprotocol/sdk
```

---

## 2026-03-29 - System Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Track system-level dependencies.

### Required System Packages

| Package | Purpose | Package Manager |
|---------|---------|----------------|
| openssl | TLS/SSL | libssl-dev |
| protobuf | Proto compilation | protobuf-compiler |
| pkg-config | Build linking | pkg-config |
| cmake | Native builds | cmake |

### Dockerfile Management

```dockerfile
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl-dev \
    protobuf-compiler \
    pkg-config \
    cmake \
    && rm -rf /var/lib/apt/lists/*
```

### Homebrew (macOS)

```bash
brew install openssl protobuf cmake
```

### Recommendation

1. Document all system deps in README
2. Add to development setup script
3. Version control system dependency versions

---

## 2026-03-29 - External API Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Manage external API dependencies and clients.

### Current Clients

| Service | Client | Status |
|---------|--------|--------|
| GitHub | gh API | Using CLI |
| OpenAI | openai crate | ✅ |
| Anthropic | anthropic crate | ✅ |
| NATS | async-nats | ✅ |

### Client Configuration

```rust
// OpenAI client
let client = OpenAI::new();
let response = client.chat().create(model::GPT4, messages).await?;

// Anthropic client
let client = Anthropic::new();
let response = client.messages().create(model::Claude3Sonnet, messages).await?;
```

### Rate Limiting

| Service | Limit | Strategy |
|---------|-------|----------|
| OpenAI | 5000 RPM | Token bucket |
| Anthropic | 1000 RPM | Exponential backoff |
| GitHub | 5000 RPH | Queue |

### Recommendation

1. Add rate limiting middleware
2. Implement retry with backoff
3. Cache responses where appropriate

---

## 2026-03-29 - Development Tool Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Track development tool dependencies.

### Required Tools

| Tool | Purpose | Version |
|------|---------|---------|
| cargo | Rust package manager | Latest |
| just | Command runner | Latest |
| sqlx-cli | Database migrations | Latest |
| buf | Proto tooling | Latest |
| wasm-pack | WASM builds | Latest |

### Installation

```bash
# Rust tools
cargo install cargo-watch cargo-audit cargo-deny

# Other tools
brew install just buf wasm-pack

# Database
cargo install sqlx-cli --features postgres
```

### CI Tool Versions

```yaml
# .github/workflows/ci.yml
- uses: dtolnay/rust-toolchain@latest
  with:
    toolchain: 1.75

- name: Install tools
  run: |
    cargo install just sqlx-cli
```

### Recommendation

1. Pin tool versions in CI
2. Document installation in CONTRIBUTING.md
3. Use `cargo-hfwd` for tool forwarding

---

## 2026-03-29 - Database Driver Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P1

### Summary

Manage database driver dependencies.

### Current Drivers

| Database | Driver | Status |
|----------|--------|--------|
| PostgreSQL | sqlx | ✅ async, compile-time checks |
| Neo4j | neo4j driver | ⚠️ Sync preferred |
| Redis | redis-rs | ✅ async support |
| SQLite | rusqlite | ⚠️ Sync only |

### Connection Pooling

```rust
// PostgreSQL with sqlx
let pool = PgPoolOptions::new()
    .max_connections(10)
    .acquire_timeout(Duration::from_secs(30))
    .connect(&database_url)
    .await?;

// Redis with redis-rs + bb8
let pool = bb8::Pool::builder()
    .max_size(10)
    .build(redis::Client::open(database_url)?)
    .await?;
```

### Recommendation

1. Standardize connection pool config
2. Add health checks
3. Implement retry logic

---

## 2026-03-29 - Serialization Library Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P1

### Summary

Evaluate serialization library dependencies.

### Current Stack

| Format | Library | Status |
|--------|---------|--------|
| JSON | serde_json | ✅ Standard |
| TOML | toml | ✅ Standard |
| YAML | serde_yaml | ✅ Available |
| MessagePack | rmp_serde | ✅ Available |
| CBOR | serde_cbor | ✅ Available |

### Performance Comparison

| Library | Serialize | Deserialize |
|---------|------------|--------------|
| serde_json | 1x (baseline) | 1x (baseline) |
| rmp_serde | 2.5x faster | 2x faster |
| serde_cbor | 2x faster | 1.8x faster |

### Recommendation

1. Keep serde_json for human-readable formats
2. Consider MessagePack for internal messaging
3. Add feature flags for format selection

---

## 2026-03-29 - Logging Library Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Evaluate logging library dependencies.

### Current Stack

| Library | Usage | Status |
|---------|-------|--------|
| tracing | Structured logging | ✅ Primary |
| tracing-subscriber | Log formatting | ✅ |
| log | Compatibility | ✅ Facade |
| env_logger | Simple cases | Legacy |

### Configuration

```rust
tracing_subscriber::fmt()
    .with_env_filter(
        EnvFilter::from_default_env()
            .add_directive("agileplus=debug".parse()?)
    )
    .with_target(true)
    .with_thread_ids(true)
    .init();
```

### Recommendation

1. Standardize on `tracing` ecosystem
2. Add JSON output for production
3. Integrate with OTEL

---

## 2026-03-29 - HTTP Client Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Evaluate HTTP client dependencies.

### Current Usage

| Crate | Usage | Status |
|-------|-------|--------|
| reqwest | REST APIs | ✅ Main |
| hyper | Low-level HTTP | Internal |
| isahc | Alternative | Deprecated |

### reqwest Configuration

```rust
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(30))
    .pool_max_idle_per_host(5)
    .build()?;

let response = client
    .post("https://api.example.com")
    .json(&payload)
    .send()
    .await?;
```

### Recommendation

1. Update to latest reqwest (0.12)
2. Add middleware for retries
3. Implement circuit breaker

---

## 2026-03-29 - gRPC Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Manage gRPC-related dependencies.

### Current Stack

| Component | Library | Status |
|-----------|---------|--------|
| Protobuf | prost | ✅ |
| gRPC runtime | tonic | ✅ |
| Health | tonic-health | ✅ |
| Reflection | tonic-reflection | ✅ |

### tonic Configuration

```rust
let health_service = HealthService::new(my_health_server);
let reflection_service = tonic_reflection::server::Builder::configure()
    .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
    .build()?;

Server::builder()
    .add_service(health_service)
    .add_service(reflection_service)
    .add_service(my_service)
    .serve(addr)
    .await?;
```

### Recommendation

1. Add reflection by default
2. Implement health checks
3. Add interceptors for auth/logging

---

## 2026-03-29 - WebAssembly Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P3

### Summary

Evaluate WebAssembly dependencies.

### Current Status

- No active WASM targets
- Potential for browser-based components

### Dependencies if Needed

```toml
[target.wasm32-unknown-unknown]
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["console"] }
js-sys = "0.3"

[lib]
crate-type = ["cdylib", "rlib"]
```

### Recommendation

1. Plan WASM targets for future
2. Add WASM-compatible dependencies
3. Set up wasm-pack build

---

## 2026-03-29 - Cache Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Manage cache-related dependencies.

### Current Stack

| Library | Usage | Status |
|---------|-------|--------|
| redis-rs | Redis client | ✅ |
| bb8 | Connection pooling | ✅ |
| moka | In-memory cache | ✅ |

### Redis Configuration

```rust
let client = redis::Client::open("redis://127.0.0.1")?;
let pool = bb8::Pool::builder()
    .max_size(10)
    .build(client)
    .await?;
```

### Moka (In-Memory)

```rust
use moka::sync::Cache;

let cache = Cache::builder()
    .max_capacity(10_000)
    .time_to_idle(Duration::from_secs(300))
    .build();
```

### Recommendation

1. Use Moka for in-process caching
2. Configure Redis pool sizes
3. Add TTL strategies

---

## 2026-03-29 - Testing Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Manage testing-related dependencies.

### Current Stack

| Library | Purpose | Status |
|---------|---------|--------|
| tokio-test | Async testing | ✅ |
| mockall | Mocking | ✅ |
| rstest | Parametrized tests | ✅ |
| proptest | Property testing | ✅ |
| wiremock | HTTP mocking | ✅ |

### Test Configuration

```rust
#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    
    #[test]
    fn test_example() {
        let mut ctx = MockClient::new();
        ctx.expect_get()
            .with(path("/api/test"))
            .returning(|_| Response::new(200, "ok"));
        
        // test code
    }
}
```

### Recommendation

1. Expand property-based testing
2. Add integration test infrastructure
3. Document test patterns

---

## 2026-03-29 - CLI Framework Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Evaluate CLI framework dependencies.

### Current Stack

| Library | Usage | Status |
|---------|-------|--------|
| clap | Argument parsing | ✅ Primary |
| anyhow | Error handling | ✅ |
| indicate | Progress bars | ✅ |
| console | Terminal UI | ✅ |

### Clap Configuration

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init { project: Option<String> },
    Build,
    Run { args: Vec<String> },
}
```

### Recommendation

1. Standardize on clap v5
2. Add shell completion generation
3. Implement interactive prompts

---

## 2026-03-29 - Metrics Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Manage metrics collection dependencies.

### Current Stack

| Library | Purpose | Status |
|---------|---------|--------|
| metrics | Core metrics | ✅ |
| metrics-exporter-prometheus | Prometheus export | ✅ |
| metrics-util | Utilities | ✅ |

### Usage

```rust
use metrics::{counter, gauge, histogram};

counter!("requests_total", "endpoint" => "/api");
gauge!("active_connections", 42.0);
histogram!("request_duration", 0.123, "endpoint" => "/api");
```

### Prometheus Endpoint

```rust
let recorder = PrometheusBuilder::new().install()?;
metrics::set_global_recorder(recorder);
```

### Recommendation

1. Add standard labels
2. Implement cardinality limits
3. Add exemplar support

---

## 2026-03-29 - Tracing Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P1

### Summary

Manage distributed tracing dependencies.

### Current Stack

| Library | Purpose | Status |
|---------|---------|--------|
| tracing | Core tracing | ✅ |
| tracing-subscriber | Log/trace output | ✅ |
| opentelemetry | OTEL support | ✅ |
| tracing-opentelemetry | Integration | ✅ |

### Configuration

```rust
let tracer = opentelemetry_otlp::new_pipeline()
    .with_endpoint("http://localhost:4317")
    .with_token("Bearer_token")
    .install_batch(opentelemetry_sdk::runtime::Tokio)?;

let telemetry = TracingPipeline::new()
    .with_tracer(tracer)
    .build();
```

### Recommendation

1. Add trace ID to all logs
2. Implement sampling strategy
3. Add span attributes

---

## 2026-03-29 - Auth Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P1

### Summary

Evaluate authentication dependencies.

### Current Stack

| Library | Purpose | Status |
|---------|---------|--------|
| jsonwebtoken | JWT handling | ✅ |
| argon2 | Password hashing | ✅ |
| totp-lite | TOTP 2FA | ✅ |

### JWT Configuration

```rust
let mut claims = Claims::default();
claims.register("sub", user_id);
claims.register("exp", expiration);

let token = encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(secret.as_bytes()),
)?;
```

### Recommendation

1. Add token refresh support
2. Implement refresh token rotation
3. Add RBAC support

---

## 2026-03-29 - Crypto Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P1

### Summary

Manage cryptographic dependencies.

### Current Stack

| Library | Purpose | Status |
|---------|---------|--------|
| ring | Low-level crypto | ✅ (via rustls) |
| rustls | TLS | ✅ |
| ed25519-dalek | Signatures | ✅ |
| sha2 | Hashing | ✅ (via sha2 crate) |

### Usage

```rust
use sha2::{Sha256, Digest};

let mut hasher = Sha256::new();
hasher.update(b"hello world");
let result = hasher.finalize();
```

### Recommendation

1. Audit crypto usage
2. Add constant-time comparison
3. Implement key derivation

---

## 2026-03-29 - Command Execution Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Evaluate command execution dependencies.

### Current Stack

| Library | Purpose | Status |
|---------|---------|--------|
| std::process::Command | Basic commands | ✅ |
| command-group | Command groups | ✅ |
| duct | Pipeline handling | ⚠️ Unused |

### command-group Usage

```rust
use command_group::{CommandGroup, NaturalCommand};

let mut group = CommandGroup::natural();
group.cmd("echo")
    .args(["hello"])
    .pipe(&mut group.cmd("wc").args(["-w"]));

let output = group.output()?;
```

### Recommendation

1. Evaluate duct for pipelines
2. Add timeout support
3. Implement stream capture

---

## 2026-03-29 - Async Stream Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P2

### Summary

Evaluate async stream processing dependencies.

### Current Stack

| Library | Purpose | Status |
|---------|---------|--------|
| futures | Stream utilities | ✅ |
| tokio-stream | Tokio integration | ✅ |
| futures-locks | Async locking | ✅ |

### Usage

```rust
use tokio_stream::StreamExt;

let mut stream = tokio_stream::iter(vec![1, 2, 3]);
while let Some(item) = stream.next().await {
    process(item).await?;
}
```

### Recommendation

1. Add buffering strategies
2. Implement backpressure
3. Add error handling utilities

---

## 2026-03-29 - Time Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P3

### Summary

Manage time-related dependencies.

### Current Stack

| Library | Purpose | Status |
|---------|---------|--------|
| chrono | DateTime handling | ✅ |
| time | Alternative | ⚠️ Considering |
| tokio::time | Async timers | ✅ (via tokio) |

### Usage

```rust
use chrono::{DateTime, Utc, Duration};

let now = Utc::now();
let later = now + Duration::days(7);
let formatted = now.format("%Y-%m-%d %H:%M:%S");
```

### Recommendation

1. Standardize on chrono
2. Add timezone handling
3. Implement duration formatting

---

## 2026-03-29 - UUID/ID Generation Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P3

### Summary

Evaluate ID generation dependencies.

### Current Stack

| Library | Usage | Status |
|---------|-------|--------|
| uuid | UUID v4 | ✅ |
| ulid | Sortable IDs | ✅ |
| nanoid | Short IDs | ⚠️ Unused |

### ULID Usage

```rust
use ulid::Ulid;

let id = Ulid::new();
// 01ARZ3NDEKTSV4RRFFQ69G5FAV - sortable, unique
```

### Recommendation

1. Standardize on ULID for entities
2. Keep UUID for external IDs
3. Consider nanoid for short URLs

---

## 2026-03-29 - Regex Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P3

### Summary

Evaluate regex dependencies.

### Current Stack

| Library | Purpose | Status |
|---------|---------|--------|
| regex | Core regex | ✅ |
| fancy-regex | Advanced features | ⚠️ Unused |

### Usage

```rust
use regex::Regex;

let re = Regex::new(r"(\d{4})-(\d{2})-(\d{2})")?;
let caps = re.captures("2024-03-29")?;
```

### Recommendation

1. Pre-compile regexes
2. Use fancy-regex for complex patterns
3. Add benchmark tests

---

## 2026-03-29 - Random Number Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P3

### Summary

Evaluate random number generation dependencies.

### Current Stack

| Library | Purpose | Status |
|---------|---------|--------|
| rand | Core RNG | ✅ |
| rand_chacha | Fast RNG | ✅ (via rand) |

### Usage

```rust
use rand::{Rng, SeedableRng, rngs::OsRng};

let mut rng = OsRng;
let num: u32 = rng.gen();
let nums: Vec<u32> = (0..10).map(|_| rng.gen()).collect();
```

### Recommendation

1. Use OsRng for security
2. Seed deterministic RNGs properly
3. Add benchmark tests

---

## 2026-03-29 - Encoding Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P3

### Summary

Evaluate encoding/decoding dependencies.

### Current Stack

| Library | Purpose | Status |
|---------|---------|--------|
| base64 | Base64 encoding | ✅ (via base64 crate) |
| hex | Hex encoding | ✅ (via hex crate) |
| encoding_rs | Text encoding | ⚠️ Unused |

### Usage

```rust
use base64::{Engine as _, engine::general_purpose::STANDARD};

let encoded = STANDARD.encode(b"hello");
let decoded = STANDARD.decode(&encoded)?;
```

### Recommendation

1. Standardize on base64 crate
2. Add URL-safe variant
3. Implement custom alphabets

---

## 2026-03-29 - URL Dependencies

**Project:** [cross-repo]
**Category:** dependencies
**Status:** pending
**Priority:** P3

### Summary

Evaluate URL handling dependencies.

### Current Stack

| Library | Purpose | Status |
|---------|---------|--------|
| url | URL parsing | ✅ |
| urlencoding | Percent encoding | ✅ |

### Usage

```rust
use url::Url;

let url = Url::parse("https://example.com/path?q=1")?;
let base = Url::parse("https://example.com/")?;
let joined = base.join("relative")?;
```

### Recommendation

1. Use url for all URL handling
2. Validate URLs before use
3. Add query parameter helpers
