# GitHub Actions Test Workflows — Ready-to-Use Templates

**Version**: 2026-03-30
**Purpose**: Copy-paste workflows for Python, Rust, Go, and JavaScript/TypeScript testing
**Coverage Enforcement**: 80% minimum, with PR comments

---

## Table of Contents

1. [Python (pytest + coverage)](#python-pytest--coverage)
2. [Rust (cargo + nextest + mutation)](#rust-cargo--nextest--mutation)
3. [Go (go test + coverage)](#go-go-test--coverage)
4. [JavaScript/TypeScript (Vitest + Playwright)](#javascripttypescript-vitest--playwright)
5. [Multi-Language Monorepo](#multi-language-monorepo)
6. [Coverage Threshold Config](#coverage-threshold-config)

---

## Python (pytest + coverage)

**File**: `.github/workflows/test-python.yml`

```yaml
name: Python Tests

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        python-version: ["3.9", "3.10", "3.11", "3.12"]
    timeout-minutes: 30

    steps:
      - uses: actions/checkout@v4

      - name: Set up Python ${{ matrix.python-version }}
        uses: actions/setup-python@v4
        with:
          python-version: ${{ matrix.python-version }}
          cache: "pip"

      - name: Install dependencies
        run: |
          python -m pip install --upgrade pip
          pip install -r requirements-dev.txt
          pip install pytest pytest-cov pytest-xdist pytest-timeout

      - name: Lint with flake8
        run: |
          pip install flake8
          # Stop on syntax errors or undefined names
          flake8 src tests --count --select=E9,F63,F7,F82 --show-source --statistics
          # Exit-zero treats all errors as warnings
          flake8 src tests --count --exit-zero --max-complexity=10 --max-line-length=127 --statistics

      - name: Run tests with coverage (parallel)
        run: |
          pytest \
            --cov=src \
            --cov-report=xml \
            --cov-report=html \
            --cov-report=term-missing \
            --junitxml=test-results.xml \
            -n auto \
            -v \
            --tb=short

      - name: Check coverage threshold
        run: |
          python -m coverage report --fail-under=80

      - name: Run mutation tests
        if: github.event_name == 'pull_request'
        continue-on-error: true
        run: |
          pip install pymute
          pymute run --code-dir src --test-dir tests

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v5
        with:
          files: ./coverage.xml
          flags: unittests
          name: codecov-umbrella
          fail_ci_if_error: true
          verbose: true

      - name: Archive test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: test-results-${{ matrix.python-version }}
          path: |
            test-results.xml
            htmlcov/
            .coverage

      - name: Comment PR with coverage
        if: github.event_name == 'pull_request'
        uses: py-cov-action/python-coverage-comment-action@v3
        with:
          GITHUB_TOKEN: ${{ github.token }}
          MINIMUM_GREEN: 80
          MINIMUM_ORANGE: 70

  mutation-testing:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    timeout-minutes: 60

    steps:
      - uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v4
        with:
          python-version: "3.11"
          cache: "pip"

      - name: Install dependencies
        run: |
          pip install -r requirements-dev.txt
          pip install pymute

      - name: Run mutation tests
        id: mutation
        run: |
          pymute run --code-dir src --test-dir tests > mutation-report.txt 2>&1
          cat mutation-report.txt

      - name: Upload mutation report
        uses: actions/upload-artifact@v4
        with:
          name: mutation-report
          path: mutation-report.txt

      - name: Comment PR with mutation results
        if: always()
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('mutation-report.txt', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `## Mutation Testing Report\n\`\`\`\n${report}\n\`\`\``
            });
```

---

## Rust (cargo + nextest + mutation)

**File**: `.github/workflows/test-rust.yml`

```yaml
name: Rust Tests

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    runs-on: ubuntu-latest
    timeout-minutes: 30

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Run clippy
        run: cargo clippy --workspace --all-targets --all-features -- -D warnings

      - name: Run tests (standard)
        run: cargo test --workspace --verbose

      - name: Install nextest
        run: cargo install cargo-nextest

      - name: Run tests (parallel with nextest)
        run: cargo nextest run --workspace

      - name: Generate coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --workspace --out Xml --output-dir target/coverage

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v5
        with:
          files: ./target/coverage/cobertura.xml
          flags: unittests
          fail_ci_if_error: true

      - name: Archive coverage report
        uses: actions/upload-artifact@v4
        with:
          name: coverage-report
          path: target/coverage/

  mutation-testing:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    timeout-minutes: 90

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2

      - name: Install cargo-mutants
        run: cargo install cargo-mutants

      - name: Run mutation tests
        id: mutation
        run: |
          cargo mutants -v --output mutation-report.txt
          cat mutation-report.txt

      - name: Upload mutation report
        uses: actions/upload-artifact@v4
        with:
          name: mutation-report
          path: mutation-report.txt

      - name: Parse and comment mutation results
        if: always()
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('mutation-report.txt', 'utf8');
            const lines = report.split('\n').slice(0, 30); // Limit to first 30 lines
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `## Mutation Testing Results\n\`\`\`\n${lines.join('\n')}\n\`\`\``
            });

  security-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run cargo audit
        uses: rustsec/audit-check-action@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

---

## Go (go test + coverage)

**File**: `.github/workflows/test-go.yml`

```yaml
name: Go Tests

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

env:
  GO_VERSION: "1.22"

jobs:
  test:
    runs-on: ubuntu-latest
    timeout-minutes: 30

    steps:
      - uses: actions/checkout@v4

      - name: Set up Go
        uses: actions/setup-go@v4
        with:
          go-version: ${{ env.GO_VERSION }}
          cache: true

      - name: Run go vet
        run: go vet ./...

      - name: Run tests with coverage
        run: |
          go test \
            -v \
            -race \
            -timeout 10m \
            -coverprofile=coverage.out \
            -covermode=atomic \
            ./...

      - name: Check coverage threshold
        run: |
          threshold=80
          coverage=$(go tool cover -func=coverage.out | grep total | awk '{print int($3)}')
          echo "Total coverage: ${coverage}%"
          if [ "$coverage" -lt "$threshold" ]; then
            echo "Coverage ${coverage}% is below threshold ${threshold}%"
            exit 1
          fi

      - name: Generate HTML coverage report
        run: go tool cover -html=coverage.out -o coverage.html

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v5
        with:
          files: ./coverage.out
          flags: unittests
          fail_ci_if_error: true

      - name: Archive coverage
        uses: actions/upload-artifact@v4
        with:
          name: coverage-report
          path: |
            coverage.out
            coverage.html

  benchmark:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    steps:
      - uses: actions/checkout@v4

      - name: Set up Go
        uses: actions/setup-go@v4
        with:
          go-version: ${{ env.GO_VERSION }}
          cache: true

      - name: Run benchmarks
        run: go test -bench=. -benchmem -run=^$ ./... | tee benchmarks.txt

      - name: Comment benchmarks on PR
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const benchmarks = fs.readFileSync('benchmarks.txt', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `## Benchmark Results\n\`\`\`\n${benchmarks}\n\`\`\``
            });

  linting:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Go
        uses: actions/setup-go@v4
        with:
          go-version: ${{ env.GO_VERSION }}

      - name: golangci-lint
        uses: golangci/golangci-lint-action@v3
        with:
          version: latest
          args: --timeout=5m
