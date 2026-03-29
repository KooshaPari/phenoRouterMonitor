# UX/AX/DX Worklogs

**Category:** UX/AX/DX | **Updated:** 2026-03-29

---

## 2026-03-29 - Deep UX Audit: CLI User Experience

**Project:** [AgilePlus]
**Category:** ux
**Status:** in_progress
**Priority:** P1

### Summary

Deep audit of CLI user experience, shell integration, and interactive terminal improvements.

### CLI UX Analysis

#### Current CLI Commands (agileplus)

```
agileplus
├── cycle          # Manage cycles
├── branch         # Branch management
├── specify        # Create/revise specs
├── research       # Research features
├── plan           # Generate work packages
├── implement      # Implement work packages
├── validate       # Governance compliance
├── ship           # Merge validated features
├── retrospective   # Generate reports
├── triage         # Classify incoming items
├── queue          # Manage triage backlog
├── module         # Manage modules
├── dashboard      # Web dashboard
└── platform       # Platform services
```

#### UX Strengths

1. ✅ Comprehensive command coverage
2. ✅ Clap-based argument parsing with help
3. ✅ Consistent command structure
4. ✅ Global flags (--db, --repo, --verbose)

#### UX Weaknesses

1. ⚠️ No shell completion (bash/zsh/fish)
2. ⚠️ No interactive mode (TUI)
3. ⚠️ Error messages could be more helpful
4. ⚠️ No progress indicators for long operations
5. ⚠️ No colorized output

### UX Improvements Needed

#### High Priority

1. **Shell Completion**
   ```bash
   # Missing: --completions flag
   agileplus --completions bash > /etc/bash_completion.d/agileplus
   agileplus --completions zsh > _agileplus
   ```

2. **Interactive Mode**
   ```rust
   // Missing: Interactive TUI for specify, plan, validate
   // Could use: ratatui, cursive, or tui-rs
   ```

3. **Progress Indicators**
   ```rust
   // Missing: Progress bars for long operations
   // Example: git operations, API calls
   indicatif::ProgressBar::new(100);
   ```

4. **Colorized Output**
   ```rust
   // Missing: Colored terminal output
   // Could use: ansi_term, colored, yansi
   anstream::auto();
   ```

#### Medium Priority

5. **Enhanced Error Messages**
   ```rust
   // Current: anyhow error chain
   // Better: Context-aware suggestions
   ```

6. **Command Aliases**
   ```bash
   # Missing: Short aliases
   agileplus spec     # instead of specify
   agileplus impl     # instead of implement
   agileplus val      # instead of validate
   ```

7. **Configuration File**
   ```toml
   # ~/.agileplus/config.toml
   [defaults]
   db = "~/.agileplus/db.sqlite"
   repo = "~/projects"

   [ui]
   color = true
   progress = true
   pager = "less"
   ```

### Action Items

- [ ] 🔴 CRITICAL: Add shell completion (bash, zsh, fish)
- [ ] 🔴 CRITICAL: Add progress indicators for long operations
- [ ] 🟡 HIGH: Add interactive TUI mode
- [ ] 🟡 HIGH: Colorize terminal output
- [ ] 🟠 MEDIUM: Add command aliases
- [ ] 🟠 MEDIUM: Create config file support
- [ ] 🟢 LOW: Add vim-style navigation

### Related

- `crates/agileplus-cli/src/main.rs`
- `crates/agileplus-cli/src/commands/`

---

## 2026-03-29 - AX Audit: Agent Experience

**Project:** [AgilePlus]
**Category:** ax
**Status:** in_progress
**Priority:** P1

### Summary

Audit of agent experience - how agents interact with the system, API design for agents, and agent tooling.

### Agent Interaction Patterns

#### Current: Stub Agent

```rust
// Current: Stub implementation
// Location: crates/agileplus-cli/src/agent_stub.rs
struct StubAgentAdapter;

// Future: Real agent adapter
// See: agileplus-agent-dispatch (planned)
```

#### Agent API Surface

