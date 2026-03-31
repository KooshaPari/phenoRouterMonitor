# Local-First Tech Stack: Deployment & Operations Guide

---

## Part 1: Local Development Environment

### Quick Start (5 minutes)

```bash
# 1. Install Process Compose
brew install process-compose

# 2. Clone/navigate to repo
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus

# 3. Create data directories
mkdir -p data/{nats,dragonfly,neo4j,minio}

# 4. Start entire stack
process-compose -f process-compose.yml up

# 5. Verify health
curl http://localhost:8080/health
curl http://localhost:8080/ready
```

### Service Access Points

| Service | URL | Purpose |
|---------|-----|---------|
| NATS Server | nats://localhost:4222 | Message broker |
| NATS Monitoring | http://localhost:8222 | Server metrics/console |
| Dragonfly | redis://localhost:6379 | Cache layer |
| Neo4j Bolt | bolt://localhost:7687 | Graph database (protocol) |
| Neo4j Browser | http://localhost:7474 | Graph DB UI |
| MinIO S3 API | http://localhost:9000 | Object storage API |
| MinIO Console | http://localhost:9001 | S3 management UI |
| AgilePlus API | http://localhost:8080 | Application server |

### Typical Development Workflow

```bash
# Terminal 1: Start infrastructure
process-compose -f process-compose.yml up

# Terminal 2: Run API in watch mode
cd agileplus-api
cargo watch -x 'run'

# Terminal 3: Run tests
cd agileplus-api
cargo test -- --nocapture

# Terminal 4: Monitor logs
process-compose logs -f agileplus-api
```

### Stopping Services

```bash
# Graceful shutdown (all processes)
process-compose -f process-compose.yml down

# Stop specific service
process-compose -f process-compose.yml stop agileplus-api

# Kill all and cleanup
process-compose -f process-compose.yml down --remove-containers
```

---

## Part 2: Container Deployment (Docker)

### Building Container Images

Create `Dockerfile` for the Rust API:

```dockerfile
# Stage 1: Build
FROM rust:1.85 as builder

WORKDIR /workspace
COPY . .

RUN cargo build --release -p agileplus-api

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /workspace/target/release/agileplus-api /usr/local/bin/

EXPOSE 8080
ENV RUST_LOG=info,agileplus=debug
ENV RUST_BACKTRACE=1

HEALTHCHECK --interval=10s --timeout=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["agileplus-api"]
```

### Docker Compose (for containerized environment)

Create `docker-compose.yml`:

```yaml
version: '3.9'

services:
  nats:
    image: nats:latest
    command: nats-server -js -m 8222
    ports:
      - "4222:4222"
      - "8222:8222"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8222/varz"]
      interval: 10s
      timeout: 5s
      retries: 5
    volumes:
      - nats_data:/data/jetstream

  dragonfly:
    image: ghcr.io/dragonflydb/dragonfly:v1.3-alpine
    ports:
      - "6379:6379"
    depends_on:
      nats:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "redis-cli", "PING"]
      interval: 10s
      timeout: 5s
      retries: 5
    command: dragonfly --bind=0.0.0.0

  neo4j:
    image: neo4j:5.20-community-alpine
    ports:
      - "7687:7687"
      - "7474:7474"
    environment:
      NEO4J_AUTH: neo4j/password
      NEO4J_server_memory_heap_initial__size: 512m
      NEO4J_server_memory_heap_max__size: 1g
    depends_on:
      dragonfly:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:7474/browser/"]
      interval: 10s
      timeout: 5s
      retries: 5
    volumes:
      - neo4j_data:/var/lib/neo4j/data

  minio:
    image: minio/minio:latest
    ports:
      - "9000:9000"
      - "9001:9001"
    environment:
      MINIO_ROOT_USER: minioadmin
      MINIO_ROOT_PASSWORD: minioadmin
    command: server /data --console-address ":9001"
    depends_on:
      neo4j:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9000/minio/health/live"]
      interval: 10s
      timeout: 5s
      retries: 3
    volumes:
      - minio_data:/data

  agileplus-api:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "8080:8080"
    environment:
      NATS_URL: nats://nats:4222
      REDIS_URL: redis://dragonfly:6379
      NEO4J_URL: bolt://neo4j:7687
      NEO4J_USER: neo4j
      NEO4J_PASSWORD: password
      S3_ENDPOINT: http://minio:9000
      S3_ACCESS_KEY: minioadmin
      S3_SECRET_KEY: minioadmin
      S3_BUCKET: artifacts
      RUST_LOG: info,agileplus=debug
    depends_on:
      nats:
        condition: service_healthy
      dragonfly:
        condition: service_healthy
      neo4j:
        condition: service_healthy
      minio:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  nats_data:
  dragonfly_data:
  neo4j_data:
  minio_data:

networks:
  default:
    name: agileplus-network
```

