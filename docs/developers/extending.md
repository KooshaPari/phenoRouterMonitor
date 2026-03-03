---
audience: [developers]
---

# Extending AgilePlus

AgilePlus uses a port-based architecture that makes it straightforward to add integrations for storage, VCS, issue trackers, and agent harnesses. This guide walks through implementing each extension type.

## Architecture Overview

Ports define the interface; adapters implement it:

```
┌─ Domain Layer (no external dependencies) ─┐
│  Spec, Plan, WorkPackage, Requirements   │
└───────────────────┬───────────────────────┘
                    │ uses
                    ↓
      ┌─ Ports (traits) ─────────────┐
      │ StoragePort                   │
      │ VcsPort                       │
      │ SyncPort                      │
      │ AgentPort                     │
      └─ ▲──────────────────────────┘
         │ implements
         │
      ┌──┴────────────────────────────────────┐
      │      Adapters (implementations)       │
      ├────────────────────────────────────────┤
      │ File Storage    | PostgreSQL Adapter   │
      │ Git VCS         | Mercurial Adapter    │
      │ Plane Sync      | GitHub Sync Adapter  │
      │ Claude Agent    | Codex Agent Adapter  │
      └────────────────────────────────────────┘
```

## Extension Points

| Extension | Trait | Purpose | Example Impl |
|-----------|-------|---------|--------------|
| **Storage** | `StoragePort` | Persist specs, plans, tasks | FileStorage, PostgreSQL |
| **VCS** | `VcsPort` | Git operations, branches | GitVcs, MercurialVcs |
| **Tracker Sync** | `SyncPort` | Issue tracker integration | PlaneSync, GitHubSync |
| **Agent Harness** | `AgentPort` | AI agent communication | ClaudeAgent, CodexAgent |

## Adding a Storage Backend

Storage adapters handle persistence of specs, plans, and tasks.

### Define the Implementation

```rust
// crates/agileplus-adapters/src/storage/postgresql.rs

use agileplus_ports::StoragePort;
use sqlx::PgPool;

pub struct PostgresqlStorage {
    pool: PgPool,
}

impl PostgresqlStorage {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }
}

impl StoragePort for PostgresqlStorage {
    async fn read_spec(&self, feature_id: &FeatureId) -> Result<Spec> {
        let row = sqlx::query_as::<_, (String, String)>(
            "SELECT title, content FROM specs WHERE feature_id = $1"
        )
        .bind(feature_id.as_str())
        .fetch_one(&self.pool)
        .await?;

        Ok(Spec::parse(&row.1)?)
    }

    async fn write_spec(&self, feature_id: &FeatureId, spec: &Spec) -> Result<()> {
        sqlx::query(
            "INSERT INTO specs (feature_id, title, content) VALUES ($1, $2, $3)
             ON CONFLICT(feature_id) DO UPDATE SET content = $3"
        )
        .bind(feature_id.as_str())
        .bind(&spec.title)
        .bind(spec.render())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_features(&self) -> Result<Vec<FeatureId>> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT feature_id FROM specs ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(FeatureId::new).collect())
    }

    // Implement read_plan, write_plan, etc.
}
```

### Register in Dependency Container

```rust
// crates/agileplus-cli/src/main.rs

use agileplus_adapters::storage::PostgresqlStorage;
use agileplus_ports::StoragePort;

#[tokio::main]
async fn main() -> Result<()> {
    let config = load_config()?;

    // Register storage adapter based on config
    let storage: Box<dyn StoragePort> = match config.storage.backend.as_str() {
        "file" => Box::new(FileStorage::new(&config.storage.path)?),
        "postgresql" => Box::new(PostgresqlStorage::new(&config.database_url).await?),
        other => panic!("Unknown storage backend: {}", other),
    };

    // Storage is now available throughout the app
    let engine = Engine::new(storage);
    // ...
}
```

## Adding a VCS Provider

VCS adapters handle git operations (or other version control).

### Implement VcsPort