| Endpoint | Purpose | Status |
|----------|---------|--------|
| `/api/features` | Feature CRUD | ✅ |
| `/api/cycles` | Cycle management | ✅ |
| `/api/events` | Event stream | ✅ |
| `/api/validate` | Governance check | ✅ |
| `/api/ship` | Merge PRs | ✅ |

### AX Improvements Needed

#### High Priority

1. **Streaming Responses**
   ```rust
   // Missing: Server-Sent Events for long operations
   // Example: validate, ship commands
   async fn ship_stream() -> impl Stream<Item = ShipEvent> {
       // Progress updates as JSON lines
   }
   ```

2. **Agent-Friendly Errors**
   ```json
   // Current: Human-readable errors
   // Better: Structured error codes for agents
   {
     "error": {
       "code": "GOV-001",
       "message": "Work package governance check failed",
       "details": {
         "rule": "code-coverage",
         "actual": 45,
         "required": 80
       },
       "fix": "Add tests to reach 80% coverage"
     }
   }
   ```

3. **Batch Operations**
   ```rust
   // Missing: Bulk operations for agents
   // Example: Implement multiple WPs at once
   POST /api/work-packages/batch
   ```

#### Medium Priority

4. **Agent Context Injection**
   ```rust
   // Missing: Standardized context for agents
   // Could use: OpenAI function calling format
   ```

5. **Progress Webhooks**
   ```rust
   // Missing: Webhook callbacks for long operations
   // Could notify agent of progress via HTTP POST
   ```

### Action Items

- [ ] 🔴 CRITICAL: Add structured error codes for agents
- [ ] 🔴 CRITICAL: Add streaming SSE responses
- [ ] 🟡 HIGH: Add batch operation endpoints
- [ ] 🟡 HIGH: Add progress webhooks
- [ ] 🟠 MEDIUM: Add agent context protocol

### Related

- `crates/agileplus-api/src/`
- `crates/agileplus-cli/src/agent_stub.rs`

---

## 2026-03-29 - DX Audit: Developer Experience

**Project:** [AgilePlus]
**Category:** dx
**Status:** in_progress
**Priority:** P1

### Summary

Audit of developer experience - onboarding, documentation, IDE support, and build tooling.

### DX Analysis

#### Current Developer Tools

| Tool | Purpose | Status |
|------|---------|--------|
| Cargo | Rust build | ✅ Good |
| Cargo watch | Auto-rebuild | ⚠️ Manual |
| rust-analyzer | IDE support | ✅ Good |
| Cargo deny | License/Security | ⚠️ Missing |
| Cargo outdated | Dep updates | ⚠️ Manual |

### DX Strengths

1. ✅ Cargo workspace organization
2. ✅ rust-analyzer integration
3. ✅ Clear crate boundaries
4. ✅adr support

### DX Weaknesses

1. ❌ No `cargo-dist` for releases
2. ❌ No `cargo-watch` in default workflow
3. ❌ Slow CI (no caching)
4. ❌ Missing devcontainer
5. ❌ No pre-commit hooks

### DX Improvements Needed

#### High Priority

1. **Dev Container**
   ```dockerfile
   # .devcontainer/Dockerfile
   # Missing: Development environment container
   # Should include: Rust, git, docker, gh cli
   ```

2. **Pre-commit Hooks**
   ```yaml
   # .pre-commit-config.yaml
   # Missing: Standardized pre-commit hooks
   # Should include: cargo fmt, cargo clippy, ruff
   ```

3. **Fast CI with Caching**
   ```yaml
   # .github/workflows/ci.yml
   # Missing: sccache, cargo cache
   # Should cache: target/, ~/.cargo/registry/
   ```

4. **Cargo Dist for Releases**
   ```toml
   # Missing: cargo-dist integration
   # Would provide: Cross-platform releases, installers
   ```

#### Medium Priority

5. **IDE Debugging Support**
   ```vscode
   // .vscode/launch.json
   // Missing: Debug configurations for CLI
   ```

6. **Documentation Generator**
   ```rust
   // Missing: cargo doc --document-private-items
   // Missing: docs.rs integration
   ```

7. **ASCIINEMA Recording**
   ```bash
   # Missing: Demo recording scripts
   # For: README, onboarding
   ```

