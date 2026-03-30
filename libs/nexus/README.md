# nexus

Service registry and discovery for microservices.

## Features

- **Registry**: Register/deregister services
- **Discovery**: Find services by name/tags
- **Health Checks**: Automatic health monitoring
- **Load Balancing**: Round-robin, random, consistent hash

## Installation

```toml
[dependencies]
nexus = { git = "https://github.com/KooshaPari/nexus" }
```

## Usage

```rust
use nexus::{Registry, Service, Endpoint};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = Registry::new();
    registry.register(Service::new("user-svc", Endpoint::new("localhost:8080"))).await?;

    let services = registry.discover("user-svc").await?;
    println!("Found {} services", services.len());
    Ok(())
}
```

### Health Monitoring

```rust
use nexus::{HealthMonitor, HealthCheckConfig};

let monitor = HealthMonitor::new();
monitor.register_service("user-svc".to_string()).await;
monitor.record_success("user-svc").await;
```

## License

MIT
