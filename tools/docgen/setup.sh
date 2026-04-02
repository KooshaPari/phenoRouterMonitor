#!/bin/bash
# Setup script for documentation generation tools
# This script installs all dependencies needed for automated doc generation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
VENV_DIR="$SCRIPT_DIR/venv"

echo "=== Setting up Documentation Generation Tools ==="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check Python version
check_python() {
    log_info "Checking Python version..."
    if command -v python3 &> /dev/null; then
        PYTHON_VERSION=$(python3 --version | cut -d' ' -f2)
        log_info "Found Python $PYTHON_VERSION"
    else
        log_error "Python 3 is required but not installed"
        exit 1
    fi
}

# Create virtual environment
setup_venv() {
    log_info "Setting up Python virtual environment..."
    if [ ! -d "$VENV_DIR" ]; then
        python3 -m venv "$VENV_DIR"
        log_info "Created virtual environment at $VENV_DIR"
    else
        log_info "Virtual environment already exists"
    fi
    
    source "$VENV_DIR/bin/activate"
    pip install --upgrade pip wheel
}

# Install Python dependencies
install_python_deps() {
    log_info "Installing Python dependencies..."
    
    pip install \
        pytest pytest-cov \
        pyyaml click rich \
        requests aiohttp \
        networkx matplotlib \
        2>&1 | grep -v "already satisfied" || true
    
    log_info "Python dependencies installed"
}

# Check and install system dependencies
install_system_deps() {
    log_info "Checking system dependencies..."
    
    # Check for asciinema (for GIF recording)
    if ! command -v asciinema &> /dev/null; then
        log_warn "asciinema not found. Install for E2E GIF recording:"
        if [[ "$OSTYPE" == "darwin"* ]]; then
            echo "  brew install asciinema"
        elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
            echo "  sudo apt install asciinema  # Debian/Ubuntu"
            echo "  sudo pacman -S asciinema    # Arch"
        fi
    else
        log_info "asciinema found"
    fi
    
    # Check for agg (asciinema GIF generator)
    if ! command -v agg &> /dev/null; then
        log_warn "agg not found. Install for GIF generation:"
        echo "  cargo install asciinema-agg"
        echo "  # Or download from: https://github.com/asciinema/agg/releases"
    else
        log_info "agg found"
    fi
    
    # Check for Node.js (for VitePress)
    if ! command -v node &> /dev/null; then
        log_warn "Node.js not found. Install for VitePress:"
        if [[ "$OSTYPE" == "darwin"* ]]; then
            echo "  brew install node"
        fi
    else
        NODE_VERSION=$(node --version | cut -d'v' -f2)
        log_info "Node.js $NODE_VERSION found"
    fi
    
    # Check for pnpm/npm
    if command -v pnpm &> /dev/null; then
        log_info "pnpm found"
        PKG_MANAGER="pnpm"
    elif command -v npm &> /dev/null; then
        log_info "npm found"
        PKG_MANAGER="npm"
    else
        log_warn "No package manager found. Install pnpm or npm."
        PKG_MANAGER=""
    fi
}

# Setup VitePress in phenodocs
setup_vitepress() {
    log_info "Setting up VitePress in phenodocs..."
    
    cd "$REPO_ROOT/phenodocs"
    
    if [ ! -d "node_modules" ]; then
        if [ -n "$PKG_MANAGER" ]; then
            if [ "$PKG_MANAGER" = "pnpm" ]; then
                pnpm install
            else
                npm install
            fi
            log_info "VitePress dependencies installed"
        else
            log_warn "Skipping VitePress setup (no package manager)"
        fi
    else
        log_info "VitePress dependencies already installed"
    fi
    
    cd "$REPO_ROOT"
}

# Create necessary directories
create_directories() {
    log_info "Creating necessary directories..."
    
    mkdir -p "$REPO_ROOT/.agileplus/specs"
    mkdir -p "$REPO_ROOT/.agileplus/worklogs"
    mkdir -p "$REPO_ROOT/docs/assets/gifs"
    
    log_info "Directories created"
}

# Run initial setup for all repos
setup_repos() {
    log_info "Setting up documentation for all phenotype repos..."
    
    REPOS=(
        "phenotype-agent-core"
        "phenotype-skills"
        "phenotype-task-engine"
        "phenotype-vessel"
        "phenotype-sentinel"
        "phenotype-router-monitor"
        "phenotype-governance"
        "phenotype-evaluation"
        "phenotype-hub"
        "thegent"
    )
    
    for repo in "${REPOS[@]}"; do
        if [ -d "$REPO_ROOT/$repo" ]; then
            log_info "Setting up $repo..."
            
            # Create docs directory structure
            mkdir -p "$REPO_ROOT/$repo/docs/.vitepress/theme"
            
            # Copy VitePress config template if not exists
            if [ ! -f "$REPO_ROOT/$repo/docs/.vitepress/config.mts" ]; then
                cp "$REPO_ROOT/phenodocs/.vitepress/config.mts" "$REPO_ROOT/$repo/docs/.vitepress/config.mts" 2>/dev/null || true
            fi
            
            # Copy custom CSS if not exists
            if [ ! -f "$REPO_ROOT/$repo/docs/.vitepress/theme/custom.css" ]; then
                cp "$REPO_ROOT/phenodocs/.vitepress/theme/custom.css" "$REPO_ROOT/$repo/docs/.vitepress/theme/custom.css" 2>/dev/null || true
            fi
            
            # Create journeys.md if not exists
            if [ ! -f "$REPO_ROOT/$repo/docs/journeys.md" ]; then
                cat > "$REPO_ROOT/$repo/docs/journeys.md" << 'EOF'
# User Journeys

This document contains E2E user journey demonstrations for $(basename "$REPO_ROOT/$repo").

## Onboarding Journey

::: info Overview
Complete setup from installation to first agent execution.
:::

<div class="journey-steps">

### Step 1: Install CLI

<div class="gif-demo">
  <img src="./assets/gifs/onboarding-step1.gif" alt="CLI Installation">
  <p><strong>Command:</strong> <code>cargo install phenotype-agent-core</code></p>
  <p><strong>Expected:</strong> Binary installed, available in PATH</p>
</div>

### Step 2: Configure Provider

<div class="gif-demo">
  <img src="./assets/gifs/onboarding-step2.gif" alt="Provider Configuration">
  <p><strong>Command:</strong> <code>agent-core config set-provider openai</code></p>
  <p><strong>Expected:</strong> API key configured, connection tested</p>
</div>

</div>

---

*Generated automatically by docgen pipeline*
EOF
            fi
        fi
    done
    
    log_info "All repos set up"
}

# Main setup function
main() {
    log_info "Starting documentation tool setup..."
    
    check_python
    setup_venv
    install_python_deps
    install_system_deps
    setup_vitepress
    create_directories
    setup_repos
    
    log_info "=== Setup Complete ==="
    echo ""
    echo "Next steps:"
    echo "  1. Run full documentation generation:"
    echo "     bash $SCRIPT_DIR/generate-docs.sh --mode full"
    echo ""
    echo "  2. Or run individual modes:"
    echo "     bash $SCRIPT_DIR/generate-docs.sh --mode specs"
    echo "     bash $SCRIPT_DIR/generate-docs.sh --mode annotations"
    echo "     bash $SCRIPT_DIR/generate-docs.sh --mode validate"
    echo ""
    echo "  3. View generated docs:"
    echo "     cd phenodocs && pnpm docs:dev"
}

# Run if executed directly
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    main "$@"
fi