### Action Items

- [ ] 🔴 CRITICAL: Add .devcontainer for development
- [ ] 🔴 CRITICAL: Add pre-commit hooks
- [ ] 🔴 CRITICAL: Improve CI caching
- [ ] 🟡 HIGH: Add cargo-dist for releases
- [ ] 🟡 HIGH: Add VSCode debug configs
- [ ] 🟠 MEDIUM: Add documentation generation
- [ ] 🟠 MEDIUM: Create demo scripts

### Related

- `crates/agileplus-cli/`
- `.github/workflows/`

---

## 2026-03-29 - Shell Integration & Completions

**Project:** [AgilePlus]
**Category:** dx
**Status:** pending
**Priority:** P2

### Summary

Shell integration analysis and completion generation for better CLI experience.

### Current State

```bash
# Currently no completions available
$ agileplus <TAB>
# Nothing happens
```

### Required Completions

#### Bash

```bash
# Commands
agileplus specify
agileplus plan
agileplus implement
agileplus validate
agileplus ship

# Subcommand completion
agileplus branch <TAB>
# checkout create delete list sync

# Flag completion
agileplus --<TAB>
# --db --repo --verbose --version --help
```

#### Zsh

```zsh
# Better completion with descriptions
agileplus specify [Create or revise a feature specification]
```

#### Fish

```fish
# Fish-style completions
agileplus specify --<TAB>
```

### Implementation

Using `clap_complete`:

```rust
use clap_complete::{Generator, Shell};

// In main.rs
fn complete<G: Generator>(cmd: &Command, name: &str) {
    clap_complete::generate(G, cmd, name, &mut std::io::stdout());
}

// Add subcommand
#[derive(Subcommand)]
enum Commands {
    /// Generate shell completions
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
}
```

### Action Items

- [ ] 🟡 HIGH: Add bash completions
- [ ] 🟡 HIGH: Add zsh completions
- [ ] 🟠 MEDIUM: Add fish completions
- [ ] 🟠 MEDIUM: Document completion installation

### Related

- `crates/agileplus-cli/src/main.rs`
- https://docs.rs/clap_complete

---

## 2026-03-29 - Interactive TUI Opportunities

**Project:** [AgilePlus]
**Category:** ux
**Status:** pending
**Priority:** P2

### Summary

Interactive TUI opportunities for better terminal experience.

### TUI Candidates

| Command | TUI Benefit | Effort |
|---------|-------------|--------|
| `agileplus specify` | Interactive spec editor | Medium |
| `agileplus plan` | Visual WP board | High |
| `agileplus validate` | Live progress | Low |
| `agileplus queue` | Triage kanban | High |

### TUI Libraries

| Library | Language | Pros | Cons |
|---------|----------|------|------|
| ratatui | Rust | Async-friendly | Newer |
| tui-rs | Rust | Stable | Deprecated |
| cursive | Rust | Simple | Sync only |
| Textual | Python | Rich | Separate lang |

### Recommendation

**Use ratatui** (tui-rs successor) for Rust TUI:

```rust
use ratatui::{Terminal, Frame};
use ratatui::widgets::*;

// Example: Live validation progress
fn draw_validation_ui<B: Backend>(f: &mut Frame<B>, state: &ValidationState) {
    let gauge = Gauge::default()
        .block(Block::default().title("Governance Check"))
        .fill(state.percentage)
        .label(format!("{}%", state.percentage));
    f.render_widget(gauge, area);
}
```

### Action Items

- [ ] 🟠 MEDIUM: Add TUI for validate command
- [ ] 🟠 MEDIUM: Add TUI for specify command
- [ ] 🟢 LOW: Add TUI for plan command

### Related

- `crates/agileplus-cli/src/commands/validate.rs`
- https://ratatui.rs/

---

## 2026-03-29 - Error Message UX

**Project:** [AgilePlus]
**Category:** ux
**Status:** pending
**Priority:** P2

### Summary

Error message UX improvements for better developer experience.

### Current Error Style

```bash
$ agileplus specify --title ""
Error: Invalid value for "--title": cannot be empty

$ agileplus validate --wp WP1
Error: validation failed: governance rule GOV-001 not met
```

