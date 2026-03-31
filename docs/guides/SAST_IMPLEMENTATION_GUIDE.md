# SAST Implementation Guide for Phenotype

**Status:** Ready for implementation
**Estimated Setup Time:** 2-3 weeks (Phase 1-2)
**Target Start Date:** 2026-04-01

---

## Phase 1: Foundation (Week 1)

### Step 1.1: Create Semgrep Rules Directory

```bash
mkdir -p .semgrep-rules
cd .semgrep-rules
```

**File: `.semgrep-rules/phenotype-core.yaml`**

```yaml
rules:
  - id: phenotype-unchecked-error
    pattern-either:
      - pattern: |
          $RES := $CALL
          $RES
      - pattern: |
          $CALL.unwrap()
    message: Unchecked error or unwrap() call may panic
    languages: [rust]
    severity: MEDIUM
    metadata:
      cwe: CWE-252
      owasp: A10:2021-Security-Logging-Monitoring

  - id: phenotype-hardcoded-secret
    pattern-either:
      - pattern: |
          $KEY = "$SECRET"
      - pattern: |
          password = "..."
    message: Hardcoded secret detected
    languages: [rust, go, python]
    severity: CRITICAL
    metadata:
      cwe: CWE-798
      owasp: A02:2021-Cryptographic-Failures
    fix: $KEY = os.getenv("$KEY")

  - id: phenotype-unvalidated-input
    pattern: |
      request.$INPUT
    message: User input not validated before use
    languages: [go, python]
    severity: HIGH
    metadata:
      cwe: CWE-20
      owasp: A03:2021-Injection

  - id: phenotype-sql-injection
    pattern-either:
      - pattern: |
          db.Exec($SQL + $USER_INPUT)
      - pattern: |
          execute($SQL % $USER_INPUT)
    message: SQL injection vulnerability
    languages: [go, python]
    severity: CRITICAL
    metadata:
      cwe: CWE-89
      owasp: A03:2021-Injection
```

**File: `.semgrep-rules/rust-specific.yaml`**

```yaml
rules:
  - id: rust-unsafe-fn
    pattern: unsafe {
      ...
    }
    message: Unsafe block detected - review for memory safety
    languages: [rust]
    severity: MEDIUM
    metadata:
      category: security
      confidence: low

  - id: rust-unwrap-panics
    pattern-either:
      - pattern: .unwrap()
      - pattern: .expect(...)
    message: unwrap/expect may panic - use ? operator instead
    languages: [rust]
    severity: MEDIUM
    fix: |
      Use Result handling or ? operator

  - id: rust-format-injection
    pattern: format!($USER_INPUT)
    message: User input in format! string - potential injection
    languages: [rust]
    severity: HIGH
    metadata:
      cwe: CWE-134
```

**File: `.semgrep-rules/go-specific.yaml`**

```yaml
rules:
  - id: go-hardcoded-password
    pattern: |
      password := "$PASSWORD"
    message: Hardcoded password detected
    languages: [go]
    severity: CRITICAL

  - id: go-insecure-random
    pattern-either:
      - pattern: rand.Intn(...)
      - pattern: math.Rand
    message: Insecure random - use crypto/rand
    languages: [go]
    severity: HIGH
    metadata:
      cwe: CWE-338
```

**File: `.semgrep-rules/python-specific.yaml`**

```yaml
rules:
  - id: python-pickle-load
    pattern: pickle.load(...)
    message: pickle.load() unsafe - use json or marshal
    languages: [python]
    severity: HIGH
    metadata:
      cwe: CWE-502

  - id: python-dangerous-eval
    pattern: eval(...)
    message: eval() is dangerous - use ast.literal_eval() instead
    languages: [python]
    severity: CRITICAL
    metadata:
      cwe: CWE-95
```

### Step 1.2: Create GitHub Actions Workflows

**File: `.github/workflows/sast-quick.yml`** (Fast PR checks)

```yaml
name: SAST Quick Scan (PR)

on:
  pull_request:
    branches: [main]
    paths-ignore:
      - '**.md'
      - '.git/**'
      - '.archive/**'

concurrency:
  group: sast-quick-${{ github.ref }}
  cancel-in-progress: true

jobs:
  semgrep:
    name: Semgrep Pattern Matching
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install Semgrep
        run: pip install semgrep

      - name: Run Semgrep (PR diff)
        id: semgrep
        run: |
          semgrep \
            --config=p/security-audit \
            --config=p/owasp-top-ten \
            --config=.semgrep-rules/ \
            --baseline-commit=origin/main \
            --json \
            --output=semgrep-results.json \
            . 2>&1 || true

      - name: Parse Results
        run: |
          COUNT=$(jq '.results | length' semgrep-results.json || echo "0")
          CRITICAL=$(jq '[.results[] | select(.extra.severity == "CRITICAL")] | length' semgrep-results.json || echo "0")
          echo "Found $COUNT issues ($CRITICAL critical)"
          echo "results_count=$COUNT" >> $GITHUB_ENV
          echo "critical_count=$CRITICAL" >> $GITHUB_ENV

      - name: Comment on PR
        if: env.results_count > 0
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const data = JSON.parse(fs.readFileSync('semgrep-results.json', 'utf8'));
            const critical = data.results.filter(r => r.extra.severity === 'CRITICAL');
            const high = data.results.filter(r => r.extra.severity === 'HIGH');

            let comment = `## 🔒 Semgrep Results\n\n`;
            comment += `**Total Issues:** ${data.results.length}\n`;
            comment += `**Critical:** ${critical.length}\n`;
            comment += `**High:** ${high.length}\n\n`;

            if (critical.length > 0) {
              comment += `### Critical Findings\n`;
              critical.slice(0, 3).forEach(r => {
                comment += `- **${r.check_id}** in \`${r.path}\`\n`;
                comment += `  ${r.extra.message}\n\n`;
              });
            }

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: comment
            });

      - name: Fail on Critical
        if: env.critical_count > 0
        run: |
          echo "Critical issues found"
          exit 1

      - name: Upload Results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: semgrep-results
          path: semgrep-results.json
```

**File: `.github/workflows/sast-full.yml`** (Deep scans, scheduled)

```yaml
name: SAST Full Scan (Nightly)

on:
  push:
    branches: [main]
  schedule:
    - cron: '0 2 * * *'  # 2 AM UTC daily

concurrency:
  group: sast-full-${{ github.ref }}
  cancel-in-progress: false

jobs:
  semgrep:
    name: Semgrep Full Analysis
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Semgrep
        run: pip install semgrep

      - name: Run Semgrep (Full scan)
        run: |
          semgrep \
            --config=p/security-audit \
            --config=p/owasp-top-ten \
            --config=p/cwe-top-25 \
            --config=.semgrep-rules/ \
            --sarif \
            --output=semgrep-results.sarif \
            . 2>&1 || true

      - name: Upload to Code Scanning
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: semgrep-results.sarif
          category: semgrep

  codeql:
    name: CodeQL Analysis
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        language: ['python', 'typescript', 'go', 'rust']
    steps:
      - uses: actions/checkout@v4

      - name: Initialize CodeQL
        uses: github/codeql-action/init@v3
        with:
          languages: ${{ matrix.language }}
          queries: security-extended

      - name: Autobuild
        uses: github/codeql-action/autobuild@v3

      - name: Perform CodeQL Analysis
        uses: github/codeql-action/analyze@v3
        with:
          category: "/language:${{matrix.language}}"

  trivy:
    name: Trivy Vulnerability Scan
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run Trivy
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: 'fs'
          scan-ref: '.'
          format: 'sarif'
          output: 'trivy-results.sarif'
          severity: 'CRITICAL,HIGH'

      - name: Upload to Code Scanning
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: trivy-results.sarif
          category: trivy

  language-specific:
    name: Language-Specific Scans
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.11'

      - name: Run Bandit
        run: |
          pip install bandit pip-audit
          bandit -r . --format sarif --output bandit-results.sarif 2>&1 || true
          pip-audit --desc 2>&1 || true

      - name: Upload Bandit Results
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: bandit-results.sarif
          category: bandit

      - name: Set up Go
        uses: actions/setup-go@v5
        with:
          go-version: 1.23

      - name: Run gosec
        run: |
          go install github.com/securego/gosec/v2/cmd/gosec@latest
          gosec -fmt sarif -out gosec-results.sarif ./... 2>&1 || true

      - name: Upload gosec Results
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: gosec-results.sarif
          category: gosec

      - name: Set up Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Run Clippy & Cargo-audit
        run: |
          cargo clippy --all-targets --all-features -- -D warnings 2>&1 || true
          cargo audit --json 2>&1 || true

  summary:
    name: Security Summary
    runs-on: ubuntu-latest
    needs: [semgrep, codeql, trivy, language-specific]
    if: always()
    steps:
      - name: Generate Report
        run: |
          echo "## Security Scan Summary" >> $GITHUB_STEP_SUMMARY
          echo "Scan completed at $(date)" >> $GITHUB_STEP_SUMMARY
          echo "- Semgrep: Pattern matching" >> $GITHUB_STEP_SUMMARY
          echo "- CodeQL: Semantic analysis" >> $GITHUB_STEP_SUMMARY
          echo "- Trivy: Vulnerability database" >> $GITHUB_STEP_SUMMARY
          echo "- Language-specific: Bandit, gosec, Clippy" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "View results in Security tab" >> $GITHUB_STEP_SUMMARY
