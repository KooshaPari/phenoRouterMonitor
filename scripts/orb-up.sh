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