### Improved Error Style

```bash
$ agileplus specify --title ""
Error: --title cannot be empty

  → A feature specification requires a non-empty title
  → Example: agileplus specify --title "Add user authentication"

$ agileplus validate --wp WP1
Error: governance check failed (GOV-001)

  The following rules were not met:
    ✗ Code coverage: 45% (required: 80%)
    ✗ Test count: 5 (required: 10)

  Run: agileplus validate --fix to auto-fix issues
```

### Error Message Guidelines

1. **Concise primary message**
2. **Actionable suggestions**
3. **Command to reproduce/fix**
4. **Link to documentation**

### Action Items

- [ ] 🟠 MEDIUM: Improve error messages across CLI
- [ ] 🟠 MEDIUM: Add --fix flag to validate
- [ ] 🟢 LOW: Create error message style guide

### Related

- `crates/agileplus-cli/src/main.rs`

---

## 2026-03-29 - Documentation UX

**Project:** [AgilePlus]
**Category:** dx
**Status:** pending
**Priority:** P2

### Summary

Documentation experience improvements for developers and users.

### Documentation Gaps

| Doc | Status | Priority |
|-----|--------|----------|
| README | ⚠️ Basic | Medium |
| CLI Help | ✅ Good | - |
| Architecture | ⚠️ Scattered | High |
| API Reference | ⚠️ Missing | High |
| Tutorials | ❌ None | High |

### Documentation Improvements

#### High Priority

1. **API Reference**
   ```
   Missing: OpenAPI/Swagger docs
   Should: Generate from code annotations
   Tool:  utoipa, poem-openapi
   ```

2. **Architecture Guide**
   ```
   Missing: System architecture overview
   Should: ADR-based architecture docs
   Tool:  adr-tools, mdbook
   ```

3. **Tutorials**
   ```
   Missing: Getting started guide
   Missing: Feature walkthrough
   Missing: Video demos
   ```

#### Medium Priority

4. **Cheat Sheet**
   ```markdown
   # Quick reference for common commands
   # PDF/HTML format for wall display
   ```

5. **Troubleshooting Guide**
   ```markdown
   # Common errors and fixes
   # Debug techniques
   ```

### Action Items

- [ ] 🔴 CRITICAL: Add OpenAPI documentation
- [ ] 🔴 CRITICAL: Create getting started tutorial
- [ ] 🟡 HIGH: Add architecture documentation
- [ ] 🟡 HIGH: Create troubleshooting guide
- [ ] 🟠 MEDIUM: Add cheat sheet

### Related

- `docs/`
- `ADR.md`

---

## 2026-03-29 - Polish & QOL Enhancements

**Project:** [AgilePlus]
**Category:** polish
**Status:** pending
**Priority:** P2

### Summary

Quality of life enhancements for daily developer experience.

### QOL Improvements

#### 1. Faster Builds

```bash
# Current: Full rebuild
cargo build

# Better: Use sccache
export RUSTC_WRAPPER=sccache
cargo build

# Better: Use mold linker
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=mold
```

#### 2. Editor Integration

```vscode
// .vscode/settings.json
{
  "editor.formatOnSave": true,
  "rust-analyzer.cargo.features": ["all"],
  "rust-analyzer.checkOnSave.command": "clippy"
}
```

#### 3. Git Aliases

```bash
# ~/.gitconfig
[alias]
  ap = "!agileplus"
  ap-spec = "!agileplus specify"
  ap-plan = "!agileplus plan"
  ap-impl = "!agileplus implement"
```

#### 4. Shell Functions

```bash
# ~/.bashrc or ~/.zshrc
apcheck() {
  agileplus validate --wp "$1" --fix && agileplus ship --wp "$1"
}
```

### Action Items

- [ ] 🟠 MEDIUM: Document sccache setup
- [ ] 🟠 MEDIUM: Share VSCode settings
- [ ] 🟢 LOW: Add git aliases guide
- [ ] 🟢 LOW: Add shell functions guide

### Related

- Developer setup docs

---

---

## 2026-03-29 - Developer Experience (DX) Patterns (Extended)

**Project:** [cross-repo]
**Category:** ux_dx
**Status:** completed
**Priority:** P1

### 1. CLI UX Best Practices

#### Progress Indicators

```rust
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

pub fn create_progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );
    pb.set_message(message.to_string());
    pb
}
```

#### Interactive Prompts

```rust
use dialoguer::{Confirm, Input, Select, MultiSelect};

pub fn prompt_confirm(message: &str) -> bool {
    Confirm::new()
        .with_prompt(message)
        .default(true)
        .interact()
        .unwrap_or(false)
}

pub fn prompt_choice<T: Clone + std::fmt::Display>(
    message: &str,
    items: &[T],
) -> Option<usize> {
    Select::new()
        .with_prompt(message)
        .items(items)
        .default(0)
        .interact()
        .ok()
}
```

---

### 2. Error Output Patterns

```rust
use color_eyre::{Report, Help, Section};
use miette::{Diagnostic, LabeledSpan};

pub fn pretty_error<E: Diagnostic>(err: E) {
    eprintln!("{:?}", err);
}

pub fn error_with_context(err: anyhow::Error, context: &str) -> Report {
    Report::new(err)
        .with_section(move || {
            Help::new().text(context)
        })
        .note("See logs for more details")
}
```

---

### 3. Structured Logging

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(data), fields(data_len = data.len()))]
pub async fn process_data(data: Vec<u8>) -> Result<()> {
    info!("Starting data processing");
    
    if data.is_empty() {
        warn!("Empty data received");
        return Ok(());
    }
    
    info!(records = data.len(), "Processing records");
    
    // Process...
    
    Ok(())
}
```

---

### 4. Configuration UX

```rust
use clap::{Parser, ValueHint};
use figment::{Figment, providers::{Format, Toml, Env, Namespace}};

#[derive(Parser, Debug)]
#[command(name = "phenotype")]
#[command(about = "Phenotype CLI", long_about = None)]
struct Args {
    /// Config file path
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    config: Option<PathBuf>,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
}

impl Args {
    pub fn figment(&self) -> Figment {
        let mut figment = Figment::new();
        
        // File config
        if let Some(config) = &self.config {
            figment = figment.merge(Toml::file(config));
        }
        
        // Environment
        figment = figment.merge(Env::prefixed("PHENOTYPE_"));
        
        // CLI args
        figment = figment.merge(Namespace::from(self));
        
        figment
    }
}
```

---

### 5. Shell Completion

```rust
use clap_complete::{Generator, Shell};

pub fn generate_completion<G: Generator>(gen: G, name: &str, cmd: &mut Command) {
    clap_complete::generate(
        gen,
        cmd,
        name,
        &mut std::io::stdout(),
    );
}

pub fn register_completions() {
    clap_complete::generate(
        clap_complete::Shell::Bash,
        &mut cmd(),
        "phenotype",
        &mut std::io::stdout(),
    );
}
```

---

### 6. Tutorial/Onboarding

```rust
pub struct Onboarding {
    steps: Vec<OnboardingStep>,
}

pub struct OnboardingStep {
    pub title: String,
    pub description: String,
    pub action: Box<dyn Fn() -> Result<()>>,
    pub validation: Box<dyn Fn() -> bool>,
}

