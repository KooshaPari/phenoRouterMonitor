# phenotype-router-monitor

Router monitoring and traffic analysis for phenotype infrastructure.

## Overview

Provides real-time monitoring of routing decisions, latency tracking, and traffic pattern analysis for the Phenotype ecosystem.

## Features

- Route latency tracking
- Traffic pattern analysis
- Health check monitoring
- Performance metrics collection

## Usage

```rust
use phenotype_router_monitor::RouterMonitor;

let monitor = RouterMonitor::new();
monitor.track_route("api", Duration::from_millis(12));
```

## License

MIT
