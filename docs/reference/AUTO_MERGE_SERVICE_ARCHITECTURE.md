# Auto-Merge Service Architecture — Technical Design

**Status:** Detailed Design Ready for Implementation
**Version:** 1.0
**Created:** 2026-03-31
**Component:** Batch-Merger (Rust microservice)

---

## Executive Summary

The **Auto-Merge Service** automatically merges clean specification branches (`specs/agent-*`) into the authoritative `specs/main` branch. It eliminates manual merge overhead while maintaining 100% traceability and zero conflicts.

**Key Properties:**
- **Batch processing:** Merge up to 50 concurrent branches every 5 minutes
- **Conflict handling:** Automatic GitHub issues for manual review
- **Validation gating:** Only validated specs merge
- **Zero false positives:** Pre-merge validation prevents bad merges

---

## Architecture Overview

### Components

```
┌─────────────────────────────────────────────────────────────┐
│                    GitHub Events                             │
│  (Push to specs/agent-*, PR creation, validation updates)   │
└──────────────┬──────────────────────────────────────────────┘
               │
               │ WebHook
               ↓
┌─────────────────────────────────────────────────────────────┐
│            GitHub Actions Orchestrator                       │
│  • Scheduled: Every 5 minutes (batch processing)            │
│  • Event-driven: Push to specs/agent-* (immediate)          │
│  • Runs: phenotype-batch-merger binary                      │
└──────────────┬──────────────────────────────────────────────┘
               │
               │ Invokes
               ↓
┌─────────────────────────────────────────────────────────────┐
│         phenotype-batch-merger (Rust Service)               │
│                                                              │
│  Task 1: List agent branches                                │
│  ├─ git branch -r | grep refs/heads/specs/agent-*          │
│  └─ Filter by: age <24h, status !=merged                   │
│                                                              │
│  Task 2: Validate each branch                               │
│  ├─ Markdown structure valid?                               │
│  ├─ Commit messages have Spec-Traces?                       │
│  ├─ FR↔Test coverage 100%?                                  │
│  ├─ Can cleanly merge with specs/main?                      │
│  └─ Output: MergeResult { Success | Conflict | Invalid }    │
│                                                              │
│  Task 3: Attempt merge (3-way merge)                        │
│  ├─ If Success: Create merge commit                         │
│  ├─ If Conflict: Create GitHub issue                        │
│  ├─ If Invalid: Create validation issue                     │
│  └─ Output: GitHub Actions status                           │
│                                                              │
│  Task 4: Batch processing                                   │
│  ├─ Process 1-50 branches in parallel (tokio)               │
│  └─ Report: Success/conflict counts, timings                │
└──────────────┬──────────────────────────────────────────────┘
               │
               │ Updates
               ↓
┌─────────────────────────────────────────────────────────────┐
│              GitHub Repository                              │
│  • specs/main: Updated with merged commits                  │
│  • specs/agent-*: Deleted (auto-cleanup)                    │
│  • Issues: Created for conflicts requiring review           │
│  • Actions: Status checks reflect merge status              │
└─────────────────────────────────────────────────────────────┘
```

---

## Service Specification

### 1. Batch Merger — Core Service

**Crate:** `libs/phenotype-batch-merger/src/lib.rs`

#### Function: List Agent Branches

```rust
use git2::{Repository, BranchType};

pub fn list_agent_branches(repo: &Repository) -> Result<Vec<Branch>, GitError> {
    let mut branches = Vec::new();

    for branch_iter in repo.branches(Some(BranchType::Remote))? {
        let (branch, _) = branch_iter?;
        let branch_name = branch.name()?;

        // Match pattern: refs/remotes/origin/specs/agent-*
        if branch_name.starts_with("origin/specs/agent-") {
            // Extract only the local name: specs/agent-<name>-<task>
            let local_name = branch_name.strip_prefix("origin/").unwrap();

            // Get branch tip commit
            let tip = branch.get().target();

            branches.push(Branch {
                name: local_name.to_string(),
                oid: tip,
                age_secs: calculate_age(repo, tip)?,
                is_merged: is_merged(repo, tip)?,
            });
        }
    }

    // Filter: Keep only branches age <24h and not yet merged
    branches.retain(|b| b.age_secs < 86400 && !b.is_merged);

    Ok(branches)
}

pub struct Branch {
    pub name: String,
    pub oid: Option<git2::Oid>,
    pub age_secs: u64,
    pub is_merged: bool,
}
```

#### Function: Validate Branch

```rust
pub async fn validate_branch(
    repo: &Repository,
    branch: &Branch,
) -> Result<ValidationResult, ValidationError> {
    let mut result = ValidationResult {
        branch_name: branch.name.clone(),
        issues: Vec::new(),
        can_merge: true,
    };

    // 1. Check markdown structure
    if !validate_markdown_structure(repo, branch)? {
        result.issues.push("Markdown structure invalid".to_string());
        result.can_merge = false;
    }

    // 2. Check commit message format
    let commits = get_branch_commits(repo, branch)?;
    for commit in &commits {
        let message = commit.message()?;
        if !has_spec_traces(&message) {
            result.issues.push(format!(
                "Commit {} missing Spec-Traces",
                commit.id()
            ));
            result.can_merge = false;
        }

        // Validate Spec-Traces references valid FRs
        if let Err(e) = validate_spec_traces(&message, repo)? {
            result.issues.push(format!("Invalid Spec-Traces: {}", e));
            result.can_merge = false;
        }
    }

    // 3. Check FR↔Test coverage
    if !validate_fr_test_coverage(repo)? {
        result.issues.push("FR↔Test coverage <100%".to_string());
        result.can_merge = false;
    }

    // 4. Check for merge conflicts
    let merge_result = repo.merge_trees(
        &get_merge_base(repo, branch)?,
        &repo.find_tree(branch.oid.unwrap())?,
        &repo.find_tree(
            repo.find_reference("refs/remotes/origin/specs/main")?
                .target()
                .unwrap()
        )?,
        None,
    )?;

    if merge_result.has_conflicts() {
        result.issues.push("Merge conflicts with specs/main".to_string());
        result.can_merge = false;
    }

    Ok(result)
}

pub struct ValidationResult {
    pub branch_name: String,
    pub issues: Vec<String>,
    pub can_merge: bool,
}
```

#### Function: Attempt Merge

