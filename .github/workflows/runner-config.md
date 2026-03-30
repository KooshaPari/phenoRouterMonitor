# Local Runner Architecture for phenotype-infrakit

## Overview

This document defines the local runner strategy for the phenotype-infrakit agent-only CI/CD environment. Local runners are strictly reserved for long-running, resource-intensive workloads while maintaining agent-only security constraints.

**Status**: Active
**Last Updated**: 2026-03-30
**Target Audience**: CI/CD automation, local runner operators, agents

---

## I. Local Runner Requirements

### Hardware Specifications

| Component | Minimum | Recommended |
|-----------|---------|------------|
| CPU Cores | 2 | 4+ |
| Memory | 4 GB | 8+ GB |
| Disk Space | 20 GB | 50+ GB |
| Storage Type | HDD | SSD (faster cache, build times) |

### Operating System Support

- **Linux (preferred)**: Ubuntu 20.04 LTS or later, Debian 11+
- **macOS**: 11.0 (Big Sur) or later
- **Not Supported**: Windows (use GitHub-hosted runners instead)

### Network Requirements

- Persistent connection to GitHub Actions (TCP 443)
- Outbound HTTPS access (no incoming ports required)
- Latency: <200ms to GitHub (for responsive job scheduling)

---

## II. Agent-Only Security Posture

### Core Principles

1. **No External Code Execution**: Local runners must never execute arbitrary code from untrusted sources.
2. **Signature Verification Required**: All workflow files triggering local runner execution MUST be reviewed and approved.
3. **Agent-Only Label Enforcement**: Only jobs explicitly labeled `self-hosted-agent-only` can use local runners.
4. **Minimal Token Scope**: GITHUB_TOKEN provisioned to local runners has read-only repo access; no secrets access.
5. **No Persistent Credentials**: Secrets must never be passed to local runner jobs.

### Approval Process

#### Workflow Jobs Using Local Runners

```yaml
jobs:
  benchmark-heavy:
    runs-on: [self-hosted, self-hosted-agent-only, benchmark-heavy]
    # This job requires explicit approval before being scheduled
```

Any new workflow file or job using local runner labels MUST:

1. Be reviewed and merged via PR
2. Include documented justification (why local runner is needed)
3. Be tagged with explicit labels identifying the workload type
4. Reference this runner-config.md in comments

#### Approval Checklist

- [ ] Job definition reviewed (no arbitrary command execution)
- [ ] Justification documented (benchmark >15min, cache-heavy, Rust compilation >30min)
- [ ] Labels correctly set (`self-hosted-agent-only` present)
- [ ] No secrets or credentials passed to job
- [ ] Workflow file committing to main branch

---

## III. GitHub Actions Setup for Local Runners

### Runner Registration

#### Step 1: Generate GitHub Personal Access Token (PAT)

```bash
# In GitHub UI: Settings → Developer settings → Personal access tokens → Tokens (classic)
# Scopes:
#   - repo (read:repo_public, read:repo_private)
#   - workflow
#   - read:org
# DO NOT grant: admin:org, admin:repo_hook, delete_repo, gist, or other high-privilege scopes
```

#### Step 2: Register Local Runner

```bash
# On local runner machine:
cd /opt/actions-runner  # or your runner directory
./config.sh --url https://github.com/KooshaPari/phenotype-infrakit \
  --token <PAT> \
  --name phenotype-runner-1 \
  --labels self-hosted-agent-only,benchmark-heavy,linux
```

#### Step 3: Install as Service (Recommended)

```bash
sudo ./svc.sh install
sudo ./svc.sh start
sudo ./svc.sh status
```

### Token Scoping (GITHUB_TOKEN)

Local runners use the repository's automatic `GITHUB_TOKEN` (generated per workflow run). Token permissions are **automatically scoped** via repository settings:

```yaml
permissions:
  contents: read                # Read-only repository access
  pull-requests: read           # Can read PR data
  # Explicitly denied in local runner environment:
  # secrets: none              # No access to secrets
  # packages: none             # No package write access
  # deployments: none          # No deployment access
```

#### Manual Token Configuration (If Needed)

For agents running sensitive workflows:

```bash
# .github/workflows/benchmark.yml
jobs:
  benchmark:
    runs-on: [self-hosted, self-hosted-agent-only, benchmark-heavy]
    permissions:
      contents: read
      pull-requests: write      # Only if posting results to PR
    env:
      GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      # ^ Automatically available; scope via repository settings
```

### Runner Health Monitoring

#### Built-in Checks

```bash
# On runner machine:
./config.sh --check
systemctl status actions.runner.KooshaPari-phenotype-infrakit.* --no-pager
```

#### Workflow Verification

```yaml
jobs:
  health-check:
    runs-on: [self-hosted, self-hosted-agent-only]
    steps:
      - name: Verify runner health
        run: |
          echo "Runner: $(uname -a)"
          echo "CPU: $(nproc) cores"
          echo "Memory: $(free -h | grep Mem)"
          echo "Disk: $(df -h /)"
```

---

## IV. Local Runner Use Cases

### Approved Use Cases

#### 1. Long-Running Benchmarks (>15 minutes)

**Why Local**: GitHub-hosted runners have 6-hour job timeout; benchmarks >15min need stable hardware for reproducibility.

**Example**:
```yaml
jobs:
  cargo-bench:
    runs-on: [self-hosted, self-hosted-agent-only, benchmark-heavy]
    timeout-minutes: 45
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@nightly
      - run: cargo bench --workspace --bench crate-benchmarks
        continue-on-error: true
```

**Metrics Tracked**:
- Execution time (wall-clock)
- Memory usage (RSS)
- Cache hit rates

#### 2. Cache-Heavy Builds

**Why Local**: Multi-layered caching (sccache, cargo-cache) requires persistent disk state.

**Example**:
```yaml
jobs:
  incremental-build:
    runs-on: [self-hosted, self-hosted-agent-only]
    steps:
      - uses: actions/checkout@v6
      - uses: Swatinem/rust-cache@v2
        with:
          cache-directories: /opt/sccache
      - run: cargo build --all --release
```

**Cache Paths**:
- `/opt/sccache/` (persistent sccache cache)
- `~/.cargo/` (local Rust registry)
- `target/` (workspace build artifacts, excluded from cleanup)

#### 3. Large Rust Compilations (>10 minutes)

**Why Local**: Rust compilation is CPU/memory intensive; local runners provide consistent hardware.

**Example**:
```yaml
jobs:
  compile-release:
    runs-on: [self-hosted, self-hosted-agent-only]
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --all --release
```

**Optimization Flags**:
- `CARGO_BUILD_JOBS=2` (limit parallelism for 4-core machines)
- `RUSTFLAGS="-C opt-level=3"` (aggressive optimization)

### Rejected Use Cases

**Do NOT use local runners for**:

- ❌ Unit tests (< 5 min execution) — use GitHub-hosted Linux
- ❌ Linting, formatting checks — use GitHub-hosted Linux
- ❌ Security scanning (CodeQL, gitleaks) — use GitHub-hosted Linux
- ❌ Arbitrary third-party code — security risk
- ❌ Any job without explicit agent-only label
- ❌ Anything requiring Windows/macOS runners (use GitHub-hosted)

---

## V. Remote Runner Strategy for Public Repos

### Policy: GitHub-Hosted Linux Only

All standard CI gates (test, lint, security) use **GitHub-hosted Linux runners** (`ubuntu-latest`):

```yaml
jobs:
  test:
    runs-on: ubuntu-latest  # ✅ Free tier
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all
```

### Cost Avoidance

| Runner | Cost | Recommendation |
|--------|------|----------------|
| GitHub-hosted Linux | Free | ✅ Use for all CI gates |
| GitHub-hosted macOS | ~$10/minute | ❌ Avoid (prohibitive) |
| GitHub-hosted Windows | ~$6/minute | ❌ Avoid (not needed) |
| Self-hosted (Local) | $0 (if available) | ✅ Use only for approved benchmarks |

### Matrix Strategy for Parallelization

For faster feedback, use matrix strategy across **multiple Linux runners**:

```yaml
jobs:
  test-matrix:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust-version: [stable, nightly]
        crate: [phenotype-cache-adapter, phenotype-policy-engine, phenotype-event-sourcing]
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@${{ matrix.rust-version }}
      - run: cargo test -p ${{ matrix.crate }}
```

