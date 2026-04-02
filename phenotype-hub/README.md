# phenotype-hub

> API gateway and service mesh for Phenotype platform

## Overview

`phenotype-hub` provides the API gateway and service mesh layer for the Phenotype platform:
- **API Gateway**: REST/gRPC proxy with intelligent routing
- **Service Mesh**: East-west traffic management
- **Authentication**: JWT, OAuth2, API keys, mTLS
- **Rate Limiting**: Per-client, per-endpoint limits
- **Circuit Breaking**: Per-service fault tolerance
- **Observability**: Metrics, traces, structured logging

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Hub Architecture                                    │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         Edge Proxy                                     │   │
│  │                                                                      │   │
│  │   ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐         │   │
│  │   │  Router  │ │  Auth    │ │ Rate Lim │ │  Logger  │         │   │
│  │   └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘         │   │
│  │        │             │             │             │                 │   │
│  │        └─────────────┼─────────────┼─────────────┘                 │   │
│  │                      │             │                                 │   │
│  └──────────────────────┼─────────────┼─────────────────────────────────┘   │
│                         │             │                                      │
│                         ▼             ▼                                      │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      Backend Services                                  │   │
│  │                                                                      │   │
│  │   ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐         │   │
│  │   │  Agent   │ │  Task   │ │  Skill   │ │   Env    │         │   │
│  │   │  Core    │ │  Engine │ │ Registry │ │ Manager  │         │   │
│  │   └──────────┘ └──────────┘ └──────────┘ └──────────┘         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Features

### Routing
- Path-based routing
- Header-based routing
- Weighted routing
- Canary deployments
- A/B testing

### Authentication
- JWT validation
- OAuth2 token exchange
- API key validation
- mTLS for service-to-service

### Rate Limiting
- Token bucket algorithm
- Sliding window
- Per-client limits
- Per-endpoint limits

### Circuit Breaking
- Per-service breakers
- Failure threshold detection
- Automatic recovery
- Fallback responses

## Quick Start

```yaml
# config.yaml
hub:
  port: 8080
  grpc_port: 9090

services:
  - name: agent-core
    url: http://localhost:8081
    health: /healthz
  - name: task-engine
    url: http://localhost:8082
    health: /healthz

auth:
  jwt:
    issuer: phenotype
    public_key_url: https://auth.phenotype.dev/.well-known/jwks.json

rate_limit:
  enabled: true
  default: 100/minute
```

```bash
# Run hub
hub serve --config config.yaml

# Register service
hub service register --name agent-core --url http://localhost:8081

# Check health
hub health
```

## API

### REST

```bash
# Proxy request to service
curl http://localhost:8080/api/v1/agent/123

# Health check
curl http://localhost:8080/healthz
```

### gRPC

```protobuf
service HubService {
    rpc Route(RouteRequest) returns (RouteResponse);
    rpc RegisterService(RegisterServiceRequest) returns (RegisterServiceResponse);
    rpc ListServices(ListServicesRequest) returns (ListServicesResponse);
}
```

## Documentation

- [Specification](SPEC.md) - Detailed system specification

## License

MIT License
