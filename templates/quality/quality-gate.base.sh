#!/usr/bin/env bash
# Quality Gate Base Template
# Location: thegent/templates/quality/quality-gate.base.sh
#
# USAGE:
# 1. Copy to your project as: hooks/quality-gate.sh or scripts/quality-gate.sh
# 2. Customize sections marked [CUSTOMIZE] for your project
# 3. Make executable: chmod +x quality-gate.sh
# 4. Integrate with your CI/CD and git hooks
#
# VERSION: 1.0 (2026-03)
# LANGUAGE: Bash
# DEPENDENCIES: bash, git, command-v (posix)

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

SCRIPT_NAME="$(basename "$0")"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Output formatting
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Quality gate result tracking
GATE_PASSED=true
GATE_ERRORS=()
GATE_WARNINGS=()

# ============================================================================
# Utility Functions
# ============================================================================

log_info() {
  echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
  echo -e "${GREEN}[PASS]${NC} $*"
}

log_warning() {
  echo -e "${YELLOW}[WARN]${NC} $*"
  GATE_WARNINGS+=("$*")
}

log_error() {
  echo -e "${RED}[FAIL]${NC} $*"
  GATE_ERRORS+=("$*")
  GATE_PASSED=false
}

# Check if command exists
command_exists() {
  command -v "$1" >/dev/null 2>&1
}

# Run a quality check
run_check() {
  local check_name="$1"
  local check_command="$2"

  log_info "Running: $check_name"
  if eval "$check_command"; then
    log_success "$check_name passed"
    return 0
  else
    log_error "$check_name failed"
    return 1
  fi
}

# ============================================================================
# GATE 1: Syntax & Format Validation
# ============================================================================

gate1_syntax_validation() {
  log_info "=== GATE 1: Syntax & Format Validation ==="

  # YAML validation
  if command_exists yq; then
    log_info "Validating YAML files..."
    if find . -name "*.yaml" -o -name "*.yml" | xargs yq '.' >/dev/null 2>&1; then
      log_success "YAML syntax valid"
    else
      log_error "YAML syntax invalid"
    fi
  fi

  # [CUSTOMIZE] Add more syntax checks for your project:
  # - JSON validation
  # - TOML validation
  # - Custom format checks
}

# ============================================================================
# GATE 2: Linting & Formatting
# ============================================================================

gate2_linting() {
  log_info "=== GATE 2: Linting & Formatting ==="

  # [CUSTOMIZE] Python (ruff)
  if command_exists ruff && [[ -f "pyproject.toml" ]] && ls -1 *.py >/dev/null 2>&1; then
    log_info "Running Python linting (ruff)..."
    if ruff check . --fix; then
      log_success "Python linting passed"
    else
      log_error "Python linting failed"
    fi
  fi

  # [CUSTOMIZE] Rust (rustfmt + clippy)
  if command_exists cargo && [[ -f "Cargo.toml" ]]; then
    log_info "Running Rust formatting (rustfmt)..."
    if cargo fmt -- --check; then
      log_success "Rust formatting passed"
    else
      log_error "Rust formatting failed"
    fi

    log_info "Running Rust linting (clippy)..."
    if cargo clippy --all-targets -- -D warnings; then
      log_success "Rust linting passed"
    else
      log_error "Rust linting failed"
    fi
  fi

  # [CUSTOMIZE] TypeScript/JavaScript (oxlint + prettier)
  if command_exists oxlint && ls -1 *.{ts,tsx,js,jsx} >/dev/null 2>&1; then
    log_info "Running JavaScript linting (oxlint)..."
    if oxlint . --fix; then
      log_success "JavaScript linting passed"
    else
      log_error "JavaScript linting failed"
    fi
  fi

  if command_exists prettier && ls -1 *.{ts,tsx,js,jsx,json,md} >/dev/null 2>&1; then
    log_info "Running code formatting (prettier)..."
    if prettier --check .; then
      log_success "Code formatting passed"
    else
      log_error "Code formatting failed"
    fi
  fi

  # [CUSTOMIZE] Go (gofmt + golangci-lint)
  if command_exists go && ls -1 *.go >/dev/null 2>&1; then
    log_info "Running Go formatting (gofmt)..."
    if gofmt -l . | grep -q .; then
      log_error "Go formatting failed"
    else
      log_success "Go formatting passed"
    fi
  fi
}

# ============================================================================
# GATE 3: Type Checking
# ============================================================================