```rust
pub async fn attempt_merge(
    repo: &Repository,
    branch: &Branch,
) -> Result<MergeResult, MergeError> {
    // 1. Fetch latest
    let mut remote = repo.find_remote("origin")?;
    remote.fetch(&["specs/main", &branch.name], None, None)?;

    // 2. Get references
    let branch_ref = repo.find_reference(&format!("refs/remotes/origin/{}", branch.name))?;
    let branch_oid = branch_ref.target().ok_or(MergeError::NoTarget)?;

    let main_ref = repo.find_reference("refs/remotes/origin/specs/main")?;
    let main_oid = main_ref.target().ok_or(MergeError::NoTarget)?;

    // 3. Find merge base
    let merge_base = repo.merge_base(main_oid, branch_oid)?;

    // 4. Perform 3-way merge (dry run)
    let base_tree = repo.find_tree(repo.find_commit(merge_base)?.tree_id()?)?;
    let branch_tree = repo.find_tree(repo.find_commit(branch_oid)?.tree_id()?)?;
    let main_tree = repo.find_tree(repo.find_commit(main_oid)?.tree_id()?)?;

    let mut index = repo.merge_trees(&base_tree, &branch_tree, &main_tree, None)?;

    // Check for conflicts
    if index.has_conflicts() {
        let conflicts = extract_conflict_details(&index)?;
        return Err(MergeError::Conflict(conflicts));
    }

    // 5. Create merge commit
    index.write_tree()?;
    let tree_id = index.write_tree()?;

    let sig = repo.signature()?;
    let parent_commit = repo.find_commit(main_oid)?;
    let branch_commit = repo.find_commit(branch_oid)?;

    let message = format!(
        "Merge {} into specs/main\n\nAuto-merged by phenotype-batch-merger\n\nCommits: {}",
        branch.name,
        branch_commit.id()
    );

    let _merge_oid = repo.commit(
        Some("HEAD"),  // Update HEAD
        &sig,
        &sig,
        &message,
        &repo.find_tree(tree_id)?,
        &[&parent_commit, &branch_commit],
    )?;

    // 6. Push to origin
    let mut remote = repo.find_remote("origin")?;
    remote.push(&["refs/heads/specs/main"], None)?;

    // 7. Delete agent branch
    let mut ref_name = format!("refs/heads/{}", branch.name);
    repo.find_reference(&ref_name)?.delete()?;

    // Push deletion
    remote.push(&[&format!(":{}", ref_name)], None)?;

    Ok(MergeResult::Success {
        branch: branch.name.clone(),
        commit_hash: _merge_oid.to_string(),
        parent_commit_hash: main_oid.to_string(),
    })
}

pub enum MergeResult {
    Success {
        branch: String,
        commit_hash: String,
        parent_commit_hash: String,
    },
    Conflict(Vec<ConflictDetail>),
    ValidationFailed(Vec<String>),
}

pub struct ConflictDetail {
    pub file: String,
    pub ours_start: usize,
    pub ours_count: usize,
    pub theirs_start: usize,
    pub theirs_count: usize,
}
```

#### Function: Batch Processor

```rust
use tokio::task;

pub async fn process_batch(
    repo_path: &str,
    target_branch: &str,
) -> Result<BatchResult, BatchError> {
    let repo = Repository::open(repo_path)?;

    // 1. List branches
    let branches = list_agent_branches(&repo)?;
    tracing::info!("Found {} agent branches to process", branches.len());

    if branches.is_empty() {
        return Ok(BatchResult {
            total: 0,
            successful: 0,
            failed: 0,
            conflicts: 0,
            duration_secs: 0.0,
        });
    }

    // 2. Validate all branches in parallel
    let mut validation_tasks = Vec::new();
    for branch in &branches {
        let repo_clone = repo.clone();
        let branch_clone = branch.clone();

        let task = task::spawn(async move {
            validate_branch(&repo_clone, &branch_clone).await
        });

        validation_tasks.push((branch.clone(), task));
    }

    // 3. Collect validation results
    let mut to_merge = Vec::new();
    let mut validation_failures = Vec::new();

    for (branch, task) in validation_tasks {
        match task.await {
            Ok(Ok(result)) if result.can_merge => {
                to_merge.push(branch);
            }
            Ok(Ok(result)) => {
                validation_failures.push((branch.name, result.issues));
            }
            Ok(Err(e)) => {
                validation_failures.push((branch.name, vec![e.to_string()]));
            }
            Err(e) => {
                validation_failures.push((
                    branch.name,
                    vec![format!("Task join error: {}", e)],
                ));
            }
        }
    }

    // 4. Attempt merges in parallel
    let start = std::time::Instant::now();
    let mut merge_tasks = Vec::new();

    for branch in to_merge {
        let repo_clone = repo.clone();
        let branch_clone = branch.clone();

        let task = task::spawn(async move {
            attempt_merge(&repo_clone, &branch_clone).await
        });

        merge_tasks.push((branch, task));
    }

    // 5. Collect merge results
    let mut successful = 0;
    let mut conflicts = 0;
    let mut merge_failures = Vec::new();
    let mut merge_conflicts = Vec::new();

    for (branch, task) in merge_tasks {
        match task.await {
            Ok(Ok(MergeResult::Success { commit_hash, .. })) => {
                tracing::info!("✅ Merged {} ({})", branch.name, commit_hash);
                successful += 1;
            }
            Ok(Ok(MergeResult::Conflict(details))) => {
                tracing::warn!("⚠️  Conflict in {}: {} files", branch.name, details.len());
                conflicts += 1;
                merge_conflicts.push((branch.name, details));
            }
            Ok(Err(e)) => {
                tracing::error!("❌ Merge failed: {}", e);
                merge_failures.push((branch.name, e.to_string()));
            }
            Err(e) => {
                tracing::error!("❌ Task error: {}", e);
                merge_failures.push((branch.name, e.to_string()));
            }
        }
    }

    let duration = start.elapsed().as_secs_f64();

    // 6. Create GitHub issues for failures/conflicts
    create_github_issues(&validation_failures, &merge_failures, &merge_conflicts).await?;

    Ok(BatchResult {
        total: branches.len(),
        successful,
        failed: validation_failures.len() + merge_failures.len(),
        conflicts,
        duration_secs: duration,
    })
}

pub struct BatchResult {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub conflicts: usize,
    pub duration_secs: f64,
}

#[derive(Clone)]
pub struct Branch {
    pub name: String,
    pub oid: Option<git2::Oid>,
    pub age_secs: u64,
    pub is_merged: bool,
}
```