```

---

## JavaScript/TypeScript (Vitest + Playwright)

**File**: `.github/workflows/test-node.yml`

```yaml
name: Node Tests

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

env:
  NODE_VERSION: "20"

jobs:
  test:
    runs-on: ubuntu-latest
    timeout-minutes: 30

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: ${{ env.NODE_VERSION }}
          cache: "npm"

      - name: Install dependencies
        run: npm ci

      - name: Lint code
        run: npm run lint --if-present

      - name: Format check
        run: npm run format:check --if-present

      - name: Run unit tests with coverage
        run: npm run test -- --coverage

      - name: Check coverage threshold
        run: |
          node << 'EOF'
          const fs = require('fs');
          const coverage = JSON.parse(fs.readFileSync('coverage/coverage-final.json', 'utf8'));
          let lines = 0, covered = 0;
          Object.values(coverage).forEach(file => {
            Object.values(file.l).forEach(line => {
              lines++;
              if (line > 0) covered++;
            });
          });
          const percent = Math.round((covered / lines) * 100);
          console.log(`Coverage: ${percent}%`);
          process.exit(percent >= 80 ? 0 : 1);
          EOF

      - name: Run mutation tests (Stryker)
        if: github.event_name == 'pull_request'
        continue-on-error: true
        run: npx stryker run || true

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v5
        with:
          files: ./coverage/coverage-final.json
          flags: unittests
          fail_ci_if_error: true

      - name: Archive coverage
        uses: actions/upload-artifact@v4
        with:
          name: coverage-report
          path: coverage/

  e2e-tests:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    timeout-minutes: 30

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: ${{ env.NODE_VERSION }}
          cache: "npm"

      - name: Install dependencies
        run: npm ci

      - name: Install Playwright browsers
        run: npx playwright install --with-deps

      - name: Run E2E tests
        run: npm run test:e2e --if-present

      - name: Upload Playwright report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: playwright-report
          path: playwright-report/
          retention-days: 30

  mutation-testing:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    timeout-minutes: 60

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: ${{ env.NODE_VERSION }}
          cache: "npm"

      - name: Install dependencies
        run: npm ci

      - name: Run Stryker mutation tests
        id: stryker
        run: npx stryker run > mutation-report.txt 2>&1 || true

      - name: Upload mutation report
        uses: actions/upload-artifact@v4
        with:
          name: mutation-report
          path: mutation-report.txt

      - name: Comment mutation results on PR
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('mutation-report.txt', 'utf8').slice(0, 2000);
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `## Mutation Testing Report\n\`\`\`\n${report}\n\`\`\``
            });
