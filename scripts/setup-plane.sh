#!/usr/bin/env bash
set -euo pipefail

# Bootstrap Plane.so Community Edition for local AgilePlus development.
# This script clones and configures plane.so to run natively via process-compose.

PLANE_REF="${PLANE_REF:-v1.2.3}"
PLANE_DIR=".agileplus/plane"
PLANE_WEB_DIR=".agileplus/plane-web"
POSTGRES_DIR=".agileplus/postgres-data"

echo "=== AgilePlus: Plane.so Local Setup ==="

# 1. Create data directories
mkdir -p .agileplus/logs "$POSTGRES_DIR"

# 2. Initialize PostgreSQL if needed
if [ ! -f "$POSTGRES_DIR/PG_VERSION" ]; then
    if pg_isready -h localhost -p 5432 &>/dev/null; then
        echo "A PostgreSQL server is already running on port 5432. Using existing server."
        # Try to create user and db on existing server
        psql -h localhost -p 5432 -d postgres -c "CREATE ROLE agileplus WITH LOGIN PASSWORD 'agileplus-dev' SUPERUSER;" 2>/dev/null || true
        psql -h localhost -p 5432 -d postgres -c "CREATE DATABASE plane OWNER agileplus;" 2>/dev/null || true
        # Create a dummy PG_VERSION to skip this block next time if needed, 
        # but better to just handle it gracefully.
        touch "$POSTGRES_DIR/PG_VERSION"
    else
        echo "Initializing PostgreSQL database..."
        initdb -D "$POSTGRES_DIR" --auth=trust --username=agileplus
        echo "PostgreSQL initialized."
        # We need to start it temporarily to create the database if it's not running
        pg_ctl -D "$POSTGRES_DIR" -l .agileplus/logs/postgres_init.log start
        sleep 2
        createdb -U agileplus plane || true
        pg_ctl -D "$POSTGRES_DIR" stop
    fi
fi

# 3. Clone plane.so API (Django backend) if not present
if [ ! -d "$PLANE_DIR" ]; then
    echo "Cloning Plane.so..."
    git clone --depth=1 --branch "$PLANE_REF" https://github.com/makeplane/plane.git "$PLANE_DIR"
    echo "Installing Python dependencies..."
    cd "$PLANE_DIR/apps/api"
    python3 -m venv .venv
    source .venv/bin/activate
    pip install -r requirements.txt
    cd -
else
    echo "Plane already present at $PLANE_DIR"
fi

# 4. Clone/setup plane.so web frontend if not present
if [ ! -d "$PLANE_DIR" ]; then
    # Cloned in step 3
    true
fi

echo "Installing Plane dependencies (monorepo)..."
cd "$PLANE_DIR"
if command -v bun &>/dev/null; then
    bun install
elif command -v pnpm &>/dev/null; then
    pnpm install
else
    echo "Error: bun or pnpm is required."
    exit 1
fi
cd -

# 5. Create .env for plane API (idempotent)
ENV_FILE="$PLANE_DIR/apps/api/.env"
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