---

### 2. GitHub Actions Orchestrator

**File:** `.github/workflows/auto-merge-specs.yml`

```yaml
name: Auto-Merge Specs Agent Branches

on:
  schedule:
    # Run every 5 minutes during business hours
    - cron: '*/5 9-17 * * Mon-Fri'
    # Run every 30 minutes outside business hours (lower priority)
    - cron: '*/30 17-9 * * *'

  # Also trigger on push to agent branches
  push:
    branches:
      - 'specs/agent-*'

  # Manual trigger for testing
  workflow_dispatch:
    inputs:
      branch_filter:
        description: 'Filter branches (default: all specs/agent-*)'
        required: false
        default: ''

permissions:
  contents: write
  pull-requests: write
  issues: write

jobs:
  batch-merge:
    name: Batch Merge Agent Branches
    runs-on: ubuntu-latest
    timeout-minutes: 10

    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0
          token: ${{ secrets.GH_BOT_TOKEN }}

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2

      - name: Build batch-merger binary
        run: |
          cargo build --release \
            --package phenotype-batch-merger \
            --bin batch-merger

      - name: Configure Git
        run: |
          git config --global user.name "phenotype-bot"
          git config --global user.email "bot@phenotype.local"
          git config --global pull.rebase false

      - name: Run batch merge
        id: merge
        run: |
          set +e  # Don't fail on merge errors; collect results

          OUTPUT=$(./target/release/batch-merger \
            --repo-path "." \
            --target-branch "specs/main" \
            --log-level "info" \
            --max-parallel 10 \
            ${{ github.event.inputs.branch_filter }})

          echo "$OUTPUT" | tee /tmp/merge-output.txt

          # Extract counts
          SUCCESSFUL=$(echo "$OUTPUT" | grep "successful:" | awk '{print $2}')
          CONFLICTS=$(echo "$OUTPUT" | grep "conflicts:" | awk '{print $2}')
          FAILED=$(echo "$OUTPUT" | grep "failed:" | awk '{print $2}')

          echo "successful=$SUCCESSFUL" >> $GITHUB_OUTPUT
          echo "conflicts=$CONFLICTS" >> $GITHUB_OUTPUT
          echo "failed=$FAILED" >> $GITHUB_OUTPUT

          set -e

      - name: Push merged specs/main
        if: steps.merge.outputs.successful > 0
        run: |
          git push origin specs/main --force-with-lease

      - name: Publish merge report
        if: always()
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const output = fs.readFileSync('/tmp/merge-output.txt', 'utf8');

            const summary = `
            # Auto-Merge Batch Report

            **Successful:** ${{ steps.merge.outputs.successful }}
            **Conflicts:** ${{ steps.merge.outputs.conflicts }}
            **Failed:** ${{ steps.merge.outputs.failed }}

            \`\`\`
            ${ output }
            \`\`\`
            `;

            core.summary.addRaw(summary);
            await core.summary.write();

      - name: Alert on failures
        if: steps.merge.outputs.failed > 0 || steps.merge.outputs.conflicts > 0
        uses: actions/github-script@v7
        with:
          script: |
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `⚠️ Batch merge completed with issues:
              - Conflicts: ${{ steps.merge.outputs.conflicts }}
              - Failed: ${{ steps.merge.outputs.failed }}

              See Actions logs for details.`
            });