impl Onboarding {
    pub fn new() -> Self {
        Self {
            steps: vec![
                OnboardingStep {
                    title: "Configure Git".into(),
                    description: "Set up your git identity".into(),
                    action: Box::new(|| configure_git()),
                    validation: Box::new(|| validate_git_config()),
                },
                OnboardingStep {
                    title: "Initialize workspace".into(),
                    description: "Create Phenotype workspace".into(),
                    action: Box::new(|| init_workspace()),
                    validation: Box::new(|| check_workspace()),
                },
            ],
        }
    }
}
```

---

_Last updated: 2026-03-29_

---

## 2026-03-30 - Modern TUI Frameworks Comparison

**Project:** [cross-repo]
**Category:** ux
**Status:** completed
**Priority:** P2

### Summary

Comparison of modern TUI frameworks for building rich terminal user interfaces in Rust.

### Framework Comparison

| Framework | Language | Status | Async | Widgets | Assessment |
|-----------|----------|--------|-------|---------|------------|
| **ratatui** | Rust | Active (2026) | Yes | 20+ | ✅ RECOMMENDED |
| **cursive** | Rust | Active | No | 15+ | 🟡 Alternative |
| **textual** | Python | Active | Yes | 50+ | 🟡 Alternative |
| **bubbletea** | Go | Active | Yes | 30+ | 🟡 Alternative |
| **chafa** | C | Active | No | Low | ❌ Avoid |

### ratatui Deep Dive (Recommended)

**What:** Declarative TUI framework (successor to tui-rs)

**Key Features:**
- 100% thread-safe
- Layout engine (block, flex, grid)
- Widget library (paragraph, table, chart, gauge)
- Cross-platform (Windows, macOS, Linux)
- 60fps rendering
- Multiple backends (crossterm, termion, ncurses)

**Hello World:**
```rust
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph},
    layout::{Layout, Direction, Constraint},
};
use std::io::stdout;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;
    
    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default()
            .title("Hello")
            .borders(Borders::ALL);
        f.render_widget(block, size);
    })?;
    
    Ok(())
}
```

**Async Integration:**
```rust
use ratatui::async_pipe::{Receiver, Sender};

async fn async_render(rx: Receiver<AppEvent>) {
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).unwrap();
    
    loop {
        tokio::select! {
            Some(event) = rx.recv() => {
                terminal.draw(|f| ui(f, &event))?;
            }
            _ = tokio::time::sleep(Duration::from_millis(16)) => {
                // 60fps render loop
            }
        }
    }
}
```

### Cursive Deep Dive

**What:** Simple TUI framework for Rust

**Key Features:**
- Simple API
- Good documentation
- Synchronous only
- Event handling

**Hello World:**
```rust
use cursive::{Cursive, views::TextView};

fn main() {
    let mut siv = Cursive::new();
    siv.add_layer(TextView::new("Hello World!"));
    siv.run();
}
```

### textual (Python)

**What:** Rich TUI framework in Python

**Key Features:**
- CSS-like styling
- React-like reactivity
- Built-in widgets
- Async support

**Hello World:**
```python
from textual.app import App
from textual.widgets import Static

class HelloApp(App):
    def compose(self):
        yield Static("Hello, World!")

if __name__ == "__main__":
    app = HelloApp()
    app.run()
```

### Phenotype TUI Opportunities

| Command | TUI Benefit | ratatui Effort | Status |
|---------|-------------|----------------|--------|
| `agileplus validate` | Live progress | Low | TODO |
| `agileplus specify` | Interactive form | Medium | TODO |
| `agileplus plan` | Visual board | High | TODO |
| `agileplus queue` | Kanban view | High | TODO |

### Migration Plan

1. **Phase 1:** Add ratatui for validation progress (P2)
2. **Phase 2:** Add TUI for specify command (P2)
3. **Phase 3:** Add Kanban board for queue (P3)

### Action Items

- [ ] Add ratatui to Cargo.toml
- [ ] Create validation progress TUI
- [ ] Add interactive form for specify
- [ ] Implement Kanban board for queue

---

## 2026-03-30 - Agent Experience (AX) 2026 Patterns

**Project:** [cross-repo]
**Category:** ax
**Status:** completed
**Priority:** P1

### Summary

2026 patterns for building excellent agent experiences - how AI agents interact with software systems.

### Agent Interaction Patterns

#### 1. Structured Output

**Current:** Natural language responses
```json
{
  "message": "The feature was created successfully. The ID is abc123."
}
```

**Better:** Structured JSON with schema
```json
{
  "status": "success",
  "data": {
    "feature_id": "abc123",
    "title": "User authentication",
    "created_at": "2026-03-30T10:00:00Z"
  },
  "meta": {
    "request_id": "req_xyz789",
    "duration_ms": 45
  }
}
```

#### 2. Error Codes for Agents

**Current:** Human-readable
```
Error: governance check failed
Rule GOV-001 not met: code coverage 45% < 80%
```

**Better:** Machine-readable with fix hints
```json
{
  "error": {
    "code": "GOV_001",
    "message": "Governance check failed",
    "details": {
      "rule": "code-coverage",
      "actual": 45,
      "required": 80,
      "delta": -35
    },
    "fix": {
      "suggestion": "Add 35% more test coverage",
      "command": "agileplus validate --fix --rule code-coverage",
      "estimated_effort": "2 hours"
    }
  }
}
```

#### 3. Streaming Responses

**Current:** Batch response
```json
{
  "results": ["item1", "item2", "item3"]
}
```

**Better:** Server-Sent Events (SSE)
```
event: progress
data: {"current": 1, "total": 10, "message": "Processing item 1"}

