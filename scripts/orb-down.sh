#!/bin/sh
# Stop and remove OrbStack containers for Dragonfly and PostgreSQL.
set -e

DRAGONFLY_NAME="agileplus-dragonfly"
POSTGRES_NAME="agileplus-postgres"

for name in "${DRAGONFLY_NAME}" "${POSTGRES_NAME}"; do
  if orb list -a 2>/dev/null | grep -q "${name}"; then
    echo "Stopping and removing ${name}"
    orb stop "${name}" 2>/dev/null || true
    orb rm "${name}" 2>/dev/null || true
  else
    echo "${name} not found, skipping"
  fi
done

echo "OrbStack containers cleaned up."
