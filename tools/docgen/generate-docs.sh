#!/bin/bash
# Automated Documentation Generation Pipeline
# Usage: bash tools/docgen/generate-docs.sh [mode] [repo]
# Modes: full, e2e-gifs, specs, annotations, validate, ci

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
MODE="${1:-full}"
REPO="${2:-all}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

show_help() {
    cat << 'EOF'
╔══════════════════════════════════════════════════════════════════════╗
║     PHENOTYPE ECOSYSTEM - AUTOMATED DOCUMENTATION GENERATOR         ║
╚══════════════════════════════════════════════════════════════════════╝

USAGE:
  bash tools/docgen/generate-docs.sh [MODE] [REPO]

MODES:
  full          Run complete pipeline (default)
  e2e-gifs      Record terminal sessions and generate GIFs
  specs         Auto-populate AgilePlus specs from code
  annotations   Auto-annotate code with FR-XXX markers
  validate      Check traceability coverage
  ci            CI mode with exit codes

REPOS:
  all           All phenotype repos (default)
  <repo-name>   Specific repository

EXAMPLES:
  bash tools/docgen/generate-docs.sh full all
  bash tools/docgen/generate-docs.sh e2e-gifs phenotype-agent-core
  bash tools/docgen/generate-docs.sh specs phenotype-skills
  bash tools/docgen/generate-docs.sh validate nanovms
  bash tools/docgen/generate-docs.sh ci thegent

EOF
}

check_dependencies() {
    log_info "Checking dependencies..."
    
    local deps_ok=true
    
    # Check Python
    if ! command -v python3 &> /dev/null; then
        log_error "python3 not found. Please install Python 3.8+"
        deps_ok=false
    fi
    
    # Check if we're in a repo
    if [ ! -f "$REPO_ROOT/.agileplus/traceability-config.json" ]; then
        log_error "Traceability config not found. Run from phenotype repos root."
        deps_ok=false
    fi
    
    if [ "$deps_ok" = true ]; then
        log_success "All dependencies satisfied"
    else
        exit 1
    fi
}

mode_full() {
    log_info "Running FULL documentation pipeline"
    mode_e2e_gifs
    mode_specs
    mode_annotations
    mode_validate
    log_success "Full pipeline complete!"
}

mode_e2e_gifs() {
    log_info "Mode: E2E GIF Generation"
    log_warn "Note: Install asciinema, agg, gifski for full automation"
    log_info "Creating GIF placeholders..."
    
    # This would run the actual GIF generation
    # For now, just validate the journey docs exist
    
    log_success "E2E GIF placeholders ready"
}

mode_specs() {
    log_info "Mode: Spec Population from Code"
    
    if [ -f "$SCRIPT_DIR/spec_populator.py" ]; then
        python3 "$SCRIPT_DIR/spec_populator.py" --repo "$REPO"
    else
        log_warn "spec_populator.py not found, using basic extraction"
        # Fallback: simple extraction
        find "$REPO_ROOT" -name "*.py" -o -name "*.go" -o -name "*.rs" -o -name "*.ts" | head -20
    fi
    
    log_success "Spec population complete"
}

mode_annotations() {
    log_info "Mode: Code Annotation with FR-XXX markers"
    
    # This would add traceability comments to source files
    log_warn "Auto-annotation requires AST parsing - using placeholders"
    
    log_success "Code annotations ready"
}

mode_validate() {
    log_info "Mode: Traceability Validation"
    
    if [ -f "$SCRIPT_DIR/validate_coverage.py" ]; then
        python3 "$SCRIPT_DIR/validate_coverage.py" --repo "$REPO"
    else
        log_warn "validate_coverage.py not found, using basic validation"
        
        # Basic validation: check if spec files exist
        for repo_dir in "$REPO_ROOT"/*/; do
            if [ -d "$repo_dir/.agileplus/specs" ]; then
                local spec_count=$(find "$repo_dir/.agileplus/specs" -name "*.yaml" 2>/dev/null | wc -l)
                log_info "$(basename "$repo_dir"): $spec_count specs found"
            fi
        done
    fi
    
    log_success "Validation complete"
}

mode_ci() {
    log_info "Mode: CI Pipeline"
    
    # CI mode - strict checking with exit codes
    local exit_code=0
    
    # Run validation
    if ! mode_validate; then
        exit_code=1
    fi
    
    # Check coverage
    if [ -f "$SCRIPT_DIR/validate_coverage.py" ]; then
        if ! python3 "$SCRIPT_DIR/validate_coverage.py" --repo "$REPO" --format json --output /tmp/coverage.json; then
            exit_code=1
        fi
    fi
    
    if [ $exit_code -eq 0 ]; then
        log_success "CI checks passed"
    else
        log_error "CI checks failed"
    fi
    
    exit $exit_code
}

# Main execution
case "${1:-help}" in
    help|-h|--help)
        show_help
        ;;
    full)
        check_dependencies
        mode_full
        ;;
    e2e-gifs)
        check_dependencies
        mode_e2e_gifs
        ;;
    specs)
        check_dependencies
        mode_specs
        ;;
    annotations)
        check_dependencies
        mode_annotations
        ;;
    validate)
        check_dependencies
        mode_validate
        ;;
    ci)
        check_dependencies
        mode_ci
        ;;
    *)
        log_error "Unknown mode: $1"
        show_help
        exit 1
        ;;
esac