```rust
// crates/agileplus-adapters/src/vcs/mercurial.rs

use agileplus_ports::VcsPort;
use std::process::Command;

pub struct MercurialVcs {
    repo_path: PathBuf,
}

impl MercurialVcs {
    pub fn new(repo_path: PathBuf) -> Self {
        Self { repo_path }
    }

    fn hg(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("hg")
            .current_dir(&self.repo_path)
            .args(args)
            .output()?;

        if !output.status.success() {
            return Err(format!("hg failed: {:?}", String::from_utf8(output.stderr))?);
        }

        Ok(String::from_utf8(output.stdout)?)
    }
}

impl VcsPort for MercurialVcs {
    fn create_branch(&self, name: &str, base: &str) -> Result<()> {
        // Mercurial: create bookmark instead of branch
        self.hg(&["bookmark", "-r", base, name])?;
        Ok(())
    }

    fn create_worktree(&self, path: &Path, branch: &str) -> Result<()> {
        // Mercurial: clone with specific revision
        self.hg(&["clone", "-r", branch, ".", path.to_string_lossy().as_ref()])?;
        Ok(())
    }

    fn commit(&self, message: &str, files: &[PathBuf]) -> Result<CommitId> {
        let file_args: Vec<&str> = files
            .iter()
            .map(|p| p.to_string_lossy().as_ref())
            .collect();

        self.hg(&["add"] + &file_args)?;
        self.hg(&["commit", "-m", message])?;

        // Get the commit hash
        let hash = self.hg(&["log", "-r", ".", "--template", "{node}"])?;
        Ok(CommitId::from(hash))
    }

    fn merge(&self, source: &str, target: &str) -> Result<MergeResult> {
        // Checkout target branch
        self.hg(&["checkout", target])?;

        // Merge source into target
        match self.hg(&["merge", source]) {
            Ok(_) => {
                // Merge succeeded
                self.hg(&["commit", "-m", &format!("Merge {} into {}", source, target)])?;
                Ok(MergeResult::Success)
            }
            Err(_) => {
                // Conflicts detected
                Ok(MergeResult::Conflicts)
            }
        }
    }
}
```

## Adding a Tracker Integration

Tracker adapters sync with issue management systems like Jira, Azure DevOps, etc.

### Implement SyncPort

```rust
// crates/agileplus-adapters/src/sync/jira.rs

use agileplus_ports::SyncPort;
use reqwest::Client;

pub struct JiraSync {
    client: Client,
    base_url: String,
    api_token: String,
}

impl JiraSync {
    pub fn new(base_url: &str, api_token: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            api_token: api_token.to_string(),
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_token)
    }

    async fn make_request(&self, method: &str, path: &str, body: Option<&str>) -> Result<String> {
        let url = format!("{}/rest/api/3{}", self.base_url, path);

        let response = match method {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            _ => return Err(format!("Unknown method: {}", method).into()),
        }
        .header("Authorization", self.auth_header())
        .header("Content-Type", "application/json")
        .send()
        .await?;

        Ok(response.text().await?)
    }
}

impl SyncPort for JiraSync {
    async fn push_issues(&self, issues: &[Issue]) -> Result<()> {
        for issue in issues {
            let body = serde_json::json!({
                "fields": {
                    "project": { "key": "AGILE" },
                    "summary": &issue.title,
                    "description": &issue.description,
                    "issuetype": { "name": map_issue_type(&issue.issue_type) },
                    "priority": { "name": map_priority(&issue.priority) },
                }
            });

            self.make_request("POST", "/issues", Some(&body.to_string())).await?;
        }
        Ok(())
    }

    async fn pull_issues(&self) -> Result<Vec<Issue>> {
        let response = self.make_request("GET", "/search?jql=project=AGILE", None).await?;
        let json: serde_json::Value = serde_json::from_str(&response)?;

        let mut issues = Vec::new();
        for issue in json["issues"].as_array().unwrap_or(&vec![]) {
            issues.push(Issue {
                id: IssueId::from(issue["key"].as_str().unwrap()),
                title: issue["fields"]["summary"].as_str().unwrap().to_string(),
                description: issue["fields"]["description"].as_str().unwrap_or("").to_string(),
                status: issue["fields"]["status"]["name"].as_str().unwrap().to_string(),
                issue_type: parse_issue_type(issue["fields"]["issuetype"]["name"].as_str().unwrap()),
                // ... map other fields
            });
        }
        Ok(issues)
    }

    async fn update_status(&self, id: &IssueId, status: Status) -> Result<()> {
        let body = serde_json::json!({
            "fields": {
                "status": { "name": status.to_string() }
            }
        });

        self.make_request("PUT", &format!("/issues/{}", id.as_str()), Some(&body.to_string())).await?;
        Ok(())
    }
}

fn map_issue_type(issue_type: &IssueType) -> &str {
    match issue_type {
        IssueType::Bug => "Bug",
        IssueType::Feature => "Task",
        IssueType::Task => "Task",
        IssueType::Idea => "Story",
    }
}
```

## Plugin Discovery & Registration

### Configuration-Based Registration

Adapters are registered in `.kittify/config.yaml`:

