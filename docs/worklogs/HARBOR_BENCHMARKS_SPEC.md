# Harbor Benchmarks Library Specification

**Project:** portage, heliosCLI
**Status:** in_progress
**Type:** Library extraction

---

## Overview

Create a `harbor-benchmarks` library to consolidate benchmark runner patterns across portage and heliosCLI.

## Current State

### heliosCLI (harness/)
- `benchmark_runner.py` - Base benchmark interface
- `harness_benchmark.py` - Main harness benchmarks
- `llm_sla_benchmark.py` - LLM SLA measurements
- `unified_benchmark.py` - Unified benchmark runner
- `http_pool.py` - HTTP connection pooling (178 LOC)

### portage (heliosBench/)
- Benchmark definitions
- Metric collection interfaces
- Result aggregation utilities

## Duplicate Patterns

| Pattern | heliosCLI | portage | LOC |
|---------|-----------|---------|-----|
| HTTP pool wrapper | `http_pool.py` | - | 178 |
| Benchmark runners | Multiple files | heliosBench/ | 500+ |
| Metric types | Various | Various | 200+ |

## Target Architecture

```
harbor-benchmarks/
├── __init__.py
├── runner.py          # BenchmarkRunner base class
├── metrics.py         # Metric collection, aggregation
├── http_pool.py       # Consolidated HTTPConnectionPool
├── result.py          # BenchmarkResult dataclass
└── config.py          # BenchmarkConfig
```

## Key Components

```python
from dataclasses import dataclass
from typing import Iterator

@dataclass
class BenchmarkResult:
    """Standard benchmark result format."""
    name: str
    duration_ms: float
    throughput: float
    latency_p50: float
    latency_p95: float
    latency_p99: float
    error_rate: float
    
class BenchmarkRunner(ABC):
    """Base class for benchmark runners."""
    
    @abstractmethod
    def run(self, config: BenchmarkConfig) -> Iterator[BenchmarkResult]:
        """Run benchmark and yield results."""
        pass
    
    @abstractmethod
    async def cleanup(self) -> None:
        """Clean up benchmark resources."""
        pass
```

## HTTP Pool Consolidation

The existing `HTTPConnectionPool` in heliosCLI is well-designed:
- Singleton pattern
- HTTP/2 enabled
- Configurable pool parameters
- Sync and async support

This should become `harbor_benchmarks.http_pool`.

## Migration Path

1. **Phase 1:** Extract HTTPConnectionPool to shared lib (1 hour)
2. **Phase 2:** Create BenchmarkRunner base class (2 hours)
3. **Phase 3:** Migrate heliosCLI benchmarks to use shared lib (2 hours)
4. **Phase 4:** Migrate portage heliosBench (2 hours)

## LOC Savings

Estimated: 300-500 LOC reduction through consolidation.

---

_Last updated: 2026-04-03_