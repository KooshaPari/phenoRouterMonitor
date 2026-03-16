#!/usr/bin/env bash
set -euo pipefail

# Bootstrap Plane.so Community Edition for local AgilePlus development.
# This script clones and configures plane.so to run natively via process-compose.

PLANE_REF="${PLANE_REF:-v0.23-dev}"
PLANE_DIR=".agileplus/plane"
PLANE_WEB_DIR=".agileplus/plane-web"
POSTGRES_DIR=".agileplus/postgres-data"

echo "=== AgilePlus: Plane.so Local Setup ==="

# 1. Create data directories
mkdir -p .agileplus/logs "$POSTGRES_DIR"

# 2. Initialize PostgreSQL if needed
if [ ! -f "$POSTGRES_DIR/PG_VERSION" ]; then
    echo "Initializing PostgreSQL database..."
    initdb -D "$POSTGRES_DIR" --auth=trust --username=agileplus
    echo "PostgreSQL initialized."
    createdb -U agileplus plane
fi

# 3. Clone plane.so API (Django backend) if not present
if [ ! -d "$PLANE_DIR" ]; then
    echo "Cloning Plane.so API..."
    git clone --depth=1 --branch "$PLANE_REF" https://github.com/makeplane/plane.git "$PLANE_DIR"
    echo "Installing Python dependencies..."
    cd "$PLANE_DIR/apiserver"
    python3 -m venv .venv
    source .venv/bin/activate
    pip install -r requirements.txt
    cd -
else
    echo "Plane API already present at $PLANE_DIR"
fi

# 4. Clone/setup plane.so web frontend if not present
if [ ! -d "$PLANE_WEB_DIR" ]; then
    echo "Setting up Plane.so Web..."
    cp -r "$PLANE_DIR/web" "$PLANE_WEB_DIR"
    cd "$PLANE_WEB_DIR"
    if command -v bun &>/dev/null; then
        bun install
    elif command -v pnpm &>/dev/null; then
        pnpm install
    else
        npm install
    fi
    cd -
else
    echo "Plane Web already present at $PLANE_WEB_DIR"
fi

# 5. Create .env for plane API (idempotent)
ENV_FILE="$PLANE_DIR/apiserver/.env"
if [ ! -f "$ENV_FILE" ]; then
    GENERATED_SECRET_KEY=$(openssl rand -hex 32)
    cat > "$ENV_FILE" << ENVEOF
DATABASE_URL=postgresql://agileplus:agileplus-dev@localhost:5432/plane
REDIS_URL=redis://localhost:6379
SECRET_KEY=${GENERATED_SECRET_KEY}
WEB_URL=http://localhost:3100
CORS_ALLOWED_ORIGINS=http://localhost:3100,http://localhost:3000
DEBUG=1
ENVEOF
fi

echo ""
echo "=== Setup Complete ==="
echo "Start the dev stack with: process-compose up"
echo "Dashboard: http://localhost:3000/dashboard"
echo "Plane.so:  http://localhost:3100"