```yaml
# .kittify/config.yaml

storage:
  backend: postgresql  # or: file, postgresql
  database_url: postgresql://localhost/agileplus_dev

vcs:
  backend: git  # or: git, mercurial

sync:
  backends:
    - type: plane
      api_key: ${PLANE_API_KEY}
      workspace_id: agileplus

    - type: github
      api_token: ${GITHUB_TOKEN}
      repo_owner: KooshaPari
      repo_name: AgilePlus

agent:
  harness: claude-code  # or: claude-code, codex
  timeout_seconds: 3600
```

### Runtime Discovery

```rust
// crates/agileplus-cli/src/registry.rs

pub fn create_storage(config: &Config) -> Result<Box<dyn StoragePort>> {
    match config.storage.backend.as_str() {
        "file" => Ok(Box::new(FileStorage::new(&config.storage.path)?)),
        "postgresql" => {
            let db = PostgresqlStorage::new(&config.storage.database_url).await?;
            Ok(Box::new(db))
        }
        other => Err(format!("Unknown storage backend: {}", other).into()),
    }
}

pub fn create_vcs(config: &Config) -> Result<Box<dyn VcsPort>> {
    match config.vcs.backend.as_str() {
        "git" => Ok(Box::new(GitVcs::new("."))),
        "mercurial" => Ok(Box::new(MercurialVcs::new("."))),
        other => Err(format!("Unknown VCS: {}", other).into()),
    }
}

pub fn create_sync_backends(config: &Config) -> Result<Vec<Box<dyn SyncPort>>> {
    let mut backends = Vec::new();

    for sync_config in &config.sync.backends {
        match sync_config.sync_type.as_str() {
            "plane" => {
                let adapter = PlaneSync::new(&sync_config.config)?;
                backends.push(Box::new(adapter) as Box<dyn SyncPort>);
            }
            "github" => {
                let adapter = GitHubSync::new(&sync_config.config)?;
                backends.push(Box::new(adapter) as Box<dyn SyncPort>);
            }
            other => eprintln!("Unknown sync backend: {}", other),
        }
    }

    Ok(backends)
}
```

## Testing Custom Adapters

Write tests against the port trait:

```rust
// crates/agileplus-adapters/tests/storage_test.rs

use agileplus_adapters::storage::PostgresqlStorage;
use agileplus_ports::StoragePort;
use agileplus_core::Spec;

#[tokio::test]
async fn postgresql_storage_roundtrips() {
    let storage = PostgresqlStorage::new("postgresql://localhost/test").await.unwrap();
    let feature_id = FeatureId::new("001");
    let spec = Spec::new("Test Feature");

    // Write
    storage.write_spec(&feature_id, &spec).await.unwrap();

    // Read
    let retrieved = storage.read_spec(&feature_id).await.unwrap();
    assert_eq!(retrieved.title, spec.title);
}
```

## Best Practices

1. **Keep ports simple** — Traits should have 5–10 methods max
2. **Error types** — Use consistent error handling across adapters
3. **Async support** — Use `async/await` for I/O operations
4. **Testing** — Unit test each adapter independently
5. **Documentation** — Document configuration requirements
6. **Logging** — Use structured logging for debugging
7. **Performance** — Add caching where appropriate (e.g., for read-heavy operations)

## Adding a New CLI Subcommand

New subcommands are defined using `clap` derive macros and registered in `crates/agileplus-cli/src/commands/`:

### 1. Define the command struct

```rust
// crates/agileplus-cli/src/commands/analyze.rs

use clap::Args;

/// Analyze feature artifacts for consistency issues
#[derive(Args, Debug)]
pub struct AnalyzeArgs {
    /// Feature slug to analyze
    #[arg(value_name = "FEATURE")]
    pub feature: String,

    /// Check cross-artifact consistency (spec vs plan vs WPs)
    #[arg(long, default_value = "true")]
    pub cross_check: bool,

    /// Output format (text or json)
    #[arg(long, default_value = "text")]
    pub output: String,
}

pub async fn execute(args: AnalyzeArgs, ctx: &Context) -> anyhow::Result<()> {
    let feature = ctx.storage
        .get_feature_by_slug(&args.feature)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Feature not found: {}", args.feature))?;

    // Delegate to engine (thin handler)
    let report = ctx.engine.analyze_feature(&feature, args.cross_check).await?;

    match args.output.as_str() {
        "json" => println!("{}", serde_json::to_string_pretty(&report)?),
        _ => print_text_report(&report),
    }

    Ok(())
}
```

### 2. Register in the main CLI

