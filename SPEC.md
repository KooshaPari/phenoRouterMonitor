# phenotype-infrakit Specification

Canonical definition of the system behavior.

## Overview

Infrastructure toolkit providing common services for the Phenotype ecosystem.

## Architecture

### Dashboard Layer
- Web UI for visualization
- Server for data aggregation
- Real-time updates

### Core Services Layer
- Analytics collection
- Authentication
- Configuration management
- Health monitoring
- Observability

### Security Layer
- Compliance scanning
- Security aggregation
- Guard services

### Testing Layer
- BDD framework
- Contract testing

## Data Models

### Health Status
```rust
struct HealthStatus {
    service: String,
    status: Status,
    last_check: DateTime,
    latency_ms: u64,
}
```

### Security Event
```rust
struct SecurityEvent {
    severity: Severity,
    source: String,
    description: String,
    timestamp: DateTime,
}
```

## API Endpoints

### Dashboard
- `GET /dashboard` - Web UI
- `GET /api/metrics` - Metrics API
- `GET /api/health` - Health summary

### Health Service
- `GET /health/{service}` - Service health
- `GET /health/all` - All services health

### Compliance
- `GET /compliance/status` - Compliance status
- `POST /compliance/scan` - Trigger scan