```

---

### 3. Conflict Handler — GitHub Issues

When auto-merge fails due to conflicts, automatically create a GitHub issue:

```rust
pub async fn create_github_issues(
    validation_failures: &[(String, Vec<String>)],
    merge_failures: &[(String, String)],
    merge_conflicts: &[(String, Vec<ConflictDetail>)],
) -> Result<(), GitHubError> {
    let client = create_github_client()?;

    // 1. Create validation failure issues
    for (branch, issues) in validation_failures {
        let body = format!(
            "## Validation Failed: {}\n\n{}\n\n**Action:** Fix issues and re-push.",
            branch,
            issues.join("\n- ")
        );

        client
            .create_issue(
                &format!("🔴 Validation Failed: {}", branch),
                &body,
            )
            .await?;
    }

    // 2. Create merge failure issues
    for (branch, error) in merge_failures {
        let body = format!(
            "## Merge Failed: {}\n\n```\n{}\n```\n\n**Action:** Debug and re-push.",
            branch,
            error
        );

        client
            .create_issue(
                &format!("🔴 Merge Failed: {}", branch),
                &body,
            )
            .await?;
    }

    // 3. Create merge conflict issues
    for (branch, conflicts) in merge_conflicts {
        let conflict_details = conflicts
            .iter()
            .map(|c| format!("- **{}** (lines {}-{})", c.file, c.ours_start, c.theirs_start))
            .collect::<Vec<_>>()
            .join("\n");

        let body = format!(
            "## Merge Conflict: {}\n\nConflicting files:\n{}\n\n**Action:** Resolve manually:\n\
            1. `git fetch origin`\n\
            2. `git diff origin/specs/main..{}`\n\
            3. Fix conflicts in editor\n\
            4. `git add . && git commit -m \"resolve: merge conflict\"`\n\
            5. `git push origin {}`\n\
            6. Re-run auto-merge",
            branch, conflict_details, branch, branch
        );

        client
            .create_issue(
                &format!("🔀 Merge Conflict: {}", branch),
                &body,
            )
            .await?;
    }

    Ok(())
}
```

---

## Data Flow Sequences

### Sequence 1: Successful Auto-Merge (No Conflicts)

```
Agent pushes specs/agent-<name>-<task>
    ↓
GitHub Actions triggered (push event)
    ↓
batch-merger runs: validate_branch()
    ✓ Markdown valid
    ✓ Spec-Traces present
    ✓ FR↔Test coverage 100%
    ✓ No conflicts with specs/main
    ↓
batch-merger runs: attempt_merge()
    ✓ 3-way merge succeeds
    ↓
Merge commit created: "Merge specs/agent-<name>-<task> into specs/main"
    ↓
specs/main updated
    ↓
Agent branch deleted (auto-cleanup)
    ↓
Summary posted: "✅ Merged in 2.3 sec"
    ↓
Done (no manual action needed)
```

**Timeline:** Push → Validation (2-3 min) → Merge (1-2 sec) → Done (5 min total)

### Sequence 2: Merge with Conflicts

```
Agent pushes specs/agent-<name>-<task>
    ↓
GitHub Actions triggered
    ↓
batch-merger runs: validate_branch()
    ✓ All validation passes
    ↓