```rust
// crates/agileplus-cli/src/main.rs

#[derive(Subcommand)]
enum Commands {
    Specify(commands::specify::SpecifyArgs),
    Plan(commands::plan::PlanArgs),
    // ... existing commands ...

    /// Analyze feature artifacts for consistency
    Analyze(commands::analyze::AnalyzeArgs),
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    let ctx = Context::new(&cli.global).await?;

    match cli.command {
        Commands::Specify(args) => commands::specify::execute(args, &ctx).await,
        Commands::Plan(args) => commands::plan::execute(args, &ctx).await,
        // ... existing arms ...
        Commands::Analyze(args) => commands::analyze::execute(args, &ctx).await,
    }
}
```

### 3. Add engine logic

```rust
// crates/agileplus-engine/src/analyze.rs

use agileplus_domain::domain::feature::Feature;

pub struct AnalysisReport {
    pub feature_slug: String,
    pub issues: Vec<ConsistencyIssue>,
    pub score: u8,  // 0-100
}

pub struct ConsistencyIssue {
    pub severity: Severity,
    pub description: String,
    pub location: String,  // "spec.md line 42"
}

impl Engine {
    pub async fn analyze_feature(
        &self,
        feature: &Feature,
        cross_check: bool,
    ) -> Result<AnalysisReport, DomainError> {
        let spec = self.vcs.read_artifact(&feature.slug, "spec.md").await?;
        let plan = self.vcs.read_artifact(&feature.slug, "plan.md").await.ok();
        let wps = self.storage.list_wps_by_feature(feature.id).await?;

        let mut issues = Vec::new();

        if cross_check {
            // Check that every FR in the spec is covered by at least one WP
            issues.extend(check_fr_coverage(&spec, &wps));
            // Check that plan references same files as WP scopes
            if let Some(plan) = &plan {
                issues.extend(check_plan_wp_alignment(plan, &wps));
            }
        }

        let score = compute_quality_score(&issues);

        Ok(AnalysisReport {
            feature_slug: feature.slug.clone(),
            issues,
            score,
        })
    }
}
```

## Extending the Event System (NATS)

New domain events are defined in `crates/agileplus-domain/src/events.rs` and published through the engine's event publisher:

### 1. Define the event type

```rust
// crates/agileplus-domain/src/events.rs

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DomainEvent {
    FeatureStateChanged {
        feature_slug: String,
        from: FeatureState,
        to: FeatureState,
        actor: String,
        timestamp: DateTime<Utc>,
        audit_entry_id: i64,
    },
    WorkPackageStateChanged {
        feature_slug: String,
        wp_id: String,
        from: WpState,
        to: WpState,
        actor: String,
        timestamp: DateTime<Utc>,
    },
    AgentCompleted {
        job_id: String,
        feature_slug: String,
        wp_id: String,
        success: bool,
        commits: Vec<String>,
    },

    // Add your new event here:
    AnalysisCompleted {
        feature_slug: String,
        score: u8,
        issue_count: u32,
        duration_ms: u64,
    },
}
```

### 2. Publish from the engine

```rust
// crates/agileplus-engine/src/analyze.rs

impl Engine {
    pub async fn analyze_feature(...) -> Result<AnalysisReport> {
        // ... analysis logic ...

        // Publish event
        self.event_publisher.publish(DomainEvent::AnalysisCompleted {
            feature_slug: feature.slug.clone(),
            score: report.score,
            issue_count: report.issues.len() as u32,
            duration_ms: elapsed.as_millis() as u64,
        }).await?;

        Ok(report)
    }
}
```

### 3. Subscribe in other components (e.g., dashboard SSE)

```rust
// crates/agileplus-api/src/events.rs

async fn sse_stream(state: AppState) -> impl IntoResponse {
    let mut subscription = state.nats.subscribe("agileplus.>").await?;

    Sse::new(async_stream::stream! {
        while let Some(msg) = subscription.next().await {
            let event: DomainEvent = serde_json::from_slice(&msg.payload)?;
            yield Ok(Event::default()
                .data(serde_json::to_string(&event)?)
                .event(event.type_name()));
        }
    })
}
```

## Next Steps

- [Contributing](contributing.md) — Development setup and PR workflow
- [Testing](testing.md) — Test patterns for adapters and commands
- [Architecture Overview](../architecture/overview.md) — Crate structure
- [Storage Port](../sdk/storage-port.md) — StoragePort API reference
- [VCS Port](../sdk/vcs-port.md) — VcsPort API reference
- [Harness Integration](../agents/harness-integration.md) — Adding agent adapters
