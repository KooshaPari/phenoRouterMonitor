# pheno-core

Phenotype Core: Foundation package for configuration, error handling, logging, and observability.

## Overview

`pheno-core` provides minimal but essential infrastructure that all other Phenotype packages depend on:

- **pheno_core.errors**: Unified error hierarchy with context support (ZenMCPError and subclasses)
- **pheno_core.config**: Configuration management with environment variables, file loading, and validation
- **pheno_core.logging**: Structured logging with multiple backends (console, JSON, file, syslog)
- **pheno_core.observability**: Abstract ports for logging, tracing, metrics, health checking, and alerting

## Installation

```bash
pip install pheno-core
# or with uv
uv pip install pheno-core
```

## Quick Start

### Configuration

```python
from pydantic import Field
from pheno_core.config import BaseConfig, from_env, from_file, ConfigLoader

class AppConfig(BaseConfig):
    app_name: str = Field(default="myapp")
    debug: bool = Field(default=False)
    port: int = Field(default=8000)

# Load from environment
config = from_env(AppConfig, env_prefix="APP_")

# Load from file
config = from_file(AppConfig, "config.json")

# Chain multiple sources
loader = ConfigLoader(AppConfig)
config = loader.from_file("config.json").from_env().build()
```

### Error Handling

```python
from pheno_core.errors import (
    ConfigurationError,
    ValidationError,
    ServiceUnavailableError,
    CircuitBreakerOpenError,
)

try:
    # Some operation
    pass
except ConfigurationError as e:
    print(f"Config error: {e.message}, code: {e.code}")
    print(f"Context: {e.context}")
```

### Logging

```python
from pheno_core.logging import (
    get_logger,
    setup_console_logging,
    setup_json_logging,
    setup_file_logging,
)

# Get a logger
logger = get_logger(__name__)
logger.info("Application started")

# Set up specialized logging
console_logger = setup_console_logging("myapp", level="DEBUG")
json_logger = setup_json_logging("myapp", level="INFO")
file_logger = setup_file_logging("myapp", "/var/log/myapp.log")

logger.debug("Debug message", user="alice", action="login")
logger.error("Error occurred", error_code="ERR_500")
```

### Observability Ports

```python
from abc import abstractmethod
from pheno_core.observability import Logger, Tracer, Meter, HealthChecker, HealthStatus

class MyLogger(Logger):
    def debug(self, message: str, **kwargs):
        print(f"DEBUG: {message}")

    def info(self, message: str, **kwargs):
        print(f"INFO: {message}")

    # ... implement other methods

class MyHealthChecker(HealthChecker):
    async def check_health(self) -> tuple[HealthStatus, str]:
        return HealthStatus.HEALTHY, "All systems operational"
```

## Error Hierarchy

```
ZenMCPError (base)
├── ConfigurationError
├── ValidationError
├── ServiceUnavailableError
├── CircuitBreakerOpenError
├── RetryableError
└── TimeoutError
```

All errors support:
- `message`: Human-readable error message
- `code`: Machine-readable error code
- `context`: Dictionary of additional context (e.g., which service failed, retry info)

## Configuration

BaseConfig extends Pydantic's BaseSettings with:
- Environment variable loading (case-insensitive)
- `.env` file support
- Type validation
- JSON/TOML/YAML file loading
- Fluent ConfigLoader API

## Logging

Available logging backends:
- Console (stdout with formatted text)
- JSON (structured logs via structlog)
- File (with rotation support)
- Syslog (for system logging)

All loggers support:
- Level-based filtering
- Contextual logging with kwargs
- Exception logging with tracebacks
- Custom formatting

## Observability Ports

Abstract interfaces for:

### Logger
Debug, info, warning, error, critical methods

### Tracer
Distributed tracing with spans and events

### Meter
Metrics collection (counters, histograms, gauges)

### HealthChecker
Async health checks returning status and message

### Alerter
Alert/notification delivery

### Registry
Service discovery and registration

## Testing

Run tests with:

```bash
pytest
```

With coverage:

```bash
pytest --cov=src/pheno_core --cov-report=term-missing
```

## Development

Install development dependencies:

```bash
pip install -e ".[dev]"
# or with uv
uv pip install -e ".[dev]"
```

Run linting:

```bash
ruff check src/ tests/
black --check src/ tests/
mypy src/
```

Format code:

```bash
ruff check --fix src/ tests/
black src/ tests/
```

## License

MIT

## Contributing

See CONTRIBUTING.md for guidelines.
