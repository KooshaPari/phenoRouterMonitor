# Multi-stage Dockerfile for all languages
# Optimized for Bun, Go, Rust, Python

# =============================================================================
# STAGE 1: Dependencies
# =============================================================================
FROM oven/bun:1 AS deps
WORKDIR /app
COPY package.json bun.lockb* ./
RUN bun install --frozen-lockfile

# =============================================================================
# STAGE 2: Build
# =============================================================================
FROM oven/bun:1 AS builder
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY . .
RUN bun run build

# =============================================================================
# STAGE 3: Go Builder
# =============================================================================
FROM golang:1.22-alpine AS go-builder
WORKDIR /build
COPY go.mod go.sum* ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -a -installsuffix cgo -o service .

# =============================================================================
# STAGE 4: Rust Builder
# =============================================================================
FROM rust:1.77-alpine AS rust-builder
WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
RUN cargo fetch
COPY . .
RUN cargo build --release

# =============================================================================
# STAGE 5: Python Builder
# =============================================================================
FROM python:3.12-slim AS python-deps
WORKDIR /app
COPY requirements.txt* pyproject.toml* ./
RUN pip install --no-cache-dir -r requirements.txt || true

# =============================================================================
# STAGE 6: Runtime (Bun)
# =============================================================================
FROM oven/bun:1-alpine AS runtime
WORKDIR /app
RUN addgroup -g 1001 -S nodejs && adduser -S nodejs -u 1001
COPY --from=builder --chown=nodejs:nodejs /app/dist ./dist
USER nodejs
EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1
ENTRYPOINT ["bun", "dist/index.js"]
