# Deep LOC Audit - Extended Opportunities

**Category:** DEEP_AUDIT | **Updated:** 2026-04-03

---

## Extended Patterns Found for Additional LOC Reduction

### 1. Command/Handler Pattern Duplication (~300 LOC)

**Pattern:** Multiple crates have similar command handler structures

```rust
// Duplicated across 5+ crates
pub trait CommandHandler<C> {
    async fn handle(&self, cmd: C) -> Result<(), Error>;
}

pub struct HandlerRegistry {
    handlers: HashMap<TypeId, Box<dyn Handler>>,
}
```

**Locations:**
- phenotype-contracts/inbound
- agileplus-domain/services
- thegent/commands
- Various microservices

**Savings:** ~200 LOC via shared handler trait

---

### 2. DTO/Response Pattern Duplication (~250 LOC)

**Pattern:** Similar DTOs across different services

```rust
// Duplicated in multiple services
pub struct ApiResponse<T> {
    pub data: T,
    pub status: String,
    pub timestamp: DateTime,
}

pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
}
```

**Locations:**
- AgentAPI++
- CliProxyAPI++
- Various phenotype services

**Savings:** ~250 LOC via shared response types

---

### 3. Configuration Builder Duplication (~200 LOC)

**Pattern:** Multiple config builder implementations

```rust
// Similar patterns in 8+ crates
pub struct ConfigBuilder {
    timeout: Option<Duration>,
    retries: Option<u32>,
    endpoint: Option<String>,
}

impl ConfigBuilder {
    pub fn with_timeout(mut self, t: Duration) -> Self { ... }
    pub fn with_retries(mut self, n: u32) -> Self { ... }
}
```

**Savings:** ~200 LOC via config builder macro

---

### 4. Validation Logic Duplication (~150 LOC)

**Pattern:** Custom validation functions repeated

```rust
// Duplicated validation functions
fn validate_email(email: &str) -> bool { ... }
fn validate_url(url: &str) -> bool { ... }
fn validate_id(id: &str) -> bool { ... }
```

**Locations:** Multiple Python and Rust services

**Savings:** ~150 LOC via shared validation library

---

### 5. Connection Pool Patterns (~100 LOC)

**Pattern:** Similar connection pool setup across services

```rust
// Duplicated pool configuration
pub struct PoolConfig {
    max_connections: u32,
    min_connections: u32,
    connection_timeout: Duration,
}
```

**Savings:** ~100 LOC via shared pool config

---

### 6. Metric Labels Duplication (~80 LOC)

**Pattern:** Similar metric labels across services

```rust
// Duplicate label definitions
const LABEL_SERVICE: &str = "service";
const LABEL_ENDPOINT: &str = "endpoint";
const LABEL_METHOD: &str = "method";
```

**Savings:** ~80 LOC via shared metric constants

---

### 7. Error Messages Duplication (~100 LOC)

**Pattern:** Same error messages in multiple services

```rust
// Duplicate messages
const ERR_NOT_FOUND: &str = "Resource not found";
const ERR_UNAUTHORIZED: &str = "Unauthorized access";
const ERR_TIMEOUT: &str = "Operation timed out";
```

**Savings:** ~100 LOC via shared error messages

---

### 8. HTTP Status Codes (~50 LOC)

**Pattern:** Custom HTTP status code constants

```rust
// Duplicate status codes
const STATUS_OK: u16 = 200;
const STATUS_CREATED: u16 = 201;
const STATUS_NOT_FOUND: u16 = 404;
```

**Savings:** ~50 LOC via stdlib or shared constants

---

### 9. Date/Time Formatting (~80 LOC)

**Pattern:** Similar date formatting across services

```rust
// Duplicate date formatters
fn format_timestamp(dt: &DateTime) -> String { ... }
fn parse_timestamp(s: &str) -> Option<DateTime> { ... }
```

**Savings:** ~80 LOC via shared datetime utilities

---

### 10. JSON Response Helpers (~120 LOC)

**Pattern:** Similar JSON response builders

```rust
// Duplicate JSON helpers
fn json_success<T: Serialize>(data: T) -> Response { ... }
fn json_error<E: Serialize>(error: E) -> Response { ... }
```

**Savings:** ~120 LOC via shared HTTP helpers

---

### 11. Logging Filters (~60 LOC)

**Pattern:** Similar log filtering logic

```rust
// Duplicate filter configuration
fn should_log(level: &LogLevel, module: &str) -> bool { ... }
```

**Savings:** ~60 LOC via shared logging config

---

### 12. Path/URL Manipulation (~70 LOC)

**Pattern:** Similar URL building utilities

```rust
// Duplicate URL utilities
fn build_url(base: &str, path: &str) -> String { ... }
fn parse_query(s: &str) -> HashMap<String, String> { ... }
```

**Savings:** ~70 LOC via shared URL library

---

## Summary: Extended Audit Total

| Pattern | LOC Savings |
|---------|-------------|
| Command/Handler Duplication | 200 |
| DTO/Response Duplication | 250 |
| Config Builder Duplication | 200 |
| Validation Logic | 150 |
| Connection Pool Patterns | 100 |
| Metric Labels | 80 |
| Error Messages | 100 |
| HTTP Status Codes | 50 |
| Date/Time Formatting | 80 |
| JSON Response Helpers | 120 |
| Logging Filters | 60 |
| Path/URL Manipulation | 70 |
| **Additional Total** | **~1,460 LOC** |

---

## Combined Total: ~8,300+ LOC Reduction

| Audit Area | LOC Target |
|------------|------------|
| Previous Target (Phases 1-5) | 6,856 |
| Extended Patterns (Above) | 1,460 |
| **TOTAL** | **~8,316 LOC** |

---

## Recommended Priorities

1. **Immediate (P0):** Remove nested duplicates, migrate logrus/viper
2. **High Priority (P1):** Create shared response types, handler traits
3. **Medium (P2):** Consolidate config builders, validation, utilities
4. **Ongoing:** Regular audits for pattern drift

---

_Last updated: 2026-04-03_