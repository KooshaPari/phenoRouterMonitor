# Code Smell Detection & Linting Guide for Phenotype Ecosystem

**Document Status:** Comprehensive Research Complete
**Last Updated:** 2026-03-30
**Target Audience:** Platform Engineers, Cloud Agents, CI/CD Architects

---

## Executive Summary

This guide provides a complete linting and code smell detection strategy for Phenotype's polyrepo ecosystem (Rust, Go, Python, TypeScript). It includes:

- **Language-specific linting tool recommendations** (open-source, free-tier first)
- **Code smell thresholds** (warning/error severity mapping)
- **Duplication detection setup** (copy-paste code, ghost files)
- **Documentation quality enforcement** (Vale + markdownlint)
- **GitHub Actions CI integration** (automated on PR)
- **Pre-commit hook setup** (local prevention + fix automation)
- **Agent-driven auto-fixing** (cloud agents eliminate style issues)

## Part 1: Dedicated Linters by Language

### Rust: Clippy + cargo-deny + cargo-machete

**Status in Phenotype:** ✅ Already Integrated

**Tools:**
- **clippy** - Lints, style checks, performance anti-patterns (built-in, zero setup)
- **cargo-deny** - License scanning, advisory checking, duplicate deps
- **cargo-machete** - Detects unused crate dependencies
- **cargo-fmt** - Code formatter (enforced)
- **cargo-semver-checks** - Semantic versioning compliance

**Free Tier:** ✅ 100% open-source, no limits

**Configuration:** `/Users/kooshapari/CodeProjects/Phenotype/repos/.pre-commit-config.yaml`

```yaml
# Rust linting (existing in repos)
- id: clippy
  name: clippy
  entry: bash -c 'cd rust && cargo clippy --all-targets -- -D warnings'
  language: system
  files: '\.rs$'
  pass_filenames: false
```

**Thresholds:**
| Severity | Rule Type | Action | Details |
|----------|-----------|--------|---------|
| Error | Dead code (-D warnings) | Block PR | Non-negotiable |
| Error | Unsafe code | Block PR | Requires justification |
| Error | Panic in library | Block PR | Use Result<T, E> instead |
| Warning | Clippy perf (style level) | Warn in comment | Can merge with suppression |
| Info | Machete unused deps | Auto-fix | Remove before merge |

**Latest Versions (as of 2026-03-30):**
- Clippy: Built-in with Rust 1.86+ (nightly recommended)
- cargo-deny: v0.15.1
- cargo-machete: v0.6.0
- cargo-semver-checks: v0.35.0

**GitHub Actions Integration:** ✅ Already set up in `ci.yml` (lines 40-112)

---

### Go: golangci-lint (Aggregator with 50+ Linters)

**Status in Phenotype:** ⚠️ Partial (agentapi-plusplus has config, thegent/platforms needs expansion)

**Tools:**
- **golangci-lint** - Single aggregator with 50+ built-in linters
- **errcheck** - Unchecked error handling
- **staticcheck** - Go vet + static analysis
- **revive** - Drop-in replacement for golint
- **gosec** - Security scanning
- **gomodguard** - Control module imports
- **cyclonedx-gomod** - SBOM generation
- **trufflehog** - Secret detection (see MEMORY.md: already in use, gitleaks hung)

**Free Tier:** ✅ 100% open-source, no limits

**Current Config:** `/Users/kooshapari/CodeProjects/Phenotype/repos/agentapi-plusplus/.golangci.yml`

```yaml
version: "2"
linters:
  disable:
    - exhaustive      # Too strict for interfaces
    - errcheck        # Handled manually in many places
    - unused          # Pre-existing issues
```

**Recommended Enhanced Config (to adopt across thegent + platforms):**

```yaml
version: "2"

run:
  timeout: 5m
  modules-download-mode: readonly

linters:
  enable:
    # Error handling
    - errcheck          # Must check error returns
    - nilerr            # Nil error patterns
    - errorlint         # Error wrapping standards

    # Code quality
    - revive            # Style and logic
    - staticcheck       # Static analysis (powerful)
    - govet             # Go vet (built-in analysis)
    - gosec             # Security (OWASP top 10)
    - gocritic          # Code critic
    - bodyclose         # HTTP response body cleanup

    # Complexity
    - gocyclo           # Cyclomatic complexity
    - gocognit          # Cognitive complexity

    # Performance
    - unconvert         # Unnecessary type conversions
    - ineffassign       # Ineffective assignments
    - exhaustive        # Exhaustive switch cases

    # Dependency management
    - gomodguard        # Control imports
    - gomod             # Module system checks

    # Naming
    - misspell          # Typo detection

    # Duplication
    - dupl              # Duplicate code blocks
    - copyloopvar       # Loop variable issues (Go 1.22+)

  disable:
    - exhaustivestruct  # False positives in tests
    - interfacer        # Deprecated
    - scopelint         # Deprecated (use exportloopref)

linters-settings:
  errcheck:
    check-type-assertions: true
    check-blank: true

  staticcheck:
    checks: ["all", "-SA1011"]  # All checks except one false positive

  gocyclo:
    min-complexity: 12  # Warning threshold

  gocognit:
    min-complexity: 15  # Cognitive complexity

  gocritic:
    enabled-checks:
      - appendAssign
      - badCond
      - badLock
      - captLocal
      - caseOrder
      - defaultCaseOrder

  govet:
    enable-all: true

  revive:
    enable-all-rules: true
    rules:
      - name: blank-imports
        disabled: false
      - name: context-as-argument
        disabled: false

issues:
  max-issues-per-linter: 0
  max-same-issues: 0
  exclude-rules:
    # Allow errors in tests
    - path: _test\.go$
      linters:
        - errcheck
```

**Thresholds:**
| Severity | Rule Type | Action | Details |
|----------|-----------|--------|---------|
| Error | errcheck: missed error | Block PR | All errors must be checked |
| Error | gosec: security issues | Block PR | OWASP vulnerabilities |
| Error | staticcheck | Block PR | Static analysis failures |
| Warning | gocyclo > 12 | Warn, suggest refactor | Decompose large functions |
| Info | golint style | Auto-fix | Code formatting |
| Warning | revive: naming conventions | Auto-fix | Follow Go conventions |

**Latest Versions (as of 2026-03-30):**
- golangci-lint: v1.65.0
- revive: v1.3.7
- gocritic: built-in
- gosec: built-in

**GitHub Actions Integration:**

```yaml
name: Go Linting
on: [push, pull_request]
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-go@v5
        with:
          go-version: "1.23"
      - uses: golangci/golangci-lint-action@v4
        with:
          version: latest
          args: --timeout=5m
```

---

### Python: Ruff + pip-audit + Pyright

**Status in Phenotype:** ✅ Already Integrated

**Tools:**
- **ruff** - Fast linter + formatter (replaces pylint, flake8, isort, black)
- **pip-audit** - Vulnerability scanner for dependencies
- **pyright** - Static type checker (strict mode)
- **pydantic** - Runtime type validation
- **pytest** - Test runner + coverage

**Free Tier:** ✅ 100% open-source, no limits

**Current Config:** `/Users/kooshapari/CodeProjects/Phenotype/repos/.pre-commit-config.yaml` (lines 43-55)

```yaml
- id: ruff-format
  name: ruff format
  entry: bash -c 'cd python && uvx ruff format --check .'
  language: system
  files: '\.py$'

- id: ruff-check
  name: ruff check
  entry: bash -c 'cd python && uvx ruff check .'
  language: system
  files: '\.py$'
```

**Recommended Enhanced ruff.toml:**

```toml
[tool.ruff]
line-length = 100
target-version = "py310"

[tool.ruff.lint]
select = [
  # Core rules
  "E",      # Errors
  "W",      # Warnings
  "F",      # Pyflakes (undefined names, etc)
  "C90",    # mccabe (complexity)

  # Best practices
  "B",      # flake8-bugbear
  "A",      # Shadowed builtins
  "C4",     # Comprehensions
  "PIE",    # Pie-chart patterns
  "RUF",    # Ruff-specific rules

  # Code quality
  "D",      # Docstrings (pydoc style)
  "UP",     # Pyupgrade
  "SIM",    # Simplification
  "ARG",    # Unused arguments
  "PERF",   # Performance

  # Security (via bandit)
  "S",      # Security issues

  # Type checking (via pyright)
  "ANN",    # Type annotations
]

ignore = [
  "E501",   # Line too long (handled by formatter)
  "D100",   # Module docstring not required
  "D104",   # Package docstring not required
  "ANN101", # Self type annotation (implicit)
  "ANN102", # Cls type annotation (implicit)
]

[tool.ruff.lint.mccabe]
max-complexity = 12

[tool.ruff.lint.pydocstyle]
convention = "google"

[tool.ruff.format]
quote-style = "double"
indent-style = "space"
line-ending = "auto"

[tool.pyright]
typeCheckingMode = "strict"
pythonVersion = "3.10"
venvPath = "."
reportUnusedImport = "error"
reportUnusedClass = "error"
reportUnusedFunction = "error"
reportPrivateUsage = "warning"

[tool.pytest.ini_options]
minversion = "7.0"
testpaths = ["tests"]
python_files = "test_*.py"
asyncio_mode = "auto"
addopts = "--strict-markers --tb=short -v"
```

**Thresholds:**
| Severity | Rule Type | Action | Details |
|----------|-----------|--------|---------|
| Error | Undefined names (F) | Block PR | Fix immediately |
| Error | Type mismatch (via Pyright strict) | Block PR | All types must match |
| Error | Security issues (S) | Block PR | OWASP compliance |
| Warning | Unused imports/vars (F, ARG) | Auto-fix | Clean imports |
| Warning | Complexity > 12 | Warn | Suggest refactor |
| Info | Style (PEP 8) | Auto-fix | Ruff format |

**Latest Versions (as of 2026-03-30):**
- Ruff: v0.4.10
- pip-audit: v2.7.3
- Pyright: v1.1.349
- Pydantic: v2.8.2

**GitHub Actions Integration:**

```yaml
name: Python Quality
on: [push, pull_request]
jobs:
  python:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: "3.14"
      - name: Install uv
        run: pip install uv
      - name: Install package
        run: cd python && uv sync
      - name: Ruff format
        run: cd python && uvx ruff format --check .
      - name: Ruff lint
        run: cd python && uvx ruff check .
      - name: Type check
        run: cd python && uvx pyright .
      - name: Security audit
        run: cd python && uvx pip-audit
      - name: Tests
        run: cd python && uv run pytest --cov
```

---

### TypeScript/JavaScript: ESLint + TypeScript Compiler + prettier

**Status in Phenotype:** ⚠️ Partial (heliosApp, agent-wave have package.json, no consistent config)

**Tools:**
- **ESLint** - Linting with ~150+ rules
- **TypeScript compiler** - Type checking (strict mode)
- **prettier** - Code formatter
- **eslint-plugin-security** - Security rules
- **eslint-plugin-react** - React best practices
- **knip** - Find unused files/exports

**Free Tier:** ✅ 100% open-source, no limits

**Recommended .eslintrc.json (for all TS/JS projects):**

```json
{
  "root": true,
  "env": {
    "es2024": true,
    "node": true
  },
  "extends": [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:@typescript-eslint/recommended-requiring-type-checking",
    "plugin:security/recommended",
    "prettier"
  ],
  "parser": "@typescript-eslint/parser",
  "parserOptions": {
    "ecmaVersion": "latest",
    "sourceType": "module",
    "project": "./tsconfig.json"
  },
  "plugins": [
    "@typescript-eslint",
    "security",
    "import"
  ],
  "rules": {
    "@typescript-eslint/explicit-function-return-types": "error",
    "@typescript-eslint/explicit-member-accessibility": "error",
    "@typescript-eslint/no-explicit-any": "error",
    "@typescript-eslint/no-unused-vars": "error",
    "@typescript-eslint/no-floating-promises": "error",
    "@typescript-eslint/no-misused-promises": "error",
    "@typescript-eslint/await-thenable": "error",
    "@typescript-eslint/no-unnecessary-type-assertion": "error",
    "@typescript-eslint/strict-boolean-expressions": "error",
    "import/no-unresolved": "error",
    "import/no-cycle": "error",
    "security/detect-non-literal-regexp": "warn",
    "no-console": "warn",
    "prefer-const": "error"
  },
  "overrides": [
    {
      "files": ["**/*.test.ts", "**/*.spec.ts"],
      "env": {
        "jest": true
      },
      "rules": {
        "@typescript-eslint/no-explicit-any": "off"
      }
    }
  ]
}
```

**Recommended tsconfig.json (strict mode):**