event: progress
data: {"current": 2, "total": 10, "message": "Processing item 2"}

event: result
data: {"item": "item2", "status": "success"}

event: complete
data: {"total": 10, "successful": 9, "failed": 1}
```

#### 4. Tool Calling Protocol

**Standard:** OpenAI function calling format
```json
{
  "tool_calls": [
    {
      "id": "call_abc123",
      "type": "function",
      "function": {
        "name": "create_feature",
        "arguments": {
          "title": "User authentication",
          "description": "Add OAuth2 support",
          "priority": "high"
        }
      }
    }
  ]
}
```

### MCP (Model Context Protocol)

**What:** Standard protocol for AI tool integration

**Benefits:**
- 70%+ of AI applications use MCP
- Vendor-neutral tool definitions
- Streaming support built-in
- Tool versioning

**Phenotype MCP Integration:**
```rust
// phenotype-mcp-server/src/lib.rs

use model_context_protocol::{
    Server, Tool, Resource, Prompt,
};

pub struct PhenotypeMcpServer {
    server: Server,
}

impl PhenotypeMcpServer {
    pub fn new() -> Self {
        let server = Server::new("phenotype");
        
        // Register tools
        server.add_tool(Tool::new(
            "create_feature",
            "Create a new feature specification",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "title": {"type": "string"},
                    "description": {"type": "string"},
                    "priority": {"type": "string", "enum": ["low", "medium", "high"]}
                },
                "required": ["title"]
            }),
            handler: create_feature_handler,
        ));
        
        Self { server }
    }
    
    pub async fn run(self) {
        self.server.run().await;
    }
}
```

### Batch Operations

**Single:**
```rust
POST /api/features
{"title": "Feature A"}
```

**Batch:**
```rust
POST /api/features/batch
{
  "operations": [
    {"op": "create", "data": {"title": "Feature A"}},
    {"op": "create", "data": {"title": "Feature B"}},
    {"op": "update", "id": "feat_123", "data": {"priority": "high"}},
    {"op": "delete", "id": "feat_456"}
  ]
}

// Response
{
  "results": [
    {"op": 0, "status": "created", "id": "feat_789"},
    {"op": 1, "status": "created", "id": "feat_790"},
    {"op": 2, "status": "updated"},
    {"op": 3, "status": "deleted"}
  ]
}
```

### Progress Webhooks

```rust
// Agent registers for progress updates
POST /api/webhooks
{
  "url": "https://agent.example.com/webhook",
  "events": ["feature.created", "validation.progress", "validation.complete"],
  "secret": "agent_secret_xyz"
}

// Server sends progress
POST https://agent.example.com/webhook
{
  "event": "validation.progress",
  "data": {
    "feature_id": "feat_123",
    "progress": 45,
    "total_checks": 20,
    "completed_checks": 9,
    "current_check": "code-coverage"
  }
}
```

### Action Items

- [ ] Add structured error codes to API
- [ ] Implement SSE for long-running operations
- [ ] Add batch operation endpoints
- [ ] Create MCP server for agent tools
- [ ] Add webhook support for progress

---

## 2026-03-30 - Developer Onboarding Experience

**Project:** [cross-repo]
**Category:** dx
**Status:** completed
**Priority:** P1

### Summary

Best practices for developer onboarding - getting new developers productive quickly.

### Onboarding Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Time to first build | < 5 min | From clone to `cargo build` |
| Time to first test | < 10 min | First `cargo test` |
| Time to first contribution | < 1 hour | First PR merged |
| Documentation coverage | > 80% | `cargo doc --document-private-items` |

### Onboarding Checklist

#### Day 1 Setup

- [ ] Clone repository
- [ ] Install Rust (via rustup)
- [ ] Configure git (name, email, signing)
- [ ] Install recommended tools
- [ ] Run initial build
- [ ] Run tests
- [ ] Read architecture overview
- [ ] Make first trivial change
- [ ] Submit first PR

#### Tool Installation

```bash
# Core tools
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Recommended tools
cargo install cargo-nextest cargo-watch cargo-dist sccache

