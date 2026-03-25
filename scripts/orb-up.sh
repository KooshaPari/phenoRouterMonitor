#!/bin/sh
# Start OrbStack containers for Dragonfly and PostgreSQL.
# This script is idempotent -- it reuses existing containers if they are
# already running, and starts stopped ones.
set -e

DRAGONFLY_NAME="agileplus-dragonfly"
POSTGRES_NAME="agileplus-postgres"

POSTGRES_USER="agileplus"
POSTGRES_PASSWORD="${PLANE_POSTGRES_PASSWORD:-agileplus-dev}"
POSTGRES_DB="plane"

start_container() {
  name="$1"
  shift
  # If already running, nothing to do
  if orb list 2>/dev/null | grep -q "${name}.*running"; then
    echo "${name} is already running"
    return 0
  fi
  # If exists but stopped, start it
  if orb list -a 2>/dev/null | grep -q "${name}"; then
    echo "Starting existing container ${name}"
    orb start "${name}"
    return 0
  fi
  # Create and run
  echo "Creating container ${name}"
  orb run "$@"
}

echo "--- Starting Dragonfly (Redis-compatible cache) ---"
start_container "${DRAGONFLY_NAME}" \
  -d --name "${DRAGONFLY_NAME}" \
  -p 6379:6379 \
  docker.dragonflydb.io/dragonflydb/dragonfly:latest \
  --maxmemory=4gb --bind 0.0.0.0

echo "--- Starting PostgreSQL 16 ---"
start_container "${POSTGRES_NAME}" \
  -d --name "${POSTGRES_NAME}" \
  -p 5432:5432 \
  -e "POSTGRES_USER=${POSTGRES_USER}" \
  -e "POSTGRES_PASSWORD=${POSTGRES_PASSWORD}" \
  -e "POSTGRES_DB=${POSTGRES_DB}" \
  postgres:16-alpine

# Wait for both to be healthy
echo "Waiting for containers to become ready..."
for i in 1 2 3 4 5 6 7 8 9 10; do
  pg_ok=false
  df_ok=false
  if redis-cli -h localhost -p 6379 ping 2>/dev/null | grep -q PONG; then
    df_ok=true
  fi
  if pg_isready -h localhost -p 5432 2>/dev/null; then
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