This distributes work across **6 concurrent Linux runners** (3 crates × 2 versions) without extra cost.

### Concurrency Settings

```yaml
concurrency:
  group: test-${{ github.ref }}-${{ matrix.crate }}
  cancel-in-progress: true  # Cancel old runs if new push arrives
```

---

## VI. Runner Labels & Job Assignment

### Label Definitions

| Label | Usage | Constraints |
|-------|-------|------------|
| `self-hosted-agent-only` | **Required** for all local runner jobs | Blocks non-agent execution |
| `github-hosted-linux` | Standard CI gates | Free tier |
| `benchmark-heavy` | Long benchmarks >15min | Requires hardware spec validation |
| `cache-persistent` | Build jobs needing cache | Local disk >=50GB |
| `linux` | Linux-only jobs | Filters non-Linux runners |

### Job Label Assignment

#### Example: Benchmark Job (Local Runner)

```yaml
jobs:
  benchmark:
    runs-on: [self-hosted, self-hosted-agent-only, benchmark-heavy, linux]
    timeout-minutes: 60
    steps:
      - uses: actions/checkout@v6
      - run: cargo bench --all
```

#### Example: Standard Test (GitHub-Hosted)

```yaml
jobs:
  test:
    runs-on: ubuntu-latest  # GitHub-hosted (no local labels)
    steps:
      - uses: actions/checkout@v6
      - run: cargo test --all
```

### Label Validation

Every workflow triggering local runners must declare labels in this order:

```yaml
runs-on:
  - self-hosted         # Platform identifier
  - self-hosted-agent-only  # REQUIRED: agent-only constraint
  - benchmark-heavy     # Workload type (optional)
  - linux               # OS filter (optional)
```

Missing `self-hosted-agent-only` will cause the job to **fail gracefully** (with clear error) rather than fall back to GitHub-hosted runners.

---

## VII. Workflow Integration Examples

### Example 1: Benchmark Workflow (Local Runner)

**File**: `.github/workflows/benchmark.yml`

```yaml
name: Benchmarks
on:
  push:
    branches: [main]
  workflow_dispatch:

jobs:
  cargo-bench:
    name: Cargo Bench (Local)
    runs-on: [self-hosted, self-hosted-agent-only, benchmark-heavy, linux]
    timeout-minutes: 60
    permissions:
      contents: read
      pull-requests: write
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@nightly
      - uses: Swatinem/rust-cache@v2
        with:
          cache-directories: /opt/sccache
          cache-on-failure: true
      - name: Run benchmarks
        run: |
          cargo bench --all --bench criterion
        timeout-minutes: 45
      - name: Comment on PR
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: 'Benchmarks completed on local runner.'
            })
```

### Example 2: CI Gates Workflow (GitHub-Hosted)

**File**: `.github/workflows/ci.yml`

```yaml
name: CI
on:
  pull_request:
  push:
    branches: [main]

jobs:
  test:
    name: Test Suite
    runs-on: ubuntu-latest  # GitHub-hosted Linux
    timeout-minutes: 30
    permissions:
      contents: read
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --all
      - run: cargo clippy --all -- -D warnings

  fmt:
    name: Format Check
    runs-on: ubuntu-latest
    permissions:
      contents: read
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --check

  security:
    name: Security Scan
    runs-on: ubuntu-latest
    permissions:
      contents: read
      security-events: write
    steps:
      - uses: actions/checkout@v6
      - uses: github/super-linter@v5
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          DEFAULT_BRANCH: main
```

### Example 3: Stacked Build (Local Cache, GitHub-Hosted Tests)

```yaml
name: Full Build Pipeline
on: [push, pull_request]

jobs:
  # Local runner: cache-persistent build
  build:
    name: Build (Local Cache)
    runs-on: [self-hosted, self-hosted-agent-only, cache-persistent, linux]
    timeout-minutes: 30
    outputs:
      artifact-url: ${{ steps.upload.outputs.artifact-url }}
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --all --release
      - id: upload
        run: echo "artifact-url=${{ env.ARTIFACT_PATH }}" >> $GITHUB_OUTPUT

  # GitHub-hosted: standard tests using built artifacts
  test:
    name: Test Suite
    runs-on: ubuntu-latest
    needs: [build]
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all
```

---

## VIII. Troubleshooting & Maintenance

### Common Issues

#### Issue: Job Never Starts (Idle Runner)

**Symptom**: Job queued but not assigned to runner.

**Diagnosis**:
```bash
# On runner machine:
tail -f /opt/actions-runner/_diag/*.log
ps aux | grep Runner.Listener
```

**Fix**:
```bash
sudo systemctl restart actions.runner.*
# Verify:
sudo systemctl status actions.runner.* --no-pager
```

#### Issue: Cache Not Persisting Across Runs

**Symptom**: Rust cache empty on next job; compilation slow.

**Diagnosis**:
```bash
ls -lah ~/.cargo/
df -h ~/.cargo/
```

**Fix**:
```bash
# Ensure persistent paths are excluded from cleanup:
# In Swatinem/rust-cache:
with:
  cache-directories: /opt/sccache
  cache-on-failure: true  # Keep cache even if job fails
```

#### Issue: Out of Disk Space

**Symptom**: Job fails with "No space left on device".

**Diagnosis**:
```bash
df -h /
du -sh ~/.cargo target/
```

**Cleanup**:
```bash
# Archive old builds:
mv ~/.cargo /archive/cargo.$(date +%Y%m%d)
cargo clean
# Restart runner:
sudo systemctl restart actions.runner.*
```

#### Issue: Unauthorized (403 Forbidden)

**Symptom**: "Unauthorized to use this runner" error.

**Diagnosis**:
```bash
# Check PAT expiration:
curl -H "Authorization: token ${GH_TOKEN}" https://api.github.com/user
```

**Fix**:
```bash
# Regenerate PAT in GitHub UI
./config.sh --unregister --token <NEW_PAT>
./config.sh --url https://github.com/KooshaPari/phenotype-infrakit \
  --token <NEW_PAT> --name phenotype-runner-1 --labels self-hosted-agent-only
sudo systemctl restart actions.runner.*
```

### Maintenance Schedule

| Task | Frequency | Owner |
|------|-----------|-------|
| Check runner logs | Daily | Automation |
| Restart stale runners | Weekly | Automation (systemd timer) |
| Clean up old caches | Monthly | Manual (`cargo clean --target-dir`) |
| Update GitHub Actions runner | Quarterly | Manual (runner self-updates) |
| Audit local runner access | Quarterly | Security team |
| Refresh PAT | Every 12 months | Manual |

---

## IX. Governance & Compliance

### Audit Trail

All local runner job executions are logged:

1. **GitHub Actions Logs**: Available in GitHub UI (repository → Actions tab)
2. **Runner Logs**: Available in `~/_diag/` on runner machine
3. **System Logs**: Available via `journalctl` on Linux runners

### Compliance Checkpoints

- ✅ Only agent-triggered workflows may use local runners
- ✅ All local runner jobs labeled with `self-hosted-agent-only`
- ✅ No credentials or secrets passed to local runner jobs
- ✅ Quarterly access audits (who triggered jobs)
- ✅ Monthly cleanup of obsolete jobs/logs

### Security Review

Any workflow changes affecting local runners require:

1. PR review and approval
2. At least one +1 from codeowners
3. No concurrent force-pushes to main during audit

---

## X. References

- [GitHub Actions: Self-Hosted Runners](https://docs.github.com/en/actions/hosting-your-own-runners/about-self-hosted-runners)
- [GitHub Actions: Runner Security](https://docs.github.com/en/actions/hosting-your-own-runners/security-hardening-for-github-actions)
- [GitHub Actions: Usage Limits](https://docs.github.com/en/actions/learn-github-actions/usage-limits-billing-and-administration)
- [Phenotype-infrakit CLAUDE.md](../CLAUDE.md)
- [GitHub Billing & Cost Management](https://docs.github.com/en/billing/managing-billing-for-github-actions)

---

**Document Status**: Active
**Last Reviewed**: 2026-03-30
**Next Review**: 2026-06-30