```

### Step 1.3: Test Locally

```bash
# Install Semgrep
pip install semgrep

# Test on single file
semgrep --config=p/security-audit crates/phenotype-error-core/src/lib.rs

# Test custom rules
semgrep --config=.semgrep-rules/ --json . > local-results.json

# View results
jq '.results | length' local-results.json
```

### Step 1.4: Enable CodeQL on GitHub

1. Go to repository settings
2. Navigate to "Security" > "Code Scanning"
3. Click "Set up code scanning" > "CodeQL"
4. Add workflow (GitHub will create it automatically)

---

## Phase 2: Language-Specific Hardening (Week 2-3)

### Step 2.1: Rust Hardening

**File: `.github/workflows/rust-security.yml`**

```yaml
name: Rust Security Checks

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
    paths:
      - 'crates/**'
      - 'Cargo.toml'
      - '.github/workflows/rust-security.yml'

jobs:
  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - uses: Swatinem/rust-cache@v2

      - name: Run Clippy
        run: |
          cargo clippy \
            --all-targets \
            --all-features \
            -- -D warnings -D clippy::all

  cargo-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run cargo-audit
        uses: rustsec/audit-check-action@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

### Step 2.2: Go Hardening

**File: `.github/workflows/go-security.yml`**

```yaml
name: Go Security Checks

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
    paths:
      - '**.go'
      - 'go.mod'
      - 'go.sum'

jobs:
  gosec:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-go@v5
        with:
          go-version: 1.23

      - name: Run gosec
        run: |
          go install github.com/securego/gosec/v2/cmd/gosec@latest
          gosec -fmt=json -out=gosec-results.json ./... || true

      - name: Upload to Code Scanning
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: gosec-results.sarif
          category: gosec

  govulncheck:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-go@v5
        with:
          go-version: 1.23

      - name: Run govulncheck
        run: |
          go install golang.org/x/vuln/cmd/govulncheck@latest
          govulncheck ./...
```

### Step 2.3: Python Hardening

**File: `.github/workflows/python-security.yml`**

```yaml
name: Python Security Checks

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
    paths:
      - '**.py'
      - 'requirements*.txt'
      - 'pyproject.toml'

jobs:
  bandit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-python@v5
        with:
          python-version: '3.11'

      - name: Run Bandit
        run: |
          pip install bandit
          bandit -r . --format sarif --output bandit-results.sarif --severity-level medium

      - name: Upload to Code Scanning
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: bandit-results.sarif
          category: bandit

  pip-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-python@v5
        with:
          python-version: '3.11'

      - name: Run pip-audit
        run: |
          pip install pip-audit
          pip-audit --desc || true
```

---

## Phase 3: Advanced Features (Week 3+)

