# Retrospective Service Trait Definitions

**Document Type:** Service Contract Specification
**Status:** Design Phase | **Implementation Target:** 200 LOC traits + 300 LOC impl
**Architecture:** Hexagonal + CQRS | **Test Coverage Target:** 80%+

---

## 1. Inbound Port: RetrospectiveService

The main domain service interface that orchestrates retrospective operations.

```rust
// File: crates/phenotype-contracts/src/ports/inbound/retrospective.rs

use crate::models::{Retrospective, AggregateResult, TimeRange, ExportFormat};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Primary inbound port for retrospective operations.
/// Defines the contract for all retrospective use cases.
///
/// # Architecture
/// This trait defines the domain boundary. All command handlers depend on
/// this abstraction, not on database implementations.
///
/// # Example
/// ```ignore
/// let service = Arc::new(RetrospectiveServiceImpl::new(repo, cache, event_bus));
/// let retrospective = service.generate(range, config).await?;
/// ```
#[async_trait]
pub trait RetrospectiveService: Send + Sync {
    /// Generate a retrospective for a specified time range.
    ///
    /// # Arguments
    /// * `range` - Time period to analyze
    /// * `config` - Generation configuration (metrics, trends, insights)
    ///
    /// # Returns
    /// A new `Retrospective` with populated metrics, trends, and insights.
    ///
    /// # Errors
    /// - `InvalidRange` - If time range is invalid
    /// - `Repository` - If persistence fails
    /// - `AggregationFailed` - If data collection fails
    async fn generate(
        &self,
        range: TimeRange,
        config: RetrospectiveConfig,
    ) -> Result<Retrospective, RetrospectiveError>;

    /// Compute aggregated metrics for an existing retrospective.
    ///
    /// Aggregation is expensive (full table scan). Results are cached for 10 minutes.
    ///
    /// # Arguments
    /// * `retrospective_id` - ID of retrospective to analyze
    ///
    /// # Returns
    /// Aggregated metrics (avg, min, max, total count)
    ///
    /// # Performance
    /// - First call: ~500ms (full scan)
    /// - Cached calls: ~1ms (Redis)
    async fn compute_aggregates(
        &self,
        retrospective_id: Uuid,
    ) -> Result<AggregateResult, RetrospectiveError>;

    /// Export retrospective in specified format.
    ///
    /// # Arguments
    /// * `retrospective_id` - ID of retrospective to export
    /// * `format` - Output format (JSON, Markdown, CSV, PDF)
    ///
    /// # Returns
    /// Raw bytes of exported document
    async fn export(
        &self,
        retrospective_id: Uuid,
        format: ExportFormat,
    ) -> Result<Bytes, RetrospectiveError>;

    /// Retrieve metadata for a retrospective.
    ///
    /// Lightweight operation (no full data load).
    /// Useful for listings and quick info displays.
    ///
    /// # Arguments
    /// * `retrospective_id` - ID of retrospective
    ///
    /// # Returns
    /// Metadata (id, created_at, author, time range)
    async fn get_metadata(
        &self,
        retrospective_id: Uuid,
    ) -> Result<RetrospectiveMetadata, RetrospectiveError>;

    /// List retrospectives in a time range.
    ///
    /// # Arguments
    /// * `from` - Start date (inclusive)
    /// * `to` - End date (inclusive)
    /// * `limit` - Maximum results (default 50, max 1000)
    /// * `offset` - Pagination offset
    ///
    /// # Returns
    /// Paginated list of retrospective metadata
    async fn list_retrospectives(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<RetrospectiveMetadata>, RetrospectiveError>;

    /// Delete a retrospective.
    ///
    /// # Arguments
    /// * `retrospective_id` - ID to delete
    /// * `cascade` - If true, delete related data (exports, cache entries)
    ///
    /// # Errors
    /// - `NotFound` - If ID doesn't exist
    /// - `Repository` - If delete fails
    async fn delete(
        &self,
        retrospective_id: Uuid,
        cascade: bool,
    ) -> Result<(), RetrospectiveError>;
}

/// Configuration for retrospective generation.
///
/// Controls which data sections are included in the retrospective.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetrospectiveConfig {
    /// Include detailed metrics
    pub include_metrics: bool,

    /// Include trend analysis
    pub include_trends: bool,

    /// Include AI-generated insights
    pub include_insights: bool,

    /// Date format string (e.g., "YYYY-MM-DD")
    pub date_format: String,

    /// Timezone (e.g., "UTC", "America/New_York")
    pub timezone: String,