### Launch with Docker Compose

```bash
# Build images
docker-compose build

# Start services
docker-compose up -d

# Check health
docker-compose ps
curl http://localhost:8080/health

# View logs
docker-compose logs -f agileplus-api

# Shutdown
docker-compose down -v
```

---

## Part 3: Kubernetes Deployment

### Namespace Setup

```bash
kubectl create namespace agileplus
kubectl config set-context --current --namespace=agileplus
```

### ConfigMap for Application Configuration

```yaml
# manifests/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: agileplus-config
  namespace: agileplus
data:
  NATS_URL: "nats://nats:4222"
  REDIS_URL: "redis://dragonfly:6379"
  NEO4J_URL: "bolt://neo4j:7687"
  NEO4J_USER: "neo4j"
  S3_ENDPOINT: "http://minio:9000"
  S3_BUCKET: "artifacts"
  RUST_LOG: "info,agileplus=debug"
```

### NATS StatefulSet

```yaml
# manifests/nats.yaml
apiVersion: v1
kind: Service
metadata:
  name: nats
  namespace: agileplus
spec:
  clusterIP: None
  selector:
    app: nats
  ports:
    - port: 4222
      name: client
    - port: 8222
      name: monitor

---
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: nats
  namespace: agileplus
spec:
  serviceName: nats
  replicas: 1
  selector:
    matchLabels:
      app: nats
  template:
    metadata:
      labels:
        app: nats
    spec:
      containers:
      - name: nats
        image: nats:latest
        command:
          - nats-server
          - -js
          - -m
          - "8222"
        ports:
          - containerPort: 4222
            name: client
          - containerPort: 8222
            name: monitor
        livenessProbe:
          httpGet:
            path: /varz
            port: 8222
          initialDelaySeconds: 10
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /varz
            port: 8222
          initialDelaySeconds: 5
          periodSeconds: 5
        volumeMounts:
          - name: jetstream
            mountPath: /data/jetstream
  volumeClaimTemplates:
  - metadata:
      name: jetstream
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 10Gi
```

### Dragonfly Deployment

```yaml
# manifests/dragonfly.yaml
apiVersion: v1
kind: Service
metadata:
  name: dragonfly
  namespace: agileplus
spec:
  selector:
    app: dragonfly
  ports:
    - port: 6379
      targetPort: 6379

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: dragonfly
  namespace: agileplus
spec:
  replicas: 1
  selector:
    matchLabels:
      app: dragonfly
  template:
    metadata:
      labels:
        app: dragonfly
    spec:
      containers:
      - name: dragonfly
        image: ghcr.io/dragonflydb/dragonfly:v1.3-alpine
        ports:
          - containerPort: 6379
        livenessProbe:
          exec:
            command:
              - redis-cli
              - PING
          initialDelaySeconds: 10
          periodSeconds: 10
        readinessProbe:
          exec:
            command:
              - redis-cli
              - PING
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        volumeMounts:
          - name: dragonfly-data
            mountPath: /data
      volumes:
        - name: dragonfly-data
          emptyDir: {}
```

### API Deployment

