<<<<<<< HEAD
#!/bin/sh
# Stop and remove OrbStack containers for Dragonfly and PostgreSQL.
# Uses docker CLI (OrbStack v2).
set -e

DRAGONFLY_NAME="agileplus-dragonfly"
POSTGRES_NAME="agileplus-postgres"

for name in "${DRAGONFLY_NAME}" "${POSTGRES_NAME}"; do
    if docker ps -a --format '{{.Names}}' | grep -q "^${name}$"; then
        echo "Stopping and removing ${name}"
        docker stop "${name}" 2>/dev/null || true
        docker rm "${name}" 2>/dev/null || true
    else
        echo "${name} not found, skipping"
    fi
done

echo "OrbStack containers cleaned up."
=======
#!/bin/bash
# OrbStack container shutdown stubs
# Containers are managed via Docker, not OrbStack

echo "OrbStack shutdown (containers managed via Docker)"
>>>>>>> origin/main