    /// Optional custom title
    pub title: Option<String>,

    /// Optional filter tags
    pub tags: Vec<String>,

    /// Include archived items
    #[serde(default)]
    pub include_archived: bool,
}

impl Default for RetrospectiveConfig {
    fn default() -> Self {
        Self {
            include_metrics: true,
            include_trends: true,
            include_insights: true,
            date_format: "YYYY-MM-DD".to_string(),
            timezone: "UTC".to_string(),
            title: None,
            tags: vec![],
            include_archived: false,
        }
    }
}

/// Export format options.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    /// JSON (machine-readable)
    Json,

    /// Markdown (human-readable)
    Markdown,

    /// CSV (spreadsheet)
    Csv,

    /// PDF (printable)
    Pdf,
}

impl fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Markdown => write!(f, "markdown"),
            Self::Csv => write!(f, "csv"),
            Self::Pdf => write!(f, "pdf"),
        }
    }
}

impl ExportFormat {
    pub fn from_str(s: &str) -> Result<Self, RetrospectiveError> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "markdown" | "md" => Ok(Self::Markdown),
            "csv" => Ok(Self::Csv),
            "pdf" => Ok(Self::Pdf),
            _ => Err(RetrospectiveError::InvalidFormat(s.to_string())),
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Json => "application/json",
            Self::Markdown => "text/markdown",
            Self::Csv => "text/csv",
            Self::Pdf => "application/pdf",
        }
    }

    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Markdown => "md",
            Self::Csv => "csv",
            Self::Pdf => "pdf",
        }
    }
}

/// Error types for retrospective operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "error_type", content = "details")]
pub enum RetrospectiveError {
    /// Retrospective not found
    #[error("Retrospective not found: {0}")]
    NotFound(Uuid),

    /// Invalid time range (e.g., end before start)
    #[error("Invalid time range: {0}")]
    InvalidRange(String),

    /// Aggregation computation failed
    #[error("Aggregation failed: {0}")]
    AggregationFailed(String),

    /// Export generation failed
    #[error("Export failed: {0}")]
    ExportFailed(String),

    /// Invalid export format
    #[error("Invalid export format: {0}")]
    InvalidFormat(String),

    /// Repository/persistence error
    #[error("Repository error: {0}")]
    Repository(String),

    /// Cache operation error
    #[error("Cache error: {0}")]
    Cache(String),

    /// Event bus publication error
    #[error("Event bus error: {0}")]
    EventBus(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl RetrospectiveError {
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Cache(_) | Self::EventBus(_) | Self::AggregationFailed(_)
        )
    }

    pub fn http_status_code(&self) -> u16 {
        match self {
            Self::NotFound(_) => 404,
            Self::InvalidRange(_) | Self::InvalidFormat(_) | Self::Validation(_) => 400,
            Self::Cache(_) | Self::EventBus(_) => 503,
            Self::Repository(_) | Self::Internal(_) => 500,
            _ => 400,
        }
    }
}
```

---

## 2. Outbound Port: RetrospectiveRepository

Persistence abstraction for retrospective data.

```rust
// File: crates/phenotype-contracts/src/ports/outbound/repository.rs

use crate::models::Retrospective;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use thiserror::Error;

/// Persistence port for retrospective aggregates.
///
/// Adapters:
/// - `phenotype-sqlite-adapter` (SQLite impl)
/// - `phenotype-postgres-adapter` (PostgreSQL impl)
/// - `phenotype-mock-adapter` (Test mocks)
#[async_trait]
pub trait RetrospectiveRepository: Send + Sync {
    /// Store a new retrospective aggregate.
    ///
    /// # Arguments
    /// * `retrospective` - Domain aggregate to persist
    ///
    /// # Returns
    /// UUID of created retrospective
    ///
    /// # Errors
    /// - Database connectivity errors
    /// - Constraint violations
    async fn create(&self, retrospective: Retrospective) -> Result<Uuid, RepositoryError>;

    /// Retrieve a retrospective by ID.
    ///
    /// # Arguments
    /// * `id` - Retrospective ID
    ///
    /// # Returns
    /// `Some(Retrospective)` if found, `None` if not found
    ///
    /// # Errors
    /// - Database errors (not 404s)
    async fn get(&self, id: Uuid) -> Result<Option<Retrospective>, RepositoryError>;