### Step 3.1: Security Dashboard

**File: `.github/workflows/security-dashboard.yml`**

```yaml
name: Security Dashboard Update

on:
  schedule:
    - cron: '0 9 * * MON'  # Weekly Monday 9 AM

jobs:
  dashboard:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Fetch Code Scanning Alerts
        run: |
          echo "## Security Dashboard - $(date)" > SECURITY_DASHBOARD.md
          echo "" >> SECURITY_DASHBOARD.md

          # Critical findings
          CRITICAL=$(gh api -H "Accept: application/vnd.github+json" \
            "/repos/${{ github.repository }}/code-scanning/alerts?state=open&severity=critical" \
            --jq 'length')

          echo "- Critical: $CRITICAL" >> SECURITY_DASHBOARD.md

          # High findings
          HIGH=$(gh api -H "Accept: application/vnd.github+json" \
            "/repos/${{ github.repository }}/code-scanning/alerts?state=open&severity=high" \
            --jq 'length')

          echo "- High: $HIGH" >> SECURITY_DASHBOARD.md

        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Create Issue with Dashboard
        uses: actions/github-script@v7
        with:
          script: |
            github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: `Weekly Security Dashboard`,
              labels: ['security', 'weekly-report'],
              body: `See Security tab for code scanning details`
            });
```

### Step 3.2: SonarQube CE (Optional)

```yaml
name: SonarQube Scan

on:
  push:
    branches: [main]

jobs:
  sonarqube:
    runs-on: ubuntu-latest
    services:
      sonarqube:
        image: sonarqube:latest
        options: -e SONAR_ES_BOOTSTRAP_CHECKS_DISABLED=true
        ports:
          - 9000:9000

    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: SonarQube Scan
        uses: SonarSource/sonarqube-scan-action@master
        env:
          SONAR_HOST_URL: ${{ secrets.SONAR_HOST_URL }}
          SONAR_TOKEN: ${{ secrets.SONAR_TOKEN }}
```

---

## Configuration Checklist

- [ ] `.semgrep-rules/` directory created with custom rules
- [ ] `.github/workflows/sast-quick.yml` created (PR checks)
- [ ] `.github/workflows/sast-full.yml` created (nightly scans)
- [ ] CodeQL enabled in GitHub settings
- [ ] Branch protection rules updated to require SAST checks
- [ ] Team notified of new security requirements
- [ ] Documentation created in `docs/guides/`
- [ ] Security contact added to `SECURITY.md`

---

## Rollout Plan by Repository

### Week 1 (Foundation)
- [ ] phenotype-infrakit (Rust monorepo) — establish baseline
- [ ] AgilePlus (mixed Rust/HTML) — establish baseline

### Week 2 (Expansion)
- [ ] platforms/thegent (large Go monorepo) — parallel scans needed
- [ ] All remaining Python projects

### Week 3 (Hardening)
- [ ] Language-specific workflows active
- [ ] Quality gates enforced
- [ ] Dashboard operational

---

## Troubleshooting

### Semgrep is too slow

**Solution:** Use diff-aware scanning on PRs
```bash
semgrep --baseline-commit=origin/main .
```

### CodeQL times out on large Go monorepo

**Solution:** Split by packages
```yaml
strategy:
  matrix:
    package:
      - ./cmd/...
      - ./pkg/...
      - ./internal/...
```

### Too many false positives

**Solution:** Tune rules and disable low-confidence ones
```bash
semgrep --config=p/security-audit --exclude-rule=rule-id-xxx .
```

### GitHub Actions billing concerns

**Solution:** Use public runners for public repos (free tier)
```yaml
runs-on: ubuntu-latest  # Free for public repos
```

---

## Support & Next Steps

- **Questions?** See `SAST_TOOL_EVALUATION.md` for detailed comparison
- **Custom rules?** See Semgrep playground: https://semgrep.dev/explore
- **CodeQL?** See GitHub docs: https://codeql.github.com/docs/
- **Team training?** Schedule security workshop

---

**Ready to implement?** Start with Step 1.1 and test locally before pushing to GitHub.
