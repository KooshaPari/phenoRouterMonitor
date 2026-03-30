<<<<<<< HEAD
#!/bin/sh
# Start OrbStack containers for Dragonfly and PostgreSQL.
# OrbStack v2 exposes docker CLI - this script uses docker commands.
# Idempotent -- reuses existing containers.
set -e

DRAGONFLY_NAME="agileplus-dragonfly"
POSTGRES_NAME="agileplus-postgres"

POSTGRES_USER="agileplus"
POSTGRES_PASSWORD="${PLANE_POSTGRES_PASSWORD:-agileplus-dev}"
POSTGRES_DB="plane"

# Function to cleanup port before starting
cleanup_port() {
    port="$1"
    # Kill any process using this port
    lsof -ti :"$port" 2>/dev/null | xargs kill -9 2>/dev/null || true
}

start_container() {
    name="$1"
    image="$2"
    port="$3"
    shift 3
    extra_args="$@"

    # If running, skip
    if docker ps --format '{{.Names}}' | grep -q "^${name}$"; then
        echo "${name} is already running"
        return 0
    fi

    # Cleanup port if something else is using it
    cleanup_port "$port"

    # If exists but stopped, start it
    if docker ps -a --format '{{.Names}}' | grep -q "^${name}$"; then
        echo "Starting existing container ${name}"
        docker start "${name}"
        return 0
    fi

    # Create and run
    echo "Creating container ${name}"
    docker run -d --name "${name}" -p "${port}:${port}" $extra_args "$image"
}

echo "--- Starting Dragonfly (Redis-compatible cache) ---"
start_container "${DRAGONFLY_NAME}" \
    "docker.dragonflydb.io/dragonflydb/dragonfly:latest" \
    6379 \
    --maxmemory=4gb --bind 0.0.0.0

echo "--- Starting PostgreSQL 16 ---"
start_container "${POSTGRES_NAME}" \
    "postgres:16-alpine" \
    5432 \
    -e "POSTGRES_USER=${POSTGRES_USER}" \
    -e "POSTGRES_PASSWORD=${POSTGRES_PASSWORD}" \
    -e "POSTGRES_DB=${POSTGRES_DB}"

# Wait for both to be healthy
echo "Waiting for containers to become ready..."
for i in 1 2 3 4 5 6 7 8 9 10; do
    pg_ok=false
    df_ok=false
    # Check dragonfly
    if redis-cli -h localhost -p 6379 ping 2>/dev/null | grep -q PONG; then
        df_ok=true
    fi
    # Check postgres - use lsof first then pg_isready
    if lsof -ti :5432 > /dev/null 2>&1 && pg_isready -h localhost -p 5432 2>/dev/null; then
        pg_ok=true
    fi
    if [ "$pg_ok" = true ] && [ "$df_ok" = true ]; then
        echo "All OrbStack containers are ready."
        exit 0
    fi
    echo "  Waiting... (${i}/10)"
    sleep 2
done

echo "ERROR: Containers did not become ready in time" >&2
exit 1
=======
#!/bin/bash
# OrbStack container management stubs
# Containers (Dragonfly, PostgreSQL) are managed directly via Docker

# Verify containers are running
if docker ps --format '{{.Names}}' | grep -q "^dragonfly$"; then
    echo "✓ Dragonfly container is running"
else
    echo "Starting Dragonfly..."
    docker run -d --name dragonfly -p 6379:6379 --ulimit memlock=-1 docker.dragonflydb.io/dragonflydb/dragonfly
fi

if docker ps --format '{{.Names}}' | grep -q "^postgres-agileplus$"; then
    echo "✓ PostgreSQL container is running"
else
    echo "Starting PostgreSQL..."
    docker run -d --name postgres-agileplus -p 5432:5432 -e POSTGRES_USER=agileplus -e POSTGRES_DB=plane -e POSTGRES_PASSWORD=agileplus-dev postgres:16-alpine
fi

echo "OrbStack containers ready"
>>>>>>> origin/main
