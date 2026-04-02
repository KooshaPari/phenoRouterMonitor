# agileplus-mcp Specification

## Architecture
```
┌─────────────────────────────────────────────────────────────────┐
│                   AgilePlus MCP                     │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────┐    ┌──────────┐    ┌──────────┐ │
│  │ FastMCP  │───▶│   gRPC  │───▶│  Rust   │ │
│  │ Server  │    │  Client │    │  Core   │ │
│  └──────────┘    └──────────┘    └──────────┘ │
│       │                                        │
│       ▼                                        │
│  ┌──────────────────────────────────────────┐ │
│  │        OTel SDK (tracing/metrics)         │ │
│  └──────────────────────────────────────────┘ │
└───────────────────────────────────────────────┘
```

## Components

| Component | Responsibility | Public API |
|-----------|----------------|-----------|
| Server | MCP protocol handler | `run()`, `serve()` |
| gRPCClient | Bridge to Rust core | `call()`, `stream()` |
| Tools | MCP tool implementations | `governance`, `features`, `status` |
| Sampling | Response sampling middleware | `sample()` |

## Data Models

```python
class ToolInput(BaseModel):
    scope: str
    action: str
    params: dict

class ToolOutput(BaseModel):
    success: bool
    data: Any
    error: str | None
```

## Performance Targets

| Metric | Target |
|--------|--------|
| Tool invocation | <500ms |
| gRPC round-trip | <200ms |
| Concurrent tools | 10 max |
| Memory usage | <100MB |