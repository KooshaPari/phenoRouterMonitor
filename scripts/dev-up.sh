#!/bin/sh
# AgilePlus Development Stack Startup Script
# Starts all services with proper port management:
# - Docker containers (OrbStack): Dragonfly (6379), PostgreSQL (5432)
# - Native services: NATS (4222), Plane API (8000), Plane Web (3000)

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJ_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJ_DIR"

AGILEPLUS_DIR="$PROJ_DIR/.agileplus"
LOG_DIR="$AGILEPLUS_DIR/logs"

# Ports that need cleanup
NATIVE_PORTS="4222 8000 3000"
DOCKER_PORTS="6379 5432"

# Function to kill process on port
kill_port() {
    port="$1"
    echo "  Checking port $port..."
    if lsof -ti :"$port" 2>/dev/null | xargs kill -9 2>/dev/null; then
        echo "    Killed process on port $port"
    else
        echo "    Port $port is free"
    fi
}

# Function to check if port is in use
check_port() {
    port="$1"
    if lsof -ti :"$port" 2>/dev/null > /dev/null; then
        return 0  # in use
    else
        return 1  # free
    fi
}

echo "=== AgilePlus Development Stack ==="

# 1. Stop any existing services first
echo ""
echo "--- Stopping existing services ---"
echo "Killing native services..."
for port in $NATIVE_PORTS; do
    kill_port "$port"
done

# Stop docker containers
echo "Stopping docker containers..."
if docker ps --format '{{.Names}}' | grep -q "agileplus-dragonfly\|agileplus-postgres"; then
    docker stop agileplus-dragonfly agileplus-postgres 2>/dev/null || true
fi

sleep 1

# 2. Start Docker containers (OrbStack)
echo ""
echo "--- Starting Docker containers (OrbStack) ---"
bash "$SCRIPT_DIR/orb-up.sh"

# 3. Start NATS Server
echo ""
echo "--- Starting NATS Server ---"
if check_port 4222; then
    echo "Port 4222 already in use, skipping NATS"
else
    nats-server --jetstream \
        --store_dir "$AGILEPLUS_DIR/nats-data" \
        --port 4222 \
        --http_port 8222 \
        > "$LOG_DIR/nats.log" 2>&1 &
    echo "NATS started on port 4222"
fi

# 4. Wait for database to be ready
echo ""
echo "--- Waiting for PostgreSQL ---"
for i in 1 2 3 4 5 6 7 8 9 10; do
    if lsof -ti :5432 > /dev/null 2>&1 && pg_isready -h localhost -p 5432 2>/dev/null; then
        echo "PostgreSQL is ready!"
        break
    fi
    echo "  Waiting for PostgreSQL... ($i/10)"
    sleep 1
done

# 5. Create database if needed
echo ""
echo "--- Ensuring database exists ---"
if ! psql -h localhost -p 5432 -U agileplus -d postgres -c "SELECT 1" > /dev/null 2>&1; then
    echo "Creating plane database..."
    psql -h localhost -p 5432 -U postgres -c "CREATE DATABASE plane OWNER agileplus;" 2>/dev/null || true
fi

# 6. Start Plane API
echo ""
echo "--- Starting Plane API ---"
if check_port 8000; then
    echo "Port 8000 already in use, skipping Plane API"
else
    cd "$AGILEPLUS_DIR/plane/apps/api"
    source .venv/bin/activate
    export REDIS_URL=redis://localhost:6379
    export DATABASE_URL=postgresql://agileplus:agileplus-dev@localhost:5432/plane
    export SECRET_KEY="agileplus-dev-secret-$(date +%s)"
    export DJANGO_SETTINGS_MODULE=plane.settings.local
    export PYTHONUNBUFFERED=1

    nohup python manage.py runserver 0.0.0.0:8000 \
        > "$LOG_DIR/plane-api.log" 2>&1 &

    API_PID=$!
    echo "Plane API started (PID: $API_PID)"

    # Wait for it to be ready
    for i in 1 2 3 4 5 6 7 8 9 10; do
        if curl -s http://localhost:8000/api/ > /dev/null 2>&1; then
            echo "Plane API is ready!"
            break
        fi
        echo "  Waiting for Plane API... ($i/10)"
        sleep 1
    done
fi

# 7. Start Plane Web
echo ""
echo "--- Starting Plane Web ---"
if check_port 3000; then
    echo "Port 3000 already in use, skipping Plane Web"
else
    cd "$AGILEPLUS_DIR/plane"
    nohup bun run web:dev --port 3000 \
        > "$LOG_DIR/plane-web.log" 2>&1 &

    WEB_PID=$!
    echo "Plane Web started (PID: $WEB_PID)"

    # Wait for it to be ready
    for i in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15; do
        if curl -s http://localhost:3000 > /dev/null 2>&1; then
            echo "Plane Web is ready!"
            break
        fi
        echo "  Waiting for Plane Web... ($i/15)"
        sleep 2
    done
fi

# 8. Summary
echo ""
echo "=== AgilePlus is running! ==="
echo ""
echo "Services:"
echo "  - PostgreSQL:  localhost:5432 (plane/plane-dev)"
echo "  - Dragonfly:  localhost:6379 (Redis-compatible)"
echo "  - NATS:       localhost:4222"
echo "  - Plane API:  http://localhost:8000"
echo "  - Plane Web:   http://localhost:3000"
echo ""
echo "Logs:"
echo "  - NATS:        $LOG_DIR/nats.log"
echo "  - Plane API:   $LOG_DIR/plane-api.log"
echo "  - Plane Web:   $LOG_DIR/plane-web.log"
echo ""
echo "To stop:  bash scripts/orb-down.sh && pkill -f 'nats-server\|manage.py runserver'"