```yaml
# manifests/api.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: agileplus-api
  namespace: agileplus
spec:
  replicas: 2
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: agileplus-api
  template:
    metadata:
      labels:
        app: agileplus-api
    spec:
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
            - weight: 100
              podAffinityTerm:
                labelSelector:
                  matchExpressions:
                    - key: app
                      operator: In
                      values:
                        - agileplus-api
                topologyKey: kubernetes.io/hostname
      containers:
      - name: api
        image: agileplus-api:latest
        imagePullPolicy: Always
        ports:
          - containerPort: 8080
            name: http
        envFrom:
          - configMapRef:
              name: agileplus-config
        env:
          - name: NEO4J_PASSWORD
            valueFrom:
              secretKeyRef:
                name: agileplus-secrets
                key: neo4j-password
          - name: S3_SECRET_KEY
            valueFrom:
              secretKeyRef:
                name: agileplus-secrets
                key: s3-secret-key
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 2
          failureThreshold: 3
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"

---
apiVersion: v1
kind: Service
metadata:
  name: agileplus-api
  namespace: agileplus
spec:
  selector:
    app: agileplus-api
  type: LoadBalancer
  ports:
    - port: 80
      targetPort: 8080
      protocol: TCP
      name: http
```

### Deploy to Kubernetes

```bash
# Create namespace and secrets
kubectl create namespace agileplus
kubectl -n agileplus create secret generic agileplus-secrets \
  --from-literal=neo4j-password=password \
  --from-literal=s3-secret-key=minioadmin

# Apply manifests
kubectl apply -f manifests/configmap.yaml
kubectl apply -f manifests/nats.yaml
kubectl apply -f manifests/dragonfly.yaml
kubectl apply -f manifests/api.yaml

# Monitor rollout
kubectl rollout status deployment/agileplus-api -n agileplus

# View logs
kubectl logs -f deployment/agileplus-api -n agileplus

# Port forward for local testing
kubectl port-forward -n agileplus svc/agileplus-api 8080:80
```

---

## Part 4: CI/CD Integration

### GitHub Actions Workflow

Create `.github/workflows/test-and-deploy.yml`:

```yaml
name: Test and Deploy

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      nats:
        image: nats:latest
        options: >-
          --health-cmd "curl -f http://localhost:8222/varz"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 4222:4222
          - 8222:8222

      dragonfly:
        image: ghcr.io/dragonflydb/dragonfly:v1.3-alpine
        options: >-
          --health-cmd "redis-cli PING"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 6379:6379

      neo4j:
        image: neo4j:5.20-community-alpine
        env:
          NEO4J_AUTH: neo4j/password
        options: >-
          --health-cmd "curl -f http://localhost:7474/browser/"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 7687:7687
          - 7474:7474

      minio:
        image: minio/minio:latest
        env:
          MINIO_ROOT_USER: minioadmin
          MINIO_ROOT_PASSWORD: minioadmin
        options: >-
          --health-cmd "curl -f http://localhost:9000/minio/health/live"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 3
        ports:
          - 9000:9000
          - 9001:9001

    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: 1.85

      - uses: Swatinem/rust-cache@v2

      - name: Run tests
        env:
          NATS_URL: nats://localhost:4222
          REDIS_URL: redis://localhost:6379
          NEO4J_URL: bolt://localhost:7687
          NEO4J_USER: neo4j
          NEO4J_PASSWORD: password
          S3_ENDPOINT: http://localhost:9000
          S3_ACCESS_KEY: minioadmin
          S3_SECRET_KEY: minioadmin
        run: cargo test --all

      - name: Lint
        run: cargo clippy --all -- -D warnings

      - name: Format check
        run: cargo fmt --all -- --check

  build:
    needs: test
    runs-on: ubuntu-latest
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'

    steps:
      - uses: actions/checkout@v4

      - uses: docker/setup-buildx-action@v3

      - uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ghcr.io/${{ github.repository }}/agileplus-api:${{ github.sha }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

---

## Part 5: Monitoring & Observability

### Prometheus Monitoring

Create `manifests/prometheus.yaml`:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
  namespace: agileplus
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
    scrape_configs:
      - job_name: agileplus-api
        static_configs:
          - targets: ['agileplus-api:8080']
        metrics_path: /metrics

---
apiVersion: v1
kind: Service
metadata:
  name: prometheus
  namespace: agileplus
spec:
  selector:
    app: prometheus
  ports:
    - port: 9090

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: prometheus
  namespace: agileplus
spec:
  replicas: 1
  selector:
    matchLabels:
      app: prometheus
  template:
    metadata:
      labels:
        app: prometheus
    spec:
      containers:
      - name: prometheus
        image: prom/prometheus:latest
        ports:
          - containerPort: 9090
        volumeMounts:
          - name: config
            mountPath: /etc/prometheus
      volumes:
        - name: config
          configMap:
            name: prometheus-config
```

### Jaeger Tracing

```yaml
# manifests/jaeger.yaml
apiVersion: v1
kind: Service
metadata:
  name: jaeger
  namespace: agileplus
spec:
  selector:
    app: jaeger
  ports:
    - port: 16686
      name: ui
    - port: 4317
      name: otlp

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: jaeger
  namespace: agileplus
spec:
  replicas: 1
  selector:
    matchLabels:
      app: jaeger
  template:
    metadata:
      labels:
        app: jaeger
    spec:
      containers:
      - name: jaeger
        image: jaegertracing/all-in-one:latest
        ports:
          - containerPort: 16686
          - containerPort: 4317
```

---

## Part 6: Troubleshooting

### Common Issues and Solutions

#### 1. Service Connection Refused
```bash
# Check service is running
docker ps | grep nats
ps aux | grep nats-server

# Check port is open
lsof -i :4222

# Test connection
nc -zv localhost 4222
```

#### 2. Permission Denied on MinIO Data
```bash
# Fix ownership
sudo chown -R 1000:1000 ./data/minio
chmod -R 755 ./data/minio
```

#### 3. Neo4j Memory Error
```bash
# Increase heap size in environment
export NEO4J_server_memory_heap_max__size=2g
```

#### 4. Redis Connection Pool Exhausted
```rust
// Increase pool size in code
let pool = bb8::Pool::builder()
    .max_size(32)  // Increase from 16
    .build(manager)
    .await?;
```

#### 5. NATS JetStream not persisting
```bash
# Verify storage directory exists
mkdir -p data/jetstream
ls -la data/jetstream

# Check disk space
df -h
```

---

## Part 7: Performance Tuning

### Local Development

```bash
# Enable cargo incremental compilation
export CARGO_INCREMENTAL=1

# Use mold linker for faster builds (macOS/Linux)
brew install mold
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
```

### Production Recommendations

| Component | Setting | Rationale |
|-----------|---------|-----------|
| NATS | `max_connections: 50000` | Handle concurrent clients |
| Dragonfly | `--maxclients 50000` | Client limit |
| Neo4j | `dbms.memory.pagecache.size=2g` | Larger hot dataset |
| MinIO | Multi-node cluster | High availability |
| API Servers | 2-3 replicas | Load distribution |

### Database Connection Pooling

```rust
// Recommended settings
redis_pool = bb8::Pool::builder()
    .max_size(32)
    .min_idle(8)
    .connection_timeout(Duration::from_secs(30))
    .build(manager)
    .await?;

neo4j_pool = neo4rs::ConfigBuilder::new()
    .uri("bolt://...")
    .username("...")
    .password("...")
    .max_connections(32)
    .build()
    .await?;
```

---

## Maintenance & Updates

### Rolling Updates

```bash
# Update container image
kubectl set image deployment/agileplus-api \
  api=agileplus-api:v1.2.0 \
  -n agileplus

# Monitor rollout
kubectl rollout status deployment/agileplus-api -n agileplus

# Rollback if needed
kubectl rollout undo deployment/agileplus-api -n agileplus
```

### Database Migrations

```bash
# Neo4j schema updates
kubectl exec -it neo4j-0 -n agileplus -- cypher-shell \
  -u neo4j -p password \
  -f /scripts/migration.cypher
```

### Backup and Restore

```bash
# Backup MinIO
mc mirror minio/artifacts /local/backup/minio

# Backup Neo4j
kubectl exec -it neo4j-0 -n agileplus -- \
  bin/neo4j-admin dump --database=neo4j --to=/backups/neo4j.dump

# Restore from backup
kubectl exec -it neo4j-0 -n agileplus -- \
  bin/neo4j-admin load --from=/backups/neo4j.dump --database=neo4j --force
```

---

**Deployment Status**: Production-Ready | **Last Updated**: March 2026
