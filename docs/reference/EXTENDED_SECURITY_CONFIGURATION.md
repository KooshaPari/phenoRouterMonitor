# Extended Security Configuration

**Status**: ✅ Implemented
**Date**: 2026-03-30
**Workflow**: `.github/workflows/extended-security.yml`

## Overview

The phenotype-infrakit repository now implements a **hyper-strict, multi-layered security scanning system** that prevents HIGH and CRITICAL severity issues from reaching the main branch. This document describes the security architecture, scanner configuration, and enforcement policies.

## Security Architecture (5 Layers)

```
┌────────────────────────────────────────────────────────────────────────────┐
│                      Security Perimeter                                    │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│  Layer 1: Secret Detection     → gitleaks + trufflehog                    │
│  Layer 2: SAST (Code Analysis) → CodeQL (6 langs) + Semgrep + Lang-      │
│                                   specific (bandit, gosec, brakeman,     │
│                                   psalm)                                  │
│  Layer 3: SCA (Dependencies)   → pip-audit, npm audit, govulncheck,      │
│                                   cargo-audit                             │
│  Layer 4: Supply Chain         → syft SBOM, OSV-Scanner lockfiles        │
│  Layer 5: Infrastructure       → tfsec, hadolint, trivy                  │
│                                                                            │
│  Hard Gate: security-summary job blocks on HIGH/CRITICAL findings         │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

## Detailed Scanner Configuration

### Layer 1: Secret Detection

#### Gitleaks (Regex-Based)
- **Tool**: gitleaks/gitleaks-action@v2
- **Scope**: Entire repository history (fetch-depth: 0)
- **Triggers**: Push to main, PRs, scheduled daily, on-demand
- **Enforcement**: Blocks on detection (CRITICAL severity)
- **Output**: GitHub Security tab

**Configuration**:
```yaml
- uses: gitleaks/gitleaks-action@v2
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

#### Trufflehog (Verified Secrets)
- **Tool**: trufflesecurity/trufflehog@main
- **Scope**: Git diff between base and head branches
- **Verification**: Only reports verified secrets (flags that tool verified ownership)
- **Enforcement**: Advisory (does not block; surfaces high-fidelity findings)
- **Output**: Workflow logs + GitHub Security tab

**Configuration**:
```yaml
- uses: trufflesecurity/trufflehog@main
  with:
    path: ./
    base: ${{ github.event.repository.default_branch }}
    head: HEAD
    extra_args: --debug --only-verified
```

---

### Layer 2: SAST (Static Application Security Testing)

#### CodeQL (GitHub-Native)
- **Tool**: github/codeql-action
- **Languages Analyzed**:
  - ✅ Go
  - ✅ Python
  - ✅ JavaScript/TypeScript
  - ✅ C++
  - ✅ Java
  - ✅ Ruby
- **Build Mode**: Auto-build (detects language and build system)
- **Fail Policy**: `fail-on: high` (blocks on HIGH/CRITICAL)
- **Schedule**: Daily (Mon 12pm UTC) + PR/push triggers
- **Output**: GitHub Security tab (SARIF format)

**Configuration**:
```yaml
strategy:
  fail-fast: false
  matrix:
    language: [go, python, javascript-typescript, cpp, java, ruby]

steps:
  - uses: github/codeql-action/init@v3
    with:
      languages: ${{ matrix.language }}
      build-mode: auto
  - uses: github/codeql-action/autobuild@v3
  - uses: github/codeql-action/analyze@v3
    with:
      fail-on: high
```

#### Semgrep (Generic SAST)
- **Tool**: returntocorp/semgrep (container)
- **Rulesets**:
  - `p/security-audit` — General security patterns
  - `p/owasp-top-ten` — OWASP Top 10 2021
  - `p/cwe-top-25` — CWE Top 25 Most Dangerous
- **Severity Filter**: HIGH + CRITICAL only
- **Fail Policy**: `fail-on: high`
- **Output**: SARIF to GitHub Security tab

**Configuration**:
```yaml
semgrep --config=p/security-audit \
  --config=p/owasp-top-ten \
  --config=p/cwe-top-25 \
  --format=sarif \
  --severity=HIGH \
  --severity=CRITICAL \
  .
```

#### Language-Specific SAST Scanners

##### Python: bandit
- **Scope**: `python/` directory
- **Level**: `-ll` (only HIGH + CRITICAL)
- **Output**: JSON + display in logs
- **Conditional**: Runs on phenotype and agileplus repositories

**Configuration**:
```yaml
bandit -r python/ -ll -f json --output /tmp/bandit.json
```

##### Go: gosec
- **Scope**: All Go packages (`./...`)
- **Format**: SARIF (uploaded to GitHub Security)
- **Fail Policy**: No-fail mode (advisory) + manual verification
- **Conditional**: Runs on thegent and phenotype

**Configuration**:
```yaml
- uses: securego/gosec@master
  with:
    args: '-no-fail -fmt sarif -out /tmp/gosec-sarif.sarif ./...'
```

##### Ruby: brakeman
- **Scope**: All Ruby files (recursive)
- **Confidence Level**: Medium and above
- **Output**: JSON
- **Conditional**: Only if `**/*.rb` files exist

**Configuration**:
```yaml
brakeman --format json --output /tmp/brakeman-results.json
```

##### PHP: psalm
- **Scope**: Project root (if composer.json exists)
- **Format**: JSON
- **Conditional**: Only if PHP files detected
- **Note**: Requires composer for dependency management

**Configuration**:
```yaml
./vendor/bin/psalm --output-format=json
```

---

### Layer 3: Software Composition Analysis (Dependency Scanning)

#### Python: pip-audit
- **Scope**: Python requirements, pyproject.toml, poetry.lock, Pipfile
- **Severity**: All (displays full details)
- **Output**: JSON + human-readable display
- **Conditional**: Only if Python dependency files exist
- **Fail Policy**: Advisory (reports but doesn't block)

**Configuration**:
```yaml
pip-audit --desc --format json --output /tmp/pip-audit.json
```

#### JavaScript: npm audit
- **Scope**: package-lock.json, pnpm-lock.yaml, yarn.lock
- **Audit Level**: `high` (HIGH + CRITICAL only)
- **Output**: JSON + truncated display (head -100)
- **Conditional**: Only if lockfiles exist
- **Fail Policy**: Reports on HIGH findings

**Configuration**:
```yaml
npm audit --audit-level=high --json > /tmp/npm-audit.json
```

#### Go: govulncheck
- **Scope**: All Go modules (`./...`)
- **Database**: Official Go vulnerability database
- **Output**: Console + exit code
- **Conditional**: Only if `go.mod` exists
- **Fail Policy**: Advisory

**Configuration**:
```yaml
go install golang.org/x/vuln/cmd/govulncheck@latest
govulncheck ./...
```

#### Rust: cargo-audit
- **Scope**: Cargo workspace
- **Tool**: rustsec/audit-check@v2.0.0 (GitHub action)
- **Fail Policy**: **Blocks on vulnerability detection** (CRITICAL)
- **Additional**: cargo-deny for license compliance + duplicate detection

**Configuration**:
```yaml
- uses: rustsec/audit-check@v2.0.0
  with:
    token: ${{ secrets.GITHUB_TOKEN }}
```

---

### Layer 4: Supply Chain Security (Artifact & Lockfile Scanning)

#### Syft (SBOM Generation)
- **Tool**: anchore/sbom-action
- **Format**: SPDX-JSON (industry standard)
- **Scope**: Entire repository
- **Output**: Artifact upload (`sbom-spdx.json`)
- **Purpose**: Supply chain transparency; enables vulnerability correlation
- **Fail Policy**: Advisory (artifact upload only)

**Configuration**:
```yaml
- uses: anchore/sbom-action@v0
  with:
    path: .
    format: spdx-json
    output-file: sbom-spdx.json
```

#### OSV-Scanner (Open Source Vulnerabilities)
- **Tool**: google/osv-scanner
- **Scope**: Cargo.lock (Rust lockfile)
- **Pre-step**: Generate Cargo.lock (if not committed)
- **Format**: SARIF
- **Output**: GitHub Security tab
- **Fail Policy**: Reports on detection (advisory)

**Configuration**:
```yaml
osv-scanner scan --lockfile=Cargo.lock --format sarif --output /tmp/osv-sarif.sarif
```

---

### Layer 5: Infrastructure Security (Config & Container Scanning)

#### tfsec (Terraform)
- **Tool**: aquasecurity/tfsec-action
- **Scope**: All `*.tf` files
- **Format**: SARIF
- **Conditional**: Only if Terraform files exist
- **Fail Policy**: Advisory
- **Checks**: AWS, Azure, GCP IaC misconfiguration detection

**Configuration**:
```yaml
- uses: aquasecurity/tfsec-action@v1.0.0
  with:
    working_directory: .
    format: sarif
    out_file: /tmp/tfsec-sarif.sarif
```

#### hadolint (Dockerfile Linting)
- **Tool**: hadolint/hadolint-action
- **Scope**: All Dockerfiles (recursive)
- **Format**: SARIF
- **Conditional**: Only if Dockerfile(s) exist
- **Fail Policy**: Advisory
- **Checks**: Best practices, caching, layer optimization, security hints

**Configuration**:
```yaml
- uses: hadolint/hadolint-action@v3.1.0
  with:
    recursive: true
    format: sarif
    output-file: /tmp/hadolint-sarif.sarif
```

#### Trivy (Container & Filesystem Scanning)
- **Tool**: aquasecurity/trivy-action
- **Scope**: Filesystem (OS packages) + container images
- **Format**: SARIF
- **Conditional**: If Dockerfile(s) or docker-compose.yml exist
- **Fail Policy**: Advisory
- **Checks**: Known CVEs, misconfigurations, secrets

**Configuration**:
```yaml
- uses: aquasecurity/trivy-action@master
  with:
    scan-type: fs
    scan-ref: .
    format: sarif
    output: /tmp/trivy-sarif.sarif
```

---

## Enforcement Policy: Hard Security Gate

### security-summary Job

**Purpose**: Acts as a gating function blocking PRs/pushes on critical findings.

**Dependencies** (must all succeed):
- `codeql`
- `semgrep`
- `dependency-cargo`
- `secrets-gitleaks`

**Checks** (exits with code 1 if any fail):
1. CodeQL failure → blocks
2. Semgrep failure → blocks
3. Cargo-audit failure → blocks
4. Gitleaks failure → blocks

**Example Gate Logic**:
```bash
if [ "${{ needs.codeql.result }}" = "failure" ]; then
  echo "CRITICAL: CodeQL detected HIGH/CRITICAL issues"
  exit 1
fi
```

**Output on Success**:
```
✓ All security gates passed
├─ CodeQL: success
├─ Semgrep: success
├─ Cargo Audit: success
└─ Gitleaks: success
```

---

## Trigger Schedule

| Trigger | Frequency | Context |
|---------|-----------|---------|
| **Push to main** | Every commit | Immediate feedback; blocks merge if fails |
| **Pull requests** | Every PR | Gating function before merge |
| **Scheduled** | Daily 3am UTC | Catches transitive vulnerabilities + new CVEs |
| **Manual (workflow_dispatch)** | On-demand | For specific investigations or re-scans |

---

## Result Upload & Visualization

### SARIF Format (GitHub Security Tab)

All SAST, SCA, and infrastructure scanners output SARIF (Static Analysis Results Format), which GitHub parses and displays in:
- **Security** → **Code scanning alerts** tab
- Organized by severity (Critical, High, Medium, Low)
- Click-through to source code + remediation guidance
- Tracks resolution (open, dismissed, fixed)

**Workflow**:
```yaml
- uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: /tmp/[scanner]-sarif.sarif
    category: [scanner-name]
```

### Artifacts

- **SBOM (syft)**: Uploaded to Actions artifacts as `sbom-spdx.json`
- **Bandit/pip-audit/npm audit**: JSON reports in temp files (available during workflow execution)
- **Logs**: All scanner outputs visible in workflow step logs

---

## Integration with CI/CD

### Branch Protection

To enforce this security gate:

1. **Go to repository Settings** → **Branches** → **main** → **Branch protection rules**
2. **Add required status check**: `Extended Security / security-summary`
3. **Enable**: "Require branches to be up to date before merging"
4. **Result**: PRs cannot merge until ALL security jobs succeed

### Example GitHub API Command

```bash
gh api repos/KooshaPari/phenotype-infrakit/branches/main/protection/required_status_checks \
  -X PATCH \
  -f checks='[{"context": "Extended Security / security-summary"}]'
```

---

## Failure Handling & Remediation

### When CodeQL Detects HIGH/CRITICAL Issues

1. **PR Workflow blocks** (cannot merge)
2. **Developer reviews alerts** in **Security** → **Code scanning alerts**
3. **Two options**:
   - **Fix the issue**: Push new commits; workflow re-runs automatically
   - **Dismiss**: If false positive, mark as "not applicable" + document reason

### When Dependency Scan Fails

1. **Example**: `cargo-audit` finds vulnerable crate
2. **Actions**:
   - Update dependency to patched version
   - OR: Pin to last safe version if no newer release
   - OR: Evaluate alternative library
3. **Verify**: Run `cargo audit` locally before pushing
   ```bash
   cd crates && cargo audit
   ```

### When Secrets Are Detected

1. **gitleaks blocks** the entire workflow
2. **Actions**:
   - Remove secret from code
   - Rotate/revoke the exposed secret immediately
   - Add to `.gitleaksignore` if truly a false positive (with justification)
   - Rewrite git history (if applicable) using `git filter-branch` or `git filter-repo`

---

## Configuration Files

### Main Workflow
- **Location**: `.github/workflows/extended-security.yml`
- **Size**: ~534 lines
- **Jobs**: 16 security + 1 gating job
- **Duration**: ~15-25 min total (parallelized)

### Related Workflows
- `security.yml` (legacy) — Can be deprecated or kept as backup
- `codeql.yml` (legacy, Rust-only) — Superseded by extended-security.yml
- `sbom.yml` (legacy, CycloneDX) — Partially superseded (syft added)

---

## Performance & Resource Consumption

| Scanner | Est. Time | CPU | Disk | Notes |
|---------|-----------|-----|------|-------|
| CodeQL (6 langs) | 8-12 min | High | 2-3GB | Matrix parallelized |
| Semgrep | 2-3 min | Medium | 500MB | Container image |
| bandit | <1 min | Low | 50MB | Python-only |
| gosec | 1-2 min | Low | 100MB | Go-only |
| cargo-audit | <1 min | Low | 50MB | Offline DB |
| npm audit | <1 min | Low | 50MB | JS-only |
| pip-audit | <1 min | Low | 50MB | Python-only |
| Gitleaks | 1-2 min | Low | 100MB | Full history scan |
| Trivy | 2-3 min | Medium | 1GB | Container DB |
| **Total (parallel)** | **15-25 min** | **Medium** | **~4-5GB** | All run concurrently |

**Optimization**:
- Conditional execution (only runs if language files exist)
- Fail-fast disabled (all jobs complete for visibility)
- Artifact caching leveraged where possible

---

## Known Limitations & Future Enhancements

### Current Limitations
1. **GitHub Actions billing**: Billed runners (macOS, Windows) excluded per account policy
2. **Python bandit**: Level `-ll` may miss medium-severity issues (acceptable trade-off)
3. **gosec `-no-fail`**: Advisory mode (doesn't block); could be made stricter
4. **OSV-Scanner**: Only scans Cargo.lock; npm/pip lockfiles require separate tools

### Future Enhancements
1. **Add property-based testing**: PEP 8 + mypy type checking integration
2. **Add fuzz testing**: libFuzzer for C/C++ code (if applicable)
3. **Add DAST (Dynamic)**: OWASP ZAP for deployed services
4. **License reporting**: SBOM-based license compliance dashboard
5. **Metrics dashboard**: Track findings over time (trend analysis)

---

## Security Best Practices (Enforcement via This Workflow)

| Practice | Scanner | Enforced |
|----------|---------|----------|
| No high-severity code flaws | CodeQL | ✅ Yes (fail-on: high) |
| No OWASP Top 10 violations | Semgrep | ✅ Yes (fail-on: high) |
| No hardcoded secrets | gitleaks + trufflehog | ✅ Yes (blocks) |
| No vulnerable dependencies | cargo-audit, pip-audit, npm audit | ✅ Yes (reported) |
| No language-specific issues | bandit, gosec, brakeman | ✅ Yes (reported) |
| No Dockerfile anti-patterns | hadolint | ✅ Yes (advised) |
| No container CVEs | trivy | ✅ Yes (advised) |
| No Terraform misconfig | tfsec | ✅ Yes (advised) |
| Supply chain transparency | syft SBOM | ✅ Yes (artifact) |
| License compliance | cargo-deny | ✅ Yes (reported) |

---

## References

- [CodeQL Documentation](https://codeql.github.com/)
- [Semgrep Registry](https://semgrep.dev/r/)
- [OWASP Top 10 2021](https://owasp.org/Top10/)
- [CWE Top 25](https://cwe.mitre.org/top25/)
- [SARIF Format](https://sarifweb.azurewebsites.net/)
- [trufflehog Verified Secrets](https://github.com/trufflesecurity/trufflehog)
- [Syft SBOM](https://github.com/anchore/syft)

---

## Testing This Workflow Locally

### Validate YAML Syntax
```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/extended-security.yml'))"
```

### Run CodeQL Locally (Optional)
```bash
# Install CodeQL CLI
gh codeql version
codeql database create /tmp/codeql-db --language=go
codeql database analyze /tmp/codeql-db
```

### Run Semgrep Locally
```bash
docker run --rm -v "$PWD:/src" returntocorp/semgrep semgrep --config=p/owasp-top-ten /src
```

### Run bandit Locally
```bash
pip install bandit
bandit -r python/ -ll
```

---

## Document History

| Date | Status | Change |
|------|--------|--------|
| 2026-03-30 | ✅ Complete | Initial extended security configuration documented |