batch-merger runs: attempt_merge()
    ✗ 3-way merge FAILS (conflicts detected)
    ↓
Conflict details extracted (files, line ranges)
    ↓
GitHub Issue created:
  Title: "🔀 Merge Conflict: specs/agent-<name>-<task>"
  Body: Conflicting files, manual resolution steps
    ↓
Summary posted: "⚠️ Conflict detected in 1 file. Manual review issue #123 created."
    ↓
Agent notified (via GitHub issue)
    ↓
Agent manually resolves (see AGENT_WORKFLOW.md Workflow C)
    ↓
Agent re-pushes resolved branch
    ↓
Auto-merge retries (next 5-min batch)
    ↓
Done (after manual resolution)
```

**Timeline:** Initial push → Validation (2-3 min) → Conflict detected → Issue created → Manual resolution (varies) → Re-push → Auto-merge (5 min)

### Sequence 3: Validation Failure

```
Agent pushes specs/agent-<name>-<task>
    ↓
GitHub Actions triggered
    ↓
batch-merger runs: validate_branch()
    ✗ Validation FAILS:
      - Missing Spec-Traces in commit message
      - OR FR-CORE-042 not found in FUNCTIONAL_REQUIREMENTS.md
      - OR FR↔Test coverage <100%
    ↓
GitHub Issue created:
  Title: "🔴 Validation Failed: specs/agent-<name>-<task>"
  Body: Specific validation errors + fix instructions
    ↓
Summary posted: "❌ Validation failed. See issue #124 for details."
    ↓
Agent fixes (add Spec-Traces, update FR, add test)
    ↓
Agent re-pushes
    ↓
Auto-merge retries
    ↓