gate3_type_checking() {
  log_info "=== GATE 3: Type Checking ==="

  # [CUSTOMIZE] Python (basedpyright)
  if command_exists basedpyright && ls -1 *.py >/dev/null 2>&1; then
    log_info "Running Python type check (basedpyright)..."
    if basedpyright .; then
      log_success "Python type checking passed"
    else
      log_warning "Python type checking found issues (non-blocking)"
    fi
  fi

  # [CUSTOMIZE] TypeScript (tsc)
  if command_exists tsc && [[ -f "tsconfig.json" ]]; then
    log_info "Running TypeScript type check..."
    if tsc --noEmit; then
      log_success "TypeScript type checking passed"
    else
      log_error "TypeScript type checking failed"
    fi
  fi

  # [CUSTOMIZE] Rust type checking (via cargo check)
  if command_exists cargo && [[ -f "Cargo.toml" ]]; then
    log_info "Running Cargo type check..."
    if cargo check --all-targets; then
      log_success "Cargo type checking passed"
    else
      log_error "Cargo type checking failed"
    fi
  fi
}

# ============================================================================
# GATE 4: Testing
# ============================================================================

gate4_testing() {
  log_info "=== GATE 4: Testing ==="

  # [CUSTOMIZE] Python (pytest)
  if command_exists pytest && [[ -d "tests" ]]; then
    log_info "Running Python tests (pytest)..."
    if pytest tests/ -v --tb=short; then
      log_success "Python tests passed"
    else
      log_error "Python tests failed"
    fi
  fi

  # [CUSTOMIZE] Rust (cargo test)
  if command_exists cargo && [[ -f "Cargo.toml" ]]; then
    log_info "Running Rust tests (cargo)..."
    if cargo test --all; then
      log_success "Rust tests passed"
    else
      log_error "Rust tests failed"
    fi
  fi

  # [CUSTOMIZE] JavaScript (vitest or jest)
  if command_exists vitest && [[ -f "vitest.config.ts" ]]; then
    log_info "Running JavaScript tests (vitest)..."
    if vitest run; then
      log_success "JavaScript tests passed"
    else
      log_error "JavaScript tests failed"
    fi
  fi

  # [CUSTOMIZE] Go (go test)
  if command_exists go && ls -1 *_test.go >/dev/null 2>&1; then
    log_info "Running Go tests..."
    if go test ./...; then
      log_success "Go tests passed"
    else
      log_error "Go tests failed"
    fi
  fi
}

# ============================================================================
# GATE 5: Security & Secrets
# ============================================================================

gate5_security() {
  log_info "=== GATE 5: Security & Secrets ==="

  # [CUSTOMIZE] Secret detection (trufflehog)
  if command_exists trufflehog; then
    log_info "Running secret detection (trufflehog)..."
    if trufflehog git file://. --since-commit HEAD --only-verified --fail; then
      log_success "Secret detection passed"
    else
      log_error "Secret detection found issues"
    fi
  fi

  # [CUSTOMIZE] Dependency vulnerabilities (for Python)
  if command_exists pip-audit && [[ -f "requirements.txt" ]] || [[ -f "pyproject.toml" ]]; then
    log_info "Running dependency audit (pip-audit)..."
    if pip-audit; then
      log_success "Dependency audit passed"
    else
      log_warning "Dependency audit found issues (non-blocking)"
    fi
  fi

  # [CUSTOMIZE] Dependency vulnerabilities (for Rust)
  if command_exists cargo && [[ -f "Cargo.lock" ]]; then
    log_info "Running Rust vulnerability check..."
    if cargo audit; then
      log_success "Rust vulnerability check passed"
    else
      log_warning "Rust vulnerability check found issues (non-blocking)"
    fi
  fi
}

# ============================================================================
# GATE 6: Code Coverage
# ============================================================================

gate6_coverage() {
  log_info "=== GATE 6: Code Coverage ==="

  # [CUSTOMIZE] Python coverage
  if command_exists coverage && [[ -f "pyproject.toml" ]]; then
    log_info "Running Python coverage check..."
    coverage_min=80
    if coverage run -m pytest tests/ >/dev/null 2>&1; then
      coverage_pct=$(coverage report | grep TOTAL | awk '{print $NF}' | sed 's/%//')
      if (( $(echo "$coverage_pct >= $coverage_min" | bc -l) )); then
        log_success "Coverage check passed ($coverage_pct%)"
      else
        log_warning "Coverage below threshold: $coverage_pct% (min: $coverage_min%)"
      fi
    fi
  fi

  # [CUSTOMIZE] Add coverage checks for other languages
}