# IDE
code --install-extension rust-lang.rust-analyzer

# Pre-commit
pip install pre-commit
pre-commit install
```

### Onboarding Automation

#### Setup Script

```bash
#!/bin/bash
# scripts/onboard.sh

set -e

echo "🚀 Starting Phenotype onboarding..."

# Check prerequisites
command -v rustc >/dev/null 2>&1 || { echo "Rust not found"; exit 1; }

# Clone
git clone https://github.com/phenotype/phenotype.git
cd phenotype

# Install tools
cargo install cargo-nextest cargo-watch sccache

# Setup pre-commit
pip install pre-commit
pre-commit install

# Build
cargo build

# Test
cargo nextest run --workspace

# Setup git hooks
cp .githooks/* .git/hooks/

echo "✅ Onboarding complete!"
echo "Next: cargo doc --serve"
```

#### Dev Container

```dockerfile
# .devcontainer/Dockerfile
FROM rust:1.76

# Install system dependencies
RUN apt-get update && apt-get install -y \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Rust tools
RUN cargo install cargo-nextest cargo-watch sccache

# Install CLI tools
RUN curl -fsSL https://bun.sh/install | bash
RUN npm install -g pnpm

# Configure
RUN rustup default stable
RUN rustup component add rustfmt clippy rust-src

# VSCode extensions
COPY .devcontainer extensions.json /workspace/.vscode/

WORKDIR /workspace
```

#### VSCode Tasks

```json
// .vscode/tasks.json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Build",
      "type": "shell",
      "command": "cargo build",
      "group": "build"
    },
    {
      "label": "Test",
      "type": "shell",
      "command": "cargo nextest run",
      "group": "test"
    },
    {
      "label": "Watch",
      "type": "shell",
      "command": "cargo watch -x build -x test",
      "group": "none",
      "isBackground": true
    },
    {
      "label": "Doc",
      "type": "shell",
      "command": "cargo doc --open",
      "group": "none"
    }
  ]
}
```

### Documentation Structure

```
docs/
├── README.md                 # Quick start
├── ARCHITECTURE.md          # System overview
├── CONTRIBUTING.md           # How to contribute
├── ONBOARDING.md            # Day 1 guide
├── TROUBLESHOOTING.md       # Common issues
├── CLI.md                   # Command reference
├── API.md                   # API reference
└── adr/                    # Architecture decisions
```

### First Contribution Guide

```markdown
# Your First Contribution

## Finding Issues

Look for issues labeled:
- `good first issue`
- `help wanted`
- `documentation`

## Workflow

1. Fork the repository
2. Create a branch: `git checkout -b feat/my-feature`
3. Make your changes
4. Test: `cargo nextest run`
5. Format: `cargo fmt`
6. Lint: `cargo clippy`
7. Commit: `git commit -am 'feat: add my feature'`
8. Push: `git push origin feat/my-feature`
9. Open PR

## PR Template

```markdown
## Summary
Brief description of the change

## Motivation
Why is this change needed?

## Testing
How was this tested?

## Checklist
- [ ] Tests added
- [ ] Documentation updated
- [ ] No clippy warnings
```
```

### Action Items

- [ ] Create `.devcontainer/` for onboarding
- [ ] Add `scripts/onboard.sh`
- [ ] Create ONBOARDING.md
- [ ] Add first-contribution guide
- [ ] Set up automated setup verification

---

_Last updated: 2026-03-30_