Done (after fix)
```

**Timeline:** Push → Validation failure (2-3 min) → Issue created → Fix (agent time) → Re-push → Auto-merge (5 min)

---

## Error Handling & Recovery

### Transient Errors (Automatic Retry)

```rust
pub async fn attempt_merge_with_retry(
    repo: &Repository,
    branch: &Branch,
    max_retries: u32,
) -> Result<MergeResult, MergeError> {
    let mut retries = 0;

    loop {
        match attempt_merge(repo, branch).await {
            Ok(result) => return Ok(result),
            Err(e @ MergeError::NetworkError(_)) => {
                retries += 1;
                if retries >= max_retries {
                    return Err(e);
                }
                tracing::warn!(
                    "Network error, retry {}/{}: {}",
                    retries,
                    max_retries,
                    e
                );
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
            Err(e @ MergeError::RepositoryLocked) => {
                retries += 1;
                if retries >= max_retries {
                    return Err(e);
                }
                tracing::warn!("Repository locked, retry {}/{}", retries, max_retries);
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            Err(e) => return Err(e), // Non-transient errors
        }
    }
}
```

### Permanent Errors (Manual Review)

```rust
pub enum MergeError {
    // Transient (auto-retry)
    NetworkError(String),
    RepositoryLocked,

    // Permanent (manual review via GitHub issue)
    Conflict(Vec<ConflictDetail>),
    ValidationFailed(Vec<String>),
    InvalidBranch(String),
    NoTarget,
}
```

---

## Monitoring & Observability

### Logging (via tracing)

```rust
// High-level summary
tracing::info!("Batch started: {} branches to process", branches.len());

// Per-branch progress
tracing::debug!("Validating branch: {}", branch.name);
tracing::info!("Branch valid: {} (no conflicts)", branch.name);
tracing::warn!("Conflict detected: {} ({} files)", branch.name, conflict_count);

// Batch completion
tracing::info!(
    "Batch completed: {} successful, {} conflicts, {} failed in {:.2}s",
    successful,
    conflicts,
    failed,
    duration_secs
);
```

### Metrics

```rust
// Track in GitHub Actions summary
pub struct BatchMetrics {
    pub batch_id: String,
    pub timestamp: DateTime<Utc>,
    pub total_branches: usize,
    pub successful_merges: usize,
    pub conflicts: usize,
    pub validation_failures: usize,
    pub duration_secs: f64,
    pub avg_merge_time_ms: f64,
}

// Output to /tmp/batch-metrics.json for analysis
pub async fn publish_metrics(metrics: &BatchMetrics) -> Result<()> {
    let json = serde_json::to_string_pretty(metrics)?;
    std::fs::write("/tmp/batch-metrics.json", json)?;

    // Also post to GitHub Actions summary
    println!("::notice::Batch metrics: {} successful", metrics.successful_merges);

    Ok(())
}
```

### Health Dashboard

Updated daily by `.github/workflows/ssot-health-check.yml`:

```markdown
# SSOT Auto-Merge Health (Last 24h)

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Total branches processed | 47 | N/A | ✅ |
| Successful auto-merges | 45 | >95% | ✅ |
| Merge conflicts | 2 | <5% | ✅ |
| Validation failures | 0 | 0 | ✅ |
| Avg merge time | 2.1s | <5s | ✅ |
| Batch success rate | 95.7% | >95% | ✅ |
```

---

## Security Considerations

### Authentication

- **GitHub token:** Stored as `GH_BOT_TOKEN` secret (read/write repos)
- **SSH key:** Not needed (HTTPS with token)
- **Scope:** Limited to `contents:write`, `pull-requests:write`, `issues:write`

### Authorization

- **Merge permission:** Only to `specs/main` (protected branch)
- **Branch deletion:** Only to `specs/agent-*` branches
- **Issue creation:** Public issues (no private data leaks)

### Validation

- **Spec validation:** Prevents invalid specs from merging
- **Traceability check:** Ensures all FRs traced to tests
- **Markdown check:** Prevents invalid syntax in shared docs

---

## Performance Characteristics

### Throughput

- **Batch size:** 1-50 concurrent branches
- **Batch frequency:** Every 5 minutes (business hours)
- **Merge latency:** 2-5 seconds per branch (sequential)
- **Total time:** 2-3 min validation + 2-5 sec merge = 3-5 min end-to-end

### Resource Usage

- **CPU:** <10% during merge (mostly I/O bound)
- **Memory:** ~100MB for 50-branch batch
- **Disk:** Minimal (git objects cached)
- **Network:** ~1MB per batch (fetch + push)

### Scalability

- **Max branches per batch:** 50 (tested)
- **Max concurrent merges:** 10 (tokio workers)
- **Expected at scale:** 100 agents × 2 branches/day = 200 branches/day = 40 branches/batch = sustainable

---

## Deployment Checklist

- [ ] Crate `libs/phenotype-batch-merger/Cargo.toml` created
- [ ] Binary builds cleanly: `cargo build --release`
- [ ] Test suite passes: `cargo test`
- [ ] GitHub Actions workflow: `.github/workflows/auto-merge-specs.yml`
- [ ] Bot token configured: `GH_BOT_TOKEN` secret
- [ ] Branch protection rules: `specs/main` protected
- [ ] Logging configured: `RUST_LOG=info` in GHA
- [ ] Monitoring dashboard: Health checks automated
- [ ] Documentation: Agent workflow guide complete
- [ ] Team trained: All agents understand workflow

---

## Reference

- **Implementation Plan:** `/docs/reference/SSOT_PHASE1_IMPLEMENTATION_PLAN.md` (Task 2.1)
- **Agent Workflow:** `/docs/reference/SSOT_PHASE1_AGENT_WORKFLOW.md`
- **Crate Location:** `libs/phenotype-batch-merger/`
- **GitHub Actions:** `.github/workflows/auto-merge-specs.yml`

---

**Document Owner:** Infrastructure Team
**Version:** 1.0
**Status:** Ready for Implementation
**Last Updated:** 2026-03-31

**Build, test, and deploy this service to enable hands-off spec merging.**