# ============================================================================
# GATE 7: Specification Traceability
# ============================================================================

gate7_traceability() {
  log_info "=== GATE 7: Specification Traceability ==="

  # [CUSTOMIZE] Check for FUNCTIONAL_REQUIREMENTS.md
  if [[ -f "FUNCTIONAL_REQUIREMENTS.md" ]]; then
    log_info "Found FUNCTIONAL_REQUIREMENTS.md"

    # Extract FR IDs from requirements file
    fr_count=$(grep -c "^## FR-" "FUNCTIONAL_REQUIREMENTS.md" || echo 0)
    log_info "Found $fr_count functional requirements"

    # [CUSTOMIZE] Verify all FRs have tests
    # This is a basic example; customize for your project structure
    if [[ $fr_count -gt 0 ]]; then
      log_success "Specification traceability check passed"
    fi
  else
    log_warning "No FUNCTIONAL_REQUIREMENTS.md found"
  fi
}

# ============================================================================
# GATE 8: Documentation
# ============================================================================

gate8_documentation() {
  log_info "=== GATE 8: Documentation ==="

  # [CUSTOMIZE] Markdown linting (vale)
  if command_exists vale && ls -1 *.md >/dev/null 2>&1; then
    log_info "Running Markdown linting (vale)..."
    if vale .; then
      log_success "Markdown linting passed"
    else
      log_warning "Markdown linting found issues (non-blocking)"
    fi
  fi

  # [CUSTOMIZE] Check for README
  if [[ ! -f "README.md" ]]; then
    log_warning "No README.md found"
  else
    log_success "README.md found"
  fi
}

# ============================================================================
# Summary & Exit
# ============================================================================

print_summary() {
  echo ""
  echo "=========================================="
  echo "Quality Gate Summary"
  echo "=========================================="

  if [[ ${#GATE_ERRORS[@]} -eq 0 ]]; then
    echo -e "${GREEN}Status: PASSED${NC}"
    echo "All quality gates passed successfully!"
  else
    echo -e "${RED}Status: FAILED${NC}"
    echo "The following gates failed:"
    printf '%s\n' "${GATE_ERRORS[@]}"
  fi

  if [[ ${#GATE_WARNINGS[@]} -gt 0 ]]; then
    echo ""
    echo "Warnings (non-blocking):"
    printf '%s\n' "${GATE_WARNINGS[@]}"
  fi

  echo "=========================================="
}

# ============================================================================
# Main Execution
# ============================================================================

main() {
  log_info "Starting quality gate checks..."
  log_info "Repository: $REPO_ROOT"

  # Run all gates
  gate1_syntax_validation
  gate2_linting
  gate3_type_checking
  gate4_testing
  gate5_security
  gate6_coverage
  gate7_traceability
  gate8_documentation

  # Print summary and exit
  print_summary

  if [[ "$GATE_PASSED" != "true" ]]; then
    exit 1
  fi

  exit 0
}

# Handle command-line arguments
if [[ "${1:-}" == "help" ]] || [[ "${1:-}" == "-h" ]] || [[ "${1:-}" == "--help" ]]; then
  cat <<EOF
Quality Gate Script

USAGE:
  $SCRIPT_NAME [OPTION]

OPTIONS:
  help, -h, --help      Show this help message
  gate1                 Run only syntax validation
  gate2                 Run only linting
  gate3                 Run only type checking
  gate4                 Run only tests
  gate5                 Run only security checks
  gate6                 Run only coverage checks
  gate7                 Run only traceability checks
  gate8                 Run only documentation checks
  all                   Run all gates (default)

EXAMPLES:
  $SCRIPT_NAME              # Run all gates
  $SCRIPT_NAME gate2        # Run linting only
  $SCRIPT_NAME gate5        # Run security checks

CUSTOMIZATION:
  This script is a template. Customize the gate functions for your project:
  1. Remove/add language-specific checks
  2. Adjust paths for your project structure
  3. Set coverage thresholds
  4. Add project-specific quality checks

See: thegent/templates/quality/README.md for more details.
EOF
  exit 0
fi

# Execute based on argument
if [[ "${1:-}" =~ ^gate[1-8]$ ]]; then
  ${1}
  print_summary
else
  main
fi