    /// Query retrospectives by date range.
    ///
    /// # Arguments
    /// * `from` - Start date (inclusive)
    /// * `to` - End date (inclusive)
    ///
    /// # Returns
    /// All retrospectives created within range (chronological order)
    ///
    /// # Performance
    /// Indexed query. Expected <100ms for typical datasets.
    async fn list_by_range(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Retrospective>, RepositoryError>;

    /// Update an existing retrospective.
    ///
    /// # Arguments
    /// * `retrospective` - Updated aggregate (ID must exist)
    ///
    /// # Returns
    /// Updated at timestamp
    ///
    /// # Errors
    /// - `NotFound` - If ID doesn't exist
    /// - Database errors
    async fn update(&self, retrospective: Retrospective) -> Result<DateTime<Utc>, RepositoryError>;

    /// Delete a retrospective.
    ///
    /// # Arguments
    /// * `id` - Retrospective ID
    ///
    /// # Errors
    /// - `NotFound` - If ID doesn't exist
    /// - Database errors
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;

    /// Count retrospectives matching criteria.
    ///
    /// Used for pagination metadata.
    async fn count_by_range(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<usize, RepositoryError>;
}

#[derive(Error, Debug, Clone)]
pub enum RepositoryError {
    #[error("Record not found")]
    NotFound,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Constraint violated: {0}")]
    ConstraintViolation(String),
}
```

---

## 3. Outbound Port: RetrospectiveCache

Caching layer for expensive computations.

```rust
// File: crates/phenotype-contracts/src/ports/outbound/cache.rs

use crate::models::AggregateResult;
use async_trait::async_trait;
use std::time::Duration;
use uuid::Uuid;
use thiserror::Error;

/// Caching port for aggregation results.
///
/// Adapters:
/// - `phenotype-redis-adapter` (Redis impl)
/// - `phenotype-in-memory-adapter` (Local cache)
/// - `phenotype-mock-adapter` (Test mocks)
#[async_trait]
pub trait RetrospectiveCache: Send + Sync {
    /// Cache aggregation results.
    ///
    /// # Arguments
    /// * `key` - Cache key (e.g., "retrospective:aggregates:{id}")
    /// * `aggregates` - Computed aggregates
    /// * `ttl` - Time to live (10 min typical)
    ///
    /// # Behavior
    /// - Overwrites existing value
    /// - Returns Ok() even if cache is unavailable (fail-open)
    ///
    /// # Performance
    /// Expected <5ms for in-memory, <50ms for Redis
    async fn cache_aggregates(
        &self,
        key: &str,
        aggregates: &AggregateResult,
        ttl: Duration,
    ) -> Result<(), CacheError>;

    /// Retrieve cached aggregates.
    ///
    /// # Arguments
    /// * `key` - Cache key
    ///
    /// # Returns
    /// `Some(aggregates)` if key exists and valid, `None` if expired or missing
    ///
    /// # Performance
    /// Expected <5ms
    async fn get_aggregates(&self, key: &str) -> Result<Option<AggregateResult>, CacheError>;

    /// Invalidate cache entry.
    ///
    /// Called when retrospective is modified.
    ///
    /// # Arguments
    /// * `id` - Retrospective ID (derives cache keys)
    ///
    /// # Behavior
    /// - No-op if key doesn't exist
    /// - Returns Ok() even if cache is unavailable
    async fn invalidate(&self, id: Uuid) -> Result<(), CacheError>;

    /// Clear all retrospective cache entries.
    ///
    /// Useful for testing, admin operations, cache resets.
    async fn clear_all(&self) -> Result<(), CacheError>;

    /// Get cache statistics.
    ///
    /// Useful for observability.
    async fn stats(&self) -> Result<CacheStats, CacheError>;
}

#[derive(Clone, Debug)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub size_bytes: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum CacheError {
    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Timeout")]
    Timeout,

    #[error("Internal error: {0}")]
    Internal(String),
}
```

---

## 4. Outbound Port: RetrospectiveEventBus

Event publication for audit trail and event sourcing.

```rust
// File: crates/phenotype-contracts/src/ports/outbound/event_bus.rs

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use thiserror::Error;

/// Event publication port for audit trail and integration.
///
/// All state-changing operations publish events for:
/// - Audit logging
/// - Event sourcing (read model updates)
/// - Integration with other services (webhooks, message queues)
///
/// Adapters:
/// - `phenotype-sqlite-eventstore` (Event append-only log in SQLite)
/// - `phenotype-kafka-adapter` (Kafka topic publication)
/// - `phenotype-redis-pubsub-adapter` (Redis Pub/Sub)
/// - `phenotype-mock-adapter` (Test mocks)
#[async_trait]
pub trait RetrospectiveEventBus: Send + Sync {
    /// Publish a retrospective event.
    ///
    /// # Arguments
    /// * `event` - Domain event
    ///
    /// # Behavior
    /// - Persists to event store
    /// - Publishes to subscribers (message queues, webhooks)
    /// - Returns error if unable to persist (fail-closed for events)
    /// - Retries automatically (exponential backoff)
    ///
    /// # Performance
    /// Expected <10ms
    async fn publish(&self, event: RetrospectiveEvent) -> Result<(), EventBusError>;

    /// Subscribe to events.
    ///
    /// Returns a stream of events. Useful for event sourcing projections.
    async fn subscribe(
        &self,
        from: EventFilter,
    ) -> Result<Box<dyn EventStream>, EventBusError>;
}

/// Retrospective domain events.
///
/// These events form the event sourcing history.
/// All events are immutable and contain full context.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "data")]
pub enum RetrospectiveEvent {
    /// Retrospective was generated
    Generated {
        id: Uuid,
        range: TimeRange,
        config: RetrospectiveConfig,
        timestamp: DateTime<Utc>,
        user_id: Option<Uuid>,
    },

    /// Retrospective was exported
    Exported {
        id: Uuid,
        format: ExportFormat,
        timestamp: DateTime<Utc>,
        file_size: usize,
        user_id: Option<Uuid>,
    },

    /// Aggregates were computed
    Aggregated {
        id: Uuid,
        metrics_count: usize,
        aggregate_result: AggregateResult,
        timestamp: DateTime<Utc>,
    },

    /// Retrospective was deleted
    Deleted {
        id: Uuid,
        timestamp: DateTime<Utc>,
        user_id: Option<Uuid>,
        reason: Option<String>,
    },

    /// Metrics were updated
    MetricsUpdated {
        id: Uuid,
        added_count: usize,
        removed_count: usize,
        timestamp: DateTime<Utc>,
    },
}

impl RetrospectiveEvent {
    pub fn aggregate_id(&self) -> Uuid {
        match self {
            Self::Generated { id, .. }
            | Self::Exported { id, .. }
            | Self::Aggregated { id, .. }
            | Self::Deleted { id, .. }
            | Self::MetricsUpdated { id, .. } => *id,
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::Generated { timestamp, .. }
            | Self::Exported { timestamp, .. }
            | Self::Aggregated { timestamp, .. }
            | Self::Deleted { timestamp, .. }
            | Self::MetricsUpdated { timestamp, .. } => *timestamp,
        }
    }

    pub fn event_type(&self) -> &'static str {
        match self {
            Self::Generated { .. } => "retrospective.generated",
            Self::Exported { .. } => "retrospective.exported",
            Self::Aggregated { .. } => "retrospective.aggregated",
            Self::Deleted { .. } => "retrospective.deleted",
            Self::MetricsUpdated { .. } => "retrospective.metrics_updated",
        }
    }
}

/// Filter for event subscriptions.
#[derive(Clone, Debug)]
pub struct EventFilter {
    pub aggregate_id: Option<Uuid>,
    pub event_types: Vec<String>,
    pub from_timestamp: Option<DateTime<Utc>>,
    pub to_timestamp: Option<DateTime<Utc>>,
}

/// Async event stream for subscriptions.
#[async_trait]
pub trait EventStream: Send {
    async fn next_event(&mut self) -> Option<RetrospectiveEvent>;
}

#[derive(Error, Debug, Clone)]
pub enum EventBusError {
    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Publication failed: {0}")]
    PublicationFailed(String),

    #[error("Subscription failed: {0}")]
    SubscriptionFailed(String),

    #[error("Timeout")]
    Timeout,

    #[error("Internal error: {0}")]
    Internal(String),
}
```

---

## 5. Domain Models (Core Entities)

```rust
// File: crates/phenotype-contracts/src/models/retrospective.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Root aggregate for retrospective analysis.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Retrospective {
    pub id: Uuid,

    pub range: TimeRange,

    pub metrics: Vec<Metric>,

    pub trends: Vec<Trend>,

    pub insights: Vec<Insight>,

    pub created_at: DateTime<Utc>,

    pub updated_at: DateTime<Utc>,

    pub author: Option<String>,

    pub tags: Vec<String>,
}

/// Time range for analysis.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TimeRange {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

impl TimeRange {
    pub fn parse(from_str: &str, to_str: &str) -> Result<Self, String> {
        let from = DateTime::parse_from_rfc3339(from_str)
            .map_err(|_| "Invalid from date")?
            .with_timezone(&Utc);

        let to = DateTime::parse_from_rfc3339(to_str)
            .map_err(|_| "Invalid to date")?
            .with_timezone(&Utc);

        if from >= to {
            return Err("from must be before to".to_string());
        }

        Ok(Self { from, to })
    }

    pub fn duration_days(&self) -> i64 {
        (self.to - self.from).num_days()
    }
}

/// Quantitative data point.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
}

/// Trend analysis result.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trend {
    pub metric_name: String,
    pub direction: TrendDirection,
    pub percentage_change: f64,
    pub significance: TrendSignificance,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrendSignificance {
    Low,
    Medium,
    High,
}

/// AI-generated insight.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Insight {
    pub title: String,
    pub description: String,
    pub recommendation: Option<String>,
    pub confidence: f64,
}

/// Aggregation results.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggregateResult {
    pub retrospective_id: Uuid,
    pub total_metrics: usize,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub timestamp: DateTime<Utc>,
}

/// Lightweight metadata.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetrospectiveMetadata {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author: Option<String>,
    pub range: TimeRange,
    pub metric_count: usize,
}
```

---

## 6. Test Utilities

```rust
// File: crates/phenotype-test-infra/src/retrospective_mocks.rs

use crate::ports::*;
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Mock repository for testing.
pub struct MockRetrospectiveRepository {
    pub calls: Arc<AtomicUsize>,
    pub should_fail: Arc<AtomicBool>,
}

#[async_trait]
impl RetrospectiveRepository for MockRetrospectiveRepository {
    async fn create(&self, retrospective: Retrospective) -> Result<Uuid, RepositoryError> {
        self.calls.fetch_add(1, Ordering::Relaxed);

        if self.should_fail.load(Ordering::Relaxed) {
            return Err(RepositoryError::Database("Mock failure".into()));
        }

        Ok(retrospective.id)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Retrospective>, RepositoryError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(None)
    }

    // ... other methods
}

/// Builder for test scenarios.
pub struct RetrospectiveTestFixture {
    repository: Arc<dyn RetrospectiveRepository>,
    cache: Arc<dyn RetrospectiveCache>,
    event_bus: Arc<dyn RetrospectiveEventBus>,
}

impl RetrospectiveTestFixture {
    pub fn new() -> Self {
        Self {
            repository: Arc::new(MockRetrospectiveRepository::default()),
            cache: Arc::new(MockRetrospectiveCache::default()),
            event_bus: Arc::new(MockRetrospectiveEventBus::default()),
        }
    }

    pub fn build_service(self) -> Arc<dyn RetrospectiveService> {
        Arc::new(RetrospectiveServiceImpl::new(
            self.repository,
            self.cache,
            self.event_bus,
        ))
    }
}
```

---

## 7. Usage Examples

### Example 1: Basic Generation

```rust
#[tokio::test]
async fn test_generate_retrospective() {
    let service = create_test_service();

    let range = TimeRange::parse("2026-03-01T00:00:00Z", "2026-03-31T23:59:59Z")?;
    let config = RetrospectiveConfig::default();

    let retrospective = service.generate(range, config).await?;

    assert!(!retrospective.metrics.is_empty());
    assert!(!retrospective.trends.is_empty());
}
```

### Example 2: Caching Aggregates

```rust
#[tokio::test]
async fn test_aggregates_cached() {
    let service = create_test_service();
    let id = Uuid::new_v4();

    // First call: computes (slow)
    let result1 = service.compute_aggregates(id).await?;

    // Second call: cached (fast)
    let result2 = service.compute_aggregates(id).await?;

    assert_eq!(result1.total_metrics, result2.total_metrics);
}
```

### Example 3: Event Publishing

```rust
#[tokio::test]
async fn test_generate_publishes_event() {
    let service = create_test_service();
    let event_bus = service.event_bus();

    service.generate(range, config).await?;

    let events = event_bus.published_events();
    assert!(events.iter().any(|e| {
        matches!(e, RetrospectiveEvent::Generated { .. })
    }));
}
```

---

## Summary

This trait definition provides:

- **Portability:** Service usable in CLI, Web API, GRPC
- **Testability:** Mock all dependencies
- **Event Sourcing:** Full audit trail
- **Caching:** Performance optimization
- **Error Handling:** Structured, actionable errors
- **Documentation:** Self-documenting contracts

**LOC Summary:**
- Inbound port: 200 LOC
- Outbound ports: 150 LOC
- Models: 100 LOC
- Test utilities: 150 LOC
- **Total:** ~600 LOC of reusable contracts