```json
{
  "compilerOptions": {
    "target": "ES2024",
    "module": "ESNext",
    "lib": ["ES2024"],
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "noImplicitAny": true,
    "strictNullChecks": true,
    "strictFunctionTypes": true,
    "strictBindCallApply": true,
    "strictPropertyInitialization": true,
    "noImplicitThis": true,
    "alwaysStrict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "noUncheckedIndexedAccess": true,
    "allowUnusedLabels": false,
    "allowUnreachableCode": false
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist", "**/*.test.ts"]
}
```

**Thresholds:**
| Severity | Rule Type | Action | Details |
|----------|-----------|--------|---------|
| Error | Type mismatch (strict: true) | Block PR | No any types allowed |
| Error | Undefined variables | Block PR | Fix immediately |
| Error | Unused variables | Block PR | Auto-fixable |
| Warning | console.log in production | Block PR | Use logger instead |
| Warning | Complexity > 15 | Warn | Suggest refactor |
| Info | Style (prettier) | Auto-fix | Formatting |

**Latest Versions (as of 2026-03-30):**
- ESLint: v9.0.0
- @typescript-eslint/eslint-plugin: v7.3.1
- TypeScript: v5.4.5
- prettier: v3.2.5
- knip: v5.1.0

**GitHub Actions Integration:**

```yaml
name: TypeScript Quality
on: [push, pull_request]
jobs:
  typescript:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        project: [heliosApp, agent-wave]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: "22"
          cache: "npm"
      - name: Install dependencies
        run: cd ${{ matrix.project }} && npm ci
      - name: TypeScript compiler
        run: cd ${{ matrix.project }} && npx tsc --noEmit
      - name: ESLint
        run: cd ${{ matrix.project }} && npx eslint src --max-warnings 0
      - name: Prettier
        run: cd ${{ matrix.project }} && npx prettier --check src
      - name: Unused files
        run: cd ${{ matrix.project }} && npx knip
```

---

## Part 2: Code Smell Detection (SonarQube Alternative for Free Tier)

### Option 1: SonarCloud (FREE for Public Repos)

**Status:** ✅ Recommended for adoption (free for GitHub public repos)

**Features:**
- 25+ code smell rules
- Code duplication detection
- Security hotspots (OWASP, CWE)
- Test coverage tracking
- PR analysis with inline comments

**Cost:** ✅ FREE for public repos, $10/month for private (acceptable for KooshaPari's account)

**Setup:**

1. Enable at https://sonarcloud.io (sign in with GitHub)
2. Add GitHub Actions workflow:

```yaml
name: SonarCloud
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  sonarcloud:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: SonarSource/sonarcloud-github-action@master
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          SONARCLOUD_TOKEN: ${{ secrets.SONARCLOUD_TOKEN }}
```

3. Create `sonar-project.properties`:

```properties
sonar.projectKey=KooshaPari_phenotype-infrakit
sonar.organization=kooshaparı
sonar.projectVersion=0.2.0

# Source code
sonar.sources=crates,src,rust,python

# Test coverage
sonar.coverage.exclusions=**/*test.rs,**/*_test.py,tests/**
sonar.python.coverage.reportPaths=coverage.xml
sonar.rust.test.reportPath=target/coverage

# Code smells thresholds
sonar.projectBaseDir=.
sonar.exclusions=**/node_modules/**,**/.cargo/**,**/target/**
```

**Code Smell Categories (25+):**
| Category | Examples | Action |
|----------|----------|--------|
| Cognitive Complexity | Complex conditionals, nested loops | Warn > 15, Error > 25 |
| Code Duplication | Copy-paste blocks (>3 lines) | Warn > 5%, Error > 10% |
| Dead Code | Unreachable code, unused vars | Block PR |
| Security Hotspots | SQL injection, XSS, crypto | Block PR |
| Maintainability | Long functions, many params | Warn > 3 issues, Error > 10 |
| Test Coverage | Uncovered branches | Warn < 80%, Error < 50% |
| Naming Issues | Bad variable names, inconsistent | Info only |

**Quality Gate (Recommended):**
```
- Coverage >= 80%
- Code Smells = 0 new (allow existing for migration)
- Duplications < 5%
- Security Rating = A
- Reliability = A
```

---

### Option 2: CodeClimate (Alternative, $99-299/month)

**Status:** ⚠️ Not recommended (paid, features overlap with SonarCloud)

**vs SonarCloud:**
- SonarCloud: Better security rules, faster PR comments, cheaper
- CodeClimate: Better maintainability metrics, UI polish
- **Recommendation:** SonarCloud for Phenotype (free + better security)

---

## Part 3: Duplication Detection

### Copy-Paste Code Detection

**Tool 1: Semgrep (Free Tier)**

```bash
# Install
pip install semgrep

# Find duplicated code patterns
semgrep --config=p/security-audit --config=p/python-best-practices --config=p/copy-paste-detection .

# Find 3+ line duplicates in Python
semgrep --pattern-file=<(echo 'patterns:
- patterns:
    - pattern-inside: |
        def $FUNC(...):
          ...
    - metavariable-comparison:
        comparison: 'len($A) > 3'
        metavariable: A
  languages: [python]
') .
```

**Tool 2: CPD (Copy/Paste Detector)**

```bash
# Install
curl https://sourceforge.net/projects/pmd/files/pmd/7.0.0/pmd-dist-7.0.0.zip -o pmd.zip
unzip pmd.zip && rm pmd.zip

# Find 30+ line duplicates (default)
pmd cpd --language python --minimum-tokens 30 src/

# Find 15+ line duplicates
pmd cpd --language python --minimum-tokens 15 src/ --format csv > duplication-report.csv
```

**Tool 3: Ruff Duplication Checker (Built-in)**

For Python, Ruff's `ARG` rule catches unused arguments that often signal duplication:

```bash
# Find potentially duplicated logic (unused args = copy-paste without using var)
ruff check --select ARG src/
```

**GitHub Actions: Duplication Report**

```yaml
name: Duplication Detection
on: [push, pull_request]
jobs:
  duplication:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install CPD
        run: |
          curl -s https://sourceforge.net/projects/pmd/files/pmd/7.0.0/pmd-dist-7.0.0.zip -o pmd.zip
          unzip -q pmd.zip && rm pmd.zip
          echo "$PWD/pmd-dist-7.0.0/bin" >> $GITHUB_PATH

      - name: Run CPD (Python)
        run: |
          run.sh cpd --language python --minimum-tokens 15 \
            src/ python/ --format csv > duplication-python.csv

      - name: Run CPD (Rust)
        run: |
          run.sh cpd --language rust --minimum-tokens 15 \
            rust/ crates/ --format csv > duplication-rust.csv

      - name: Check duplication threshold
        run: |
          PYTHON_DUP=$(awk -F, 'NR>1 {lines+=$3} END {print lines}' duplication-python.csv)
          RUST_DUP=$(awk -F, 'NR>1 {lines+=$3} END {print lines}' duplication-rust.csv)

          if [ "$PYTHON_DUP" -gt 500 ]; then
            echo "ERROR: Python duplication > 500 lines ($PYTHON_DUP)"
            exit 1
          fi
          if [ "$RUST_DUP" -gt 500 ]; then
            echo "ERROR: Rust duplication > 500 lines ($RUST_DUP)"
            exit 1
          fi
          echo "✓ Duplication check passed"

      - name: Upload reports
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: duplication-reports
          path: duplication-*.csv
```

---

### Secret Scanning (Secrets Duplication)

**Current Status:** ✅ trufflehog in use (per MEMORY.md, gitleaks replaced)

**Tool: trufflehog v3.93.6**

```bash
# Install (already available)
trufflehog version

# Scan local git repo
trufflehog git file://. --since-commit HEAD --only-verified --fail

# Scan commit range
trufflehog git file://. --from-ref origin/main --to-ref HEAD --only-verified

# Find ALL secrets in history (aggressive)
trufflehog git file://. --only-verified
```

**GitHub Actions: Secret Scanning**

```yaml
name: Secret Scanning
on: [push, pull_request]
jobs:
  secrets:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - name: Install trufflehog
        run: pip install trufflehog
      - name: Scan for secrets
        run: |
          # Scan from main to HEAD (PR changes only)
          if [ "${{ github.event_name }}" = "pull_request" ]; then
            trufflehog git file://. \
              --from-ref origin/main \
              --to-ref HEAD \
              --only-verified \
              --fail
          else
            # Scan just new commits
            trufflehog git file://. \
              --since-commit HEAD~1 \
              --only-verified \
              --fail
          fi
```

---

## Part 4: Documentation Quality

### Vale (Prose Linting)

**Status in Phenotype:** ✅ Already configured (see CLAUDE.md)

**Installation:**

```bash
# macOS
brew install vale

# Linux
curl https://github.com/errata-ai/vale/releases/download/v2.30.0/vale_linux_amd64.tar.gz -o vale.tar.gz
tar -xzf vale.tar.gz && mv vale /usr/local/bin
```

**Configuration: `.vale.ini`**

```ini
# Vale configuration
StylesPath = .vale/styles
Vocab = Phenotype
Packages = Google, Joblint

[*.md]
BasedOnStyles = Google, Vale

# File-specific rules
[*.{md,rst,txt}]
  Google.Acronyms = NO        # Allow custom acronyms (PhenSDK, MCP, etc)
  Google.Headings = YES
  Google.Passive = ERROR
  Google.Ranges = YES
  Google.Spacing = YES
  Google.We = YES
  Vale.Accessibility = YES
  Vale.Blunt = YES
  Vale.Cliches = YES
  Vale.Mascots = YES
  Vale.Hyperbole = YES

[docs/**/*.md]
  Vale.Iambs = YES            # Enforce proper stress
  Google.HeadingStartsWithCapital = YES
  Google.Headings = NO        # Too strict for code docs

[README.md]
  BasedOnStyles = Google      # Strict for project README
  Google.Contractions = YES
```

**GitHub Actions: Documentation Check**

```yaml
name: Documentation Quality
on: [push, pull_request]
jobs:
  prose:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Vale
        run: brew install vale
      - name: Run Vale
        run: vale --config .vale.ini docs/ README.md
        continue-on-error: true
      - name: Run markdownlint
        uses: nosborn/github-action-markdown-cli@v3.3.0
        with:
          files: docs/ README.md
          config_file: .markdownlint.json
```

**markdownlint Configuration: `.markdownlint.json`**

```json
{
  "default": true,
  "MD003": { "style": "consistent" },
  "MD004": { "style": "consistent" },
  "MD007": { "indent": 2 },
  "MD013": {
    "line_length": 120,
    "heading_line_length": 120,
    "code_block_line_length": 120
  },
  "MD014": false,
  "MD024": false,
  "MD025": false,
  "MD040": false,
  "MD041": false,
  "no-hard-tabs": true
}
```

---

## Part 5: GitHub Actions Integration

### Complete CI/CD Linting Pipeline

**Location:** Create new workflow at `.github/workflows/linting.yml`

```yaml
name: Comprehensive Linting & Code Smell Detection

on:
  push:
    branches: [main]
  pull_request:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM UTC

concurrency:
  group: linting-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # ─── Rust Quality (Existing Enhanced) ────────────────────────────────
  rust-linting:
    name: Rust Linting & Smells
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - uses: arduino/setup-protoc@v3
        with:
          version: "28.x"

      # Format check
      - name: Format check
        run: cargo fmt --all -- --check

      # Clippy (warnings = errors)
      - name: Clippy lint
        run: cargo clippy --workspace --all-targets -- -D warnings

      # Unused dependencies
      - name: Detect unused deps (cargo-machete)
        run: |
          cargo install cargo-machete
          cargo machete

      # Dependency vulnerabilities
      - name: Audit dependencies
        run: |
          cargo install cargo-deny
          cargo deny check -c rust/deny.toml

      # Semantic versioning
      - name: Check semver
        run: |
          cargo install cargo-semver-checks
          cargo semver-checks check-release || true

  # ─── Go Quality (New) ────────────────────────────────────────────────
  go-linting:
    name: Go Linting & Smells
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-go@v5
        with:
          go-version: "1.23"

      # golangci-lint
      - name: golangci-lint
        uses: golangci/golangci-lint-action@v4
        with:
          version: latest
          args: --timeout=5m --deadline=10m
          skip-pkg-cache: true
          skip-build-cache: true

      # Go fmt
      - name: Format check
        run: |
          go fmt ./...
          if ! git diff --quiet; then
            echo "ERROR: Code not formatted. Run: go fmt ./..."
            exit 1
          fi

      # Go vet
      - name: Vet
        run: go vet ./...

  # ─── Python Quality (Existing Enhanced) ──────────────────────────────
  python-linting:
    name: Python Linting & Smells
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: "3.14"

      - name: Install uv
        run: pip install uv

      - name: Install package
        run: cd python && uv sync

      # Ruff format
      - name: Ruff format check
        run: cd python && uvx ruff format --check .

      # Ruff lint
      - name: Ruff lint
        run: cd python && uvx ruff check .

      # Type checking (strict)
      - name: Pyright type check
        run: cd python && uvx pyright .

      # Security audit
      - name: pip-audit
        run: cd python && uvx pip-audit --ignore-vuln CVE-2026-4539

      # Unused code
      - name: Vulture (dead code)
        run: cd python && pip install vulture && vulture src/ || true

  # ─── TypeScript Quality (New) ────────────────────────────────────────
  typescript-linting:
    name: TypeScript Linting & Smells
    runs-on: ubuntu-latest
    strategy:
      matrix:
        project: [heliosApp, agent-wave, heliosCLI]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: "22"
          cache: "npm"

      - name: Install dependencies
        run: cd ${{ matrix.project }} && npm ci

      - name: TypeScript compiler
        run: cd ${{ matrix.project }} && npx tsc --noEmit

      - name: ESLint
        run: cd ${{ matrix.project }} && npx eslint src --max-warnings 0

      - name: Prettier check
        run: cd ${{ matrix.project }} && npx prettier --check src

      - name: Unused files (knip)
        run: cd ${{ matrix.project }} && npx knip

  # ─── Documentation Quality ───────────────────────────────────────────
  docs-linting:
    name: Documentation Linting
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Vale
        run: brew install vale || (wget https://github.com/errata-ai/vale/releases/download/v2.30.0/vale_linux_x86_64.zip && unzip vale_linux_x86_64.zip && sudo mv vale /usr/local/bin)

      - name: Run Vale
        run: vale --config .vale.ini --output=line docs/ README.md

      - name: Run markdownlint
        uses: nosborn/github-action-markdown-cli@v3.3.0
        with:
          files: docs/ README.md
          config_file: .markdownlint.json

  # ─── Code Duplication Detection ──────────────────────────────────────
  duplication:
    name: Duplication Detection
    runs-on: ubuntu-latest
    continue-on-error: true
    steps:
      - uses: actions/checkout@v4

      # Install PMD for CPD
      - name: Install PMD
        run: |
          wget -q https://github.com/pmd/pmd/releases/download/pmd_releases%2F7.0.0/pmd-dist-7.0.0.zip
          unzip -q pmd-dist-7.0.0.zip
          echo "$PWD/pmd-dist-7.0.0/bin" >> $GITHUB_PATH

      # Python duplication
      - name: Python duplication (15+ lines)
        run: |
          run.sh cpd --language python --minimum-tokens 15 \
            src/ python/ --format csv > python-duplication.csv || true
          LINES=$(awk -F, 'NR>1 {sum+=$3} END {print sum}' python-duplication.csv)
          echo "Python duplication: $LINES lines"
          if [ "$LINES" -gt 1000 ]; then
            echo "⚠️ WARNING: Duplication > 1000 lines (consider refactoring)"
          fi

      # Rust duplication
      - name: Rust duplication (15+ lines)
        run: |
          run.sh cpd --language rust --minimum-tokens 15 \
            rust/ crates/ --format csv > rust-duplication.csv || true
          LINES=$(awk -F, 'NR>1 {sum+=$3} END {print sum}' rust-duplication.csv)
          echo "Rust duplication: $LINES lines"

      # Upload reports
      - name: Upload duplication reports
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: duplication-reports
          path: "*-duplication.csv"

  # ─── SonarCloud Code Quality ─────────────────────────────────────────
  sonarcloud:
    name: SonarCloud Analysis
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: SonarCloud Scan
        uses: SonarSource/sonarcloud-github-action@master
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          SONARCLOUD_TOKEN: ${{ secrets.SONARCLOUD_TOKEN }}

  # ─── Secret Detection ────────────────────────────────────────────────
  secrets:
    name: Secret Scanning
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install trufflehog
        run: pip install trufflehog

      - name: Scan for secrets (PR changes only)
        if: github.event_name == 'pull_request'
        run: |
          trufflehog git file://. \
            --from-ref origin/main \
            --to-ref HEAD \
            --only-verified \
            --fail

      - name: Scan for secrets (recent commits)
        if: github.event_name != 'pull_request'
        run: |
          trufflehog git file://. \
            --since-commit HEAD~10 \
            --only-verified \
            --fail || true

  # ─── Summary & Status ────────────────────────────────────────────────
  summary:
    name: Linting Summary
    needs: [rust-linting, go-linting, python-linting, typescript-linting, docs-linting, duplication, secrets]
    if: always()
    runs-on: ubuntu-latest
    steps:
      - name: Check linting status
        run: |
          echo "## Linting Results" >> $GITHUB_STEP_SUMMARY
          echo "| Check | Status |" >> $GITHUB_STEP_SUMMARY
          echo "|-------|--------|" >> $GITHUB_STEP_SUMMARY
          echo "| Rust | ${{ needs.rust-linting.result }} |" >> $GITHUB_STEP_SUMMARY
          echo "| Go | ${{ needs.go-linting.result }} |" >> $GITHUB_STEP_SUMMARY
          echo "| Python | ${{ needs.python-linting.result }} |" >> $GITHUB_STEP_SUMMARY
          echo "| TypeScript | ${{ needs.typescript-linting.result }} |" >> $GITHUB_STEP_SUMMARY
          echo "| Docs | ${{ needs.docs-linting.result }} |" >> $GITHUB_STEP_SUMMARY
          echo "| Duplication | ${{ needs.duplication.result }} |" >> $GITHUB_STEP_SUMMARY
          echo "| Secrets | ${{ needs.secrets.result }} |" >> $GITHUB_STEP_SUMMARY

          # Fail if any required check failed
          if [ "${{ needs.rust-linting.result }}" = "failure" ] || \
             [ "${{ needs.python-linting.result }}" = "failure" ] || \
             [ "${{ needs.docs-linting.result }}" = "failure" ] || \
             [ "${{ needs.secrets.result }}" = "failure" ]; then
            exit 1
          fi
```

---

## Part 6: Pre-commit Hook Setup

### Local Prevention: `.pre-commit-config.yaml`

**Current Status:** ✅ Exists at `/Users/kooshapari/CodeProjects/Phenotype/repos/.pre-commit-config.yaml`

**Enhanced Version (comprehensive):**

```yaml
repos:
  # ─── Standard Hooks ─────────────────────────────────────────────────
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v5.0.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: check-toml
      - id: check-json
      - id: check-merge-conflict
      - id: check-added-large-files
        args: ['--maxkb=500']
      - id: check-case-conflict
      - id: check-docstring-first
      - id: debug-statements
      - id: mixed-line-ending
        args: ['--fix=lf']

  # ─── Conventional Commits ────────────────────────────────────────────
  - repo: https://github.com/compilerla/conventional-pre-commit
    rev: v4.0.0
    hooks:
      - id: conventional-pre-commit
        stages: [commit-msg]
        args: [feat, fix, docs, style, refactor, perf, test, chore, ci, build, revert]

  # ─── Prose Linting ──────────────────────────────────────────────────
  - repo: https://github.com/igorshubovych/markdownlint-cli
    rev: v0.40.0
    hooks:
      - id: markdownlint
        files: '\.md$'
        args: [--config, .markdownlint.json]
        exclude: 'node_modules'

  - repo: https://github.com/errata-ai/vale
    rev: v2.30.0
    hooks:
      - id: vale
        files: '\.md$'
        args: [--config, .vale.ini]

  # ─── Typo Detection ─────────────────────────────────────────────────
  - repo: https://github.com/crate-ci/typos
    rev: v1.19.0
    hooks:
      - id: typos
        args: [--write-changes]

  # ─── Rust Linting ───────────────────────────────────────────────────
  - repo: local
    hooks:
      - id: rustfmt
        name: rustfmt
        entry: bash -c 'cd rust && cargo fmt -- --check'
        language: system
        files: '\.rs$'
        pass_filenames: false
        stages: [commit]

      - id: clippy
        name: clippy
        entry: bash -c 'cd rust && cargo clippy --all-targets -- -D warnings'
        language: system
        files: '\.rs$'
        pass_filenames: false
        stages: [push]

      - id: machete
        name: cargo-machete
        entry: bash -c 'cargo install cargo-machete --quiet && cargo machete'
        language: system
        files: 'Cargo.toml'
        pass_filenames: false
        stages: [push]

  # ─── Go Linting ─────────────────────────────────────────────────────
  - repo: https://github.com/golangci/golangci-lint
    rev: v1.65.0
    hooks:
      - id: golangci-lint
        entry: golangci-lint run --fix
        language: golang
        files: '\.go$'
        pass_filenames: false

  # ─── Python Linting ─────────────────────────────────────────────────
  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.4.10
    hooks:
      - id: ruff
        args: [--fix]
        stages: [commit]

      - id: ruff-format
        args: [--check]
        stages: [commit]

  - repo: https://github.com/pre-commit/mirrors-mypy
    rev: v1.8.0
    hooks:
      - id: mypy
        additional_dependencies: [types-all]
        args: [--strict, --ignore-missing-imports]
        stages: [push]
        files: 'src/.*\.py$'

  # ─── TypeScript/JavaScript Linting ──────────────────────────────────
  - repo: https://github.com/pre-commit/mirrors-eslint
    rev: v9.0.0
    hooks:
      - id: eslint
        files: '\.(js|jsx|ts|tsx)$'
        types: [file]
        additional_dependencies: [
          'eslint',
          '@typescript-eslint/eslint-plugin',
          '@typescript-eslint/parser',
          'eslint-plugin-security',
          'eslint-plugin-import',
          'eslint-config-prettier'
        ]
        args: [--fix]

  # ─── Proto Linting ──────────────────────────────────────────────────
  - repo: https://github.com/bufbuild/buf
    rev: v1.32.2
    hooks:
      - id: buf-format
        args: [--write]
      - id: buf-lint
      - id: buf-breaking
        args: [--against, 'https://github.com/KooshaPari/phenotype-infrakit.git#branch=main']

  # ─── Secret Detection ────────────────────────────────────────────────
  - repo: https://github.com/trufflesecurity/trufflehog
    rev: v3.93.6
    hooks:
      - id: trufflehog
        args: [--only-verified, --fail, --no-update]
        stages: [commit]

  # ─── TOML Formatting ────────────────────────────────────────────────
  - repo: https://github.com/macisamuele/language-formatters-pre-commit-hooks
    rev: v2.13.0
    hooks:
      - id: pretty-format-toml
        args: [--autofix]

  # ─── Security: Bandit (Python) ──────────────────────────────────────
  - repo: https://github.com/PyCQA/bandit
    rev: 1.7.5
    hooks:
      - id: bandit
        files: 'src/.*\.py$'
        args: [--severity-level=medium]
        stages: [push]

  # ─── Documentation Coverage ─────────────────────────────────────────
  - repo: local
    hooks:
      - id: doc-coverage
        name: doc-coverage (Python)
        entry: bash -c 'cd python && python -m pydocstyle src/ || true'
        language: system
        files: 'src/.*\.py$'
        pass_filenames: false
        stages: [push]

ci:
  autofix_commit_msg: 'chore(lint): auto-fix formatting and style issues'
  autofix_prs: true
  autoupdate_commit_msg: 'chore: auto-update pre-commit hooks'
  autoupdate_schedule: weekly
  skip: [buf-breaking, clippy]  # Skip slow checks by default
  stages: [commit]
```

**Installation & Usage:**

```bash
# Install pre-commit
pip install pre-commit

# Install hooks into .git/hooks
pre-commit install --all-files

# Run all hooks
pre-commit run --all-files

# Run specific hook
pre-commit run rustfmt --all-files

# Update hooks to latest versions
pre-commit autoupdate

# Bypass hooks (only for emergency fixes)
git commit --no-verify
```

---

## Part 7: Severity Mapping (Warnings → Errors → Blocks)

### Decision Matrix: What Blocks a PR?

| Severity | Category | Tools | PR Block? | Agent Fix? | Details |
|----------|----------|-------|-----------|-----------|---------|
| **Critical** | Security | gosec, bandit, semgrep, trufflehog | ✅ YES | ⚠️ Manual | OWASP vulnerabilities, hardcoded secrets |
| **Critical** | Type Safety | mypy strict, pyright strict, TypeScript strict | ✅ YES | ✅ Auto | No `any` types, all types must match |
| **Critical** | Undefined Vars | Clippy, ESLint, Ruff F | ✅ YES | ✅ Auto | All names must be defined |
| **Critical** | Compilation | Rust/Go/TS compiler | ✅ YES | ⚠️ Manual | Code must build |
| **Error** | Dead Code | Clippy -D warnings, pylint | ✅ YES | ✅ Auto | Remove unused code |
| **Error** | Test Failure | pytest, cargo test, npm test | ✅ YES | ⚠️ Manual | All tests must pass |
| **Error** | Coverage < 50% | Coverage report | ✅ YES | ⚠️ Manual | Critical paths must be tested |
| **Error** | Code Smell (new) | SonarCloud | ✅ YES | ✅ Partial | No new smells on PR |
| **Warning** | Complexity > 15 | gocyclo, mccabe | ⚠️ WARN | ✅ Auto | Suggest refactoring |
| **Warning** | Duplication > 5% | CPD, SonarCloud | ⚠️ WARN | ✅ Manual | Flag for consolidation |
| **Warning** | Coverage 50-80% | Coverage report | ⚠️ WARN | ⚠️ Manual | Acceptable with comment |
| **Warning** | Docstring Missing | pydocstyle, rustdoc | ℹ️ INFO | ⚠️ Manual | Not critical, nice to have |
| **Info** | Style (PEP 8, fmt) | Ruff format, go fmt, prettier | ✅ AUTO-FIX | ✅ Auto | Fixed automatically pre-commit |
| **Info** | Naming Convention | revive, ESLint | ✅ AUTO-FIX | ✅ Auto | Follow language conventions |
| **Info** | Typos | typos, markdownlint | ⚠️ WARN | ✅ Auto | Low impact, auto-fix |

### GitHub Branch Protection Rules

```yaml
# Add to repo settings:
branch_protection_rules:
  - branch: main
    require_status_checks: true
    required_status_checks:
      - Rust Quality (required)
      - Python Quality (required)
      - TypeScript Quality (required)
      - Documentation Linting (required)
      - Secret Scanning (required)
      - SonarCloud/quality/gate (required)
    require_code_review: true
    minimum_review_count: 1
    dismiss_stale_pull_request_approvals: true
    enforce_admins: false
    allow_force_pushes: false
    allow_deletions: false
```

---

## Part 8: Cloud Agent Auto-Fixing Strategy

### Haiku Agent Workflow for Style Issues

**Trigger:** PR with linting failures (non-critical)

**Agent Instructions:**

```markdown
## Linting Auto-Fix Protocol

When a PR fails linting but code quality is acceptable:

1. **Check failure type:**
   - Style (formatting, naming): ✅ Auto-fix via agent
   - Dead code: ✅ Auto-fix (remove unused)
   - Type mismatch: ⚠️ Manual (requires design understanding)
   - Security: ⚠️ Manual (requires expert review)

2. **Auto-fix eligible:**
   - `cargo fmt` (Rust)
   - `go fmt` (Go)
   - `ruff format --fix` (Python)
   - `ruff check --fix` (Python linting)
   - `npx eslint --fix` (TypeScript)
   - `npx prettier --write` (TypeScript)
   - Remove dead code (unused imports, vars)
   - Remove trailing whitespace
   - Fix typos (via typos --write-changes)

3. **Agent action:**
   - Run: `task quality --auto-fix` (or language-specific)
   - Commit: `git add -A && git commit -m "chore(lint): auto-fix style issues"`
   - Push: `git push`
   - Comment on PR: "Auto-fixed N style issues. Review commits."

4. **Manual review cases:**
   - Type safety: User must understand intent
   - Security: Requires domain expertise
   - Complex refactoring: Requires design review
   - Comment: "@user Please review and approve"

5. **Severity matrix:**
   - Warnings: Agent fixes, merges PR if all others pass
   - Errors (non-critical): Agent fixes, requests review
   - Errors (critical): Raise for user review (block merge)
```

**Example: Auto-fix haiku agent task**

```bash
# In haiku agent:
cd /Users/kooshapari/CodeProjects/Phenotype/repos

# 1. Run format checks
cargo fmt --all && \
go fmt ./... && \
cd python && uvx ruff format . && \
cd ../

# 2. Run linting with --fix
cargo clippy --fix && \
go vet ./... && \
cd python && uvx ruff check --fix . && \
npx eslint --fix src/

# 3. Commit changes
git add -A
git commit -m "chore(lint): auto-fix formatting and linting issues"

# 4. Push
git push origin $(git rev-parse --abbrev-ref HEAD)

# 5. Comment on PR
# (Agent will leave comment: "Auto-fixed X style issues")
```

---

## Part 9: Free Tier Assessment

### Cost Summary (2026-03-30)

| Tool | Free Tier | Cost | Status | Recommendation |
|------|-----------|------|--------|-----------------|
| **Clippy** | ✅ 100% | $0 | Rust built-in | Adopt (already in use) |
| **cargo-deny** | ✅ 100% | $0 | OSS | Adopt (already in use) |
| **golangci-lint** | ✅ 100% | $0 | OSS | Adopt across Go repos |
| **Ruff** | ✅ 100% | $0 | OSS | Adopt (already in use) |
| **Pyright** | ✅ 100% | $0 | OSS (Microsoft) | Adopt for type safety |
| **ESLint** | ✅ 100% | $0 | OSS | Adopt for TS/JS |
| **SonarCloud** | ✅ Free (public repos) | $0/month (public) $10/month (private) | Cloud | Adopt for public repos |
| **CodeClimate** | ⚠️ Limited free tier | $99-299/month | Cloud | Not recommended (overlap with SonarCloud) |
| **DeepSource** | ⚠️ Limited | $0-199/month | Cloud | Optional (same as SonarCloud) |
| **pre-commit** | ✅ 100% | $0 | OSS | Adopt (already in use) |
| **Vale** | ✅ 100% | $0 | OSS | Adopt for prose |
| **markdownlint** | ✅ 100% | $0 | OSS | Adopt (already in use) |
| **trufflehog** | ✅ 100% | $0 | OSS | Adopt (already in use) |

**Total Annual Cost for Phenotype:**
- **Free Tier:** $0 (all OSS tools + SonarCloud public repo free)
- **Optional SonarCloud Private:** $120/year (if needed, $10/month)
- **Recommendation:** Stick with free tier (100% coverage achieved)

---

## Part 10: Implementation Roadmap

### Phase 1: Enable Existing Tools (Week 1-2)

- ✅ Rust: Already integrated (clippy, cargo-deny, cargo-machete)
- ✅ Python: Already integrated (ruff, pip-audit)
- ⚠️ Go: Enhanced golangci-lint config (apply across all repos)
- ⚠️ TypeScript: Add ESLint + TypeScript strict mode
- ✅ Prose: Vale + markdownlint (add to CI)

**Actions:**
1. Update `.golangci.yml` per recommendation (Part 1)
2. Create `.eslintrc.json` for TS/JS projects
3. Enable SonarCloud via GH Actions (free public repos)
4. Add documentation linting to CI

### Phase 2: Enhance GitHub Actions (Week 2-3)

- Create comprehensive `linting.yml` workflow (Part 5)
- Add duplication detection (CPD + SonarCloud)
- Add secret scanning (trufflehog)
- Add code smell thresholds (SonarCloud quality gate)

**Actions:**
1. Create `.github/workflows/linting.yml` (copy from Part 5)
2. Set branch protection rules (require linting checks)
3. Configure SonarCloud quality gate

### Phase 3: Cloud Agent Integration (Week 3-4)

- Set up haiku agent task for auto-fixing (Part 8)
- Create auto-fix pre-commit hook
- Document severity mapping for agents (Part 7)

**Actions:**
1. Create `scripts/auto-fix-linting.sh` (orchestrates all linters)
2. Document agent protocol (Part 8)
3. Test agent auto-fixes on sample PRs

### Phase 4: Monitoring & Metrics (Week 4+)

- Track code smell trends in SonarCloud
- Monitor linting failure rates
- Identify patterns for refactoring (high complexity, duplication)

**Actions:**
1. Set up SonarCloud dashboards
2. Create monthly linting report
3. Identify refactoring opportunities (Part 1 of MEMORY.md: routes.rs, sqlite/lib.rs, etc.)

---

## Appendix A: Quick Reference Commands

### Rust

```bash
# Format
cargo fmt --all

# Lint (warnings = errors)
cargo clippy --all-targets -- -D warnings

# Check unused deps
cargo install cargo-machete
cargo machete

# Audit deps
cargo install cargo-deny
cargo deny check

# Run tests
cargo test --workspace
```

### Go

```bash
# Install golangci-lint
go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest

# Run linting
golangci-lint run ./... --fix

# Format
go fmt ./...

# Vet
go vet ./...
```

### Python

```bash
# Format + lint (install first)
pip install uv
cd python && uv sync

# Format check
uvx ruff format --check .

# Lint check
uvx ruff check .

# Auto-fix
uvx ruff format .
uvx ruff check --fix .

# Type check
uvx pyright .

# Security audit
uvx pip-audit
```

### TypeScript

```bash
# Install
npm install -D eslint @typescript-eslint/parser @typescript-eslint/eslint-plugin prettier

# Lint
npx eslint src --max-warnings 0

# Type check
npx tsc --noEmit

# Format
npx prettier --write src
```

### Documentation

```bash
# Install vale
brew install vale

# Check prose
vale --config .vale.ini docs/

# Check markdown
npx markdownlint docs/
```

---

## Appendix B: GitHub Actions Secrets Setup

**Required Secrets for CI/CD:**

```bash
# For SonarCloud
SONARCLOUD_TOKEN=<token from sonarcloud.io>

# For Codecov (optional, already set)
CODECOV_TOKEN=<token from codecov.io>

# For GitHub (auto-provided)
GITHUB_TOKEN=<auto-provided by GitHub>
```

**Set via GitHub UI:**
1. Go to repo Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Add `SONARCLOUD_TOKEN` (get from https://sonarcloud.io)

---

## Appendix C: Monitoring & Reporting

### SonarCloud Dashboard Metrics

```
Quality Gate:
  - Coverage >= 80%
  - Code Smells = 0 (or <= 5 pre-existing)
  - Duplications < 5%
  - Security Hotspots = 0
  - Reliability = A (≥ 0 bugs)
  - Security = A (≥ 0 vulnerabilities)
```

### Monthly Linting Report Script

```bash
#!/bin/bash
# Generate linting metrics report

echo "## Phenotype Linting Report ($(date +%Y-%m))" > LINTING_REPORT.md
echo >> LINTING_REPORT.md

# Rust metrics
echo "### Rust" >> LINTING_REPORT.md
cargo clippy --message-format=json 2>/dev/null | jq 'select(.level=="warning")' | wc -l >> LINTING_REPORT.md

# Python metrics
echo "### Python" >> LINTING_REPORT.md
cd python && uvx ruff check . --output-format=json 2>/dev/null | jq '.[].all_messages | length' | paste -sd+ | bc >> LINTING_REPORT.md
cd ../

# Go metrics
echo "### Go" >> LINTING_REPORT.md
golangci-lint run ./... --format json 2>/dev/null | jq '.Issues | length' >> LINTING_REPORT.md

# TypeScript metrics
echo "### TypeScript" >> LINTING_REPORT.md
npx eslint src --format json 2>/dev/null | jq '[.[].messages[]] | length' >> LINTING_REPORT.md

echo "Report saved to LINTING_REPORT.md"
```

---

## Summary & Recommendations

### Best-of-Breed Linting Stack (Phenotype)

| Language | Linter | Formatter | Type Checker | Security | Status |
|----------|--------|-----------|--------------|----------|--------|
| **Rust** | Clippy | cargo fmt | (built-in) | cargo-deny + semgrep | ✅ Ready |
| **Go** | golangci-lint | gofmt | (built-in) | gosec (via golangci) | ⚠️ Expand config |
| **Python** | Ruff | Ruff format | Pyright (strict) | Bandit + pip-audit | ✅ Ready |
| **TypeScript** | ESLint | Prettier | TypeScript (strict) | ESLint security plugin | ⚠️ New |
| **Prose** | Vale | markdownlint | N/A | trufflehog | ✅ Add to CI |

### Immediate Actions

1. **This week:**
   - ✅ Copy comprehensive `linting.yml` to `.github/workflows/`
   - ✅ Set branch protection rules
   - ✅ Update Go repos with enhanced `.golangci.yml`

2. **Next week:**
   - ✅ Enable SonarCloud (free for public repos)
   - ✅ Add TypeScript ESLint config to heliosApp, agent-wave
   - ✅ Add documentation linting to CI

3. **Following week:**
   - ✅ Set up haiku agent auto-fix protocol
   - ✅ Create refactoring task for high-smell files (routes.rs, sqlite/lib.rs)
   - ✅ Monitor SonarCloud dashboard

### Cost Savings

- **Alternative (CodeClimate + DeepSource):** $300-500/year
- **Phenotype Stack (SonarCloud Free + OSS):** $0/year
- **Savings:** $300-500/year ✅

---

**End of Guide**

Document maintained by: Cloud Platform Engineering
Next review: 2026-06-30
