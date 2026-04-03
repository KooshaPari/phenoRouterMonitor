# Adopting phenotype-health

Health checking for Phenotype services.

## Quick Start

```rust
use phenotype_health::{HealthChecker, HealthStatus, HealthMonitor};

// Implement the HealthChecker trait
struct MyService {
    db: DatabasePool,
    cache: CachePool,
}

impl HealthChecker for MyService {
    fn health_check(&self) -> HealthStatus {
        if self.db.is_healthy() && self.cache.is_healthy() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Degraded(vec![
                "Database unhealthy".to_string(),
            ])
        }
    }
}

// Use the health monitor
let monitor = HealthMonitor::new();
monitor.register("my-service", my_service);
```

## Health Checkers

```rust
use phenotype_health::checkers::*;

// Redis health checker
let checker = RedisHealthChecker::new(redis_url);

// PostgreSQL health checker  
let checker = PostgresHealthChecker::new(pg_pool);

// HTTP health checker
let checker = HttpHealthChecker::new(endpoint);
```

## HTTP Endpoint

```rust
use phenotype_health::axum::health_routes;

let app = Router::new()
    .merge(health_routes());
```
