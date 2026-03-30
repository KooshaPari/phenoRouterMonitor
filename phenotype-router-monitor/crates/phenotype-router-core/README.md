# phenotype-router-core

Core routing primitives for HTTP request distribution and path matching.

## Modules

- **error**: Error types and result wrappers
- **matcher**: Path matching strategies (Exact, Wildcard, Regex)
- **balancer**: Load balancing strategies (RoundRobin, Random, LeastConnections)
- **route**: Route and Backend data structures
- **registry**: Priority-based route registry
- **metrics**: Routing and backend metrics tracking

## Key Types

### MatcherStrategy
Trait for path matching implementations:
- `ExactMatcher`: Case-sensitive exact matching
- `WildcardMatcher`: Glob patterns (`*` = any sequence, `?` = single char)
- `RegexMatcher`: Full regex support

### LoadBalancingStrategy
Trait for backend selection:
- `RoundRobin`: Sequential cycling through backends
- `Random`: Random selection with uniform distribution
- `LeastConnections`: Select backend with fewest active connections

### RouteRegistry
Manages routes with priority support:
- Register routes with configurations and backends
- Find matching routes by path
- Priority-based lookup (higher priority checked first)
- Thread-safe with RwLock

### Backend
Represents a backend server:
- URL and ID tracking
- Connection counting (atomic)
- Request/error metrics
- Health status management

## Tests

Over 40 tests covering:
- Exact, wildcard, and regex matching
- Round-robin, random, and least-connections balancing
- Route registration and lookup
- Priority ordering
- Thread safety
- Error handling

## Example

```rust
use phenotype_router_core::*;

// Create registry
let registry = RouteRegistry::new();

// Create route with wildcard matcher
let config = RouteConfig {
    id: "api".to_string(),
    name: "API".to_string(),
    matcher_type: "wildcard".to_string(),
    pattern: "/api/*".to_string(),
    balancer_type: "round-robin".to_string(),
    backends: vec!["b1".to_string()],
    priority: Some(10),
};

let backends = vec![
    Backend::new("b1".to_string(), "http://localhost:3000".to_string()),
];

registry.register_route(config, backends).unwrap();

// Find route and select backend
let route = registry.find_route("/api/users").unwrap();
let backend = route.backends()[0].clone();
println!("Route to: {}", backend.url());
```