```

---

## Multi-Language Monorepo

**File**: `.github/workflows/test-monorepo.yml`

```yaml
name: Monorepo Tests

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  detect-changes:
    runs-on: ubuntu-latest
    outputs:
      python: ${{ steps.filter.outputs.python }}
      rust: ${{ steps.filter.outputs.rust }}
      go: ${{ steps.filter.outputs.go }}
      node: ${{ steps.filter.outputs.node }}

    steps:
      - uses: actions/checkout@v4

      - uses: dorny/paths-filter@v2
        id: filter
        with:
          filters: |
            python:
              - 'src/python/**'
              - 'tests/python/**'
            rust:
              - 'crates/**'
              - 'Cargo.toml'
            go:
              - 'cmd/**'
              - 'pkg/**'
              - 'go.mod'
            node:
              - 'apps/**'
              - 'packages/**'
              - 'package.json'

  test-python:
    needs: detect-changes
    if: needs.detect-changes.outputs.python == 'true'
    uses: ./.github/workflows/test-python.yml

  test-rust:
    needs: detect-changes
    if: needs.detect-changes.outputs.rust == 'true'
    uses: ./.github/workflows/test-rust.yml

  test-go:
    needs: detect-changes
    if: needs.detect-changes.outputs.go == 'true'
    uses: ./.github/workflows/test-go.yml

  test-node:
    needs: detect-changes
    if: needs.detect-changes.outputs.node == 'true'
    uses: ./.github/workflows/test-node.yml

  all-tests-passed:
    runs-on: ubuntu-latest
    needs: [test-python, test-rust, test-go, test-node]
    if: always()

    steps:
      - name: Check test results
        run: |
          if [[ "${{ needs.test-python.result }}" == "failure" || \
                "${{ needs.test-rust.result }}" == "failure" || \
                "${{ needs.test-go.result }}" == "failure" || \
                "${{ needs.test-node.result }}" == "failure" ]]; then
            echo "One or more test jobs failed"
            exit 1
          fi
```

---

## Coverage Threshold Config

**File**: `codecov.yml` (at repo root)

```yaml
# Coverage enforcement configuration for Codecov

coverage:
  precision: 2
  round: down
  range: [70, 100]  # Pass if coverage between 70-100%

threshold:
  absolute: 80      # Overall project coverage must be 80%+
  relative: 5       # New code must not reduce coverage by more than 5%

patch:
  target: 80        # New code must be 80%+ covered
  only_pulls: true  # Only check patches on PRs

ignore:
  - tests           # Don't count test files in coverage
  - examples
  - docs
  - "**/__pycache__"

comment:
  layout: "reach,diff,flags,tree"
  behavior: default
  require_changes: true   # Fail if coverage decreases
  require_base: true      # Require base branch in comparison

status:
  project: true           # Check entire project coverage
  patch: true             # Check patch coverage
  changes: true           # Check changes coverage
```

**File**: `pyproject.toml` (Python-specific)

```toml
[tool.pytest.ini_options]
testpaths = ["tests"]
addopts = "--cov=src --cov-report=xml --cov-report=html --cov-report=term-missing"
minversion = "7.0"

[tool.coverage.run]
branch = true
source = ["src"]
omit = [
    "*/tests/*",
    "*/migrations/*",
    "*/__pycache__/*",
    "*/site-packages/*",
]

[tool.coverage.report]
fail_under = 80
precision = 2
exclude_lines = [
    "pragma: no cover",
    "def __repr__",
    "raise AssertionError",
    "raise NotImplementedError",
    "if __name__ == .__main__.:",
    "if TYPE_CHECKING:",
    "class .*\\bProtocol\\):",
    "@(abc\\.)?abstractmethod",
]
```

**File**: `.cargo/config.toml` (Rust)

```toml
[build]
target-dir = "target"

[term]
verbose = false
```

**File**: `vitest.config.ts` (Vitest)

```typescript
import { defineConfig } from "vitest/config";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
  test: {
    globals: true,
    environment: "happy-dom",
    coverage: {
      provider: "v8",
      reporter: ["text", "json", "html", "lcov"],
      exclude: [
        "node_modules/",
        "dist/",
        "coverage/",
        "**/*.test.ts",
        "**/*.spec.ts",
      ],
      lines: 80,
      functions: 80,
      branches: 80,
      statements: 80,
    },
  },
});
```

---

## Usage Instructions

### 1. Copy Workflows

```bash
mkdir -p .github/workflows
cp docs/reference/GITHUB_ACTIONS_TEST_WORKFLOWS.md .github/workflows/

# Copy individual workflows based on your language
cp test-python.yml .github/workflows/  # if using Python
cp test-rust.yml .github/workflows/    # if using Rust
cp test-go.yml .github/workflows/      # if using Go
cp test-node.yml .github/workflows/    # if using JavaScript/TypeScript
```

### 2. Create `codecov.yml`

```bash
cp codecov.yml .  # Place at repo root
```

### 3. Configure Language-Specific Files

- **Python**: Add `[tool.pytest]` and `[tool.coverage]` sections to `pyproject.toml`
- **Rust**: Install coverage tool: `cargo install cargo-tarpaulin`
- **Go**: Already built-in via `go test`
- **JavaScript/TypeScript**: Add `vitest.config.ts` or `jest.config.ts`

### 4. Enable GitHub Status Checks

In GitHub repo settings:

1. Navigate to **Settings → Branches → main**
2. Under "Require status checks to pass before merging":
   - Add `codecov/project` (coverage)
   - Add `codecov/patch` (new code coverage)
   - Add language-specific test jobs: `test-python`, `test-rust`, `test-go`, `test-node`

### 5. Test Locally Before Pushing

```bash
# Python
pytest --cov=src --cov-report=term-missing

# Rust
cargo test --workspace

# Go
go test -cover ./...

# JavaScript/TypeScript
npm run test -- --coverage
```

---

## Customization Guide

### Adjust Coverage Threshold

Edit `codecov.yml`:
```yaml
threshold:
  absolute: 75  # Change to 75% if 80% is too strict
```

### Add Python Version Matrix

Edit `test-python.yml`:
```yaml
matrix:
  python-version: ["3.8", "3.9", "3.10", "3.11", "3.12"]
```

### Enable Weekly Mutation Testing Only

Edit `test-rust.yml`:
```yaml
on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday
```

### Exclude Files from Coverage

**Python** (`pyproject.toml`):
```toml
[tool.coverage.run]
omit = ["src/migrations/*", "src/legacy/*"]
```

**JavaScript** (`vitest.config.ts`):
```typescript
coverage: {
  exclude: ["src/legacy/**", "src/migrations/**"]
}
```

---

## Troubleshooting

### Codecov Token Issues

For private repos, add token to secrets:

```bash
# GitHub Settings → Secrets → New repository secret
# Name: CODECOV_TOKEN
# Value: (get from https://codecov.io/account)
```

Then in workflow:

```yaml
- uses: codecov/codecov-action@v5
  with:
    token: ${{ secrets.CODECOV_TOKEN }}
    fail_ci_if_error: true
```

### Slow Parallel Tests

Reduce parallelism:

```yaml
# Python
pytest -n 2  # Instead of -n auto

# Rust
cargo nextest run --test-threads 2
```

### Coverage Report Not Uploading

Verify file exists:

```bash
# Python
ls -la coverage.xml

# Rust
ls -la target/coverage/cobertura.xml

# Go
ls -la coverage.out

# JavaScript
ls -la coverage/coverage-final.json
```

---

**Last Updated**: March 30, 2026
**For Issues**: Reference `/docs/reference/QA_TESTING_TOOLS_GUIDE.md`
