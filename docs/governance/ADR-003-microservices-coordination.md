# ADR-003: Microservices Coordination with Event-Driven Architecture and Saga Pattern

- **Date**: 2026-03-30
- **Status**: Accepted
- **Deciders**: Phenotype Platform Team
- **Affected Repositories**: phenotype-infrakit, heliosCLI, heliosApp, platforms/thegent

---

## Context

The Phenotype platform has evolved from a monolithic codebase into three primary coordinating
services operating across distinct domains:

1. **phenotype-infrakit** — Rust-based shared library platform for core infrastructure,
   configuration, validation, event sourcing, state machines, and policy evaluation.
2. **heliosCLI** — Command-line harness for local development, workflow orchestration, and
   agent execution. Communicates with local processes, sandboxes, and remote backends.
3. **heliosApp / platforms/thegent** — Distributed Go-based agent orchestration platform,
   MCP server host, workflow coordination engine, and multi-backend execution environment.

### Current Coordination Challenges

Currently, these services communicate via:
- **Synchronous HTTP REST** with ad-hoc request/response contracts (no schema enforcement)
- **Shared Git repositories** for spec synchronization (eventual consistency, merge conflicts)
- **File system polling** for work queue and state synchronization
- **Direct process spawning** with environment variable pass-through (tight coupling)
- **In-memory message queues** within single processes (no cross-service observability)

This architecture creates:

- **Temporal coupling**: Services must be online simultaneously; request timeouts cause cascade
  failures.
- **Tight coupling**: Changes to one service's API require coordinated updates across dependent
  services.
- **Eventual consistency gaps**: Git-based spec sync can diverge for hours during merge
  conflicts or network outages.
- **Observable failures**: State machine transitions and long-running workflows lack visibility
  when split across services. Failure recovery requires manual intervention.
- **Scaling bottlenecks**: Synchronous HTTP hops create request amplification (N services = N
  serial latencies).

### Requirements for Cross-Service Coordination

1. **Loose coupling**: Services should not require online coordination; message delivery should
   be guaranteed asynchronously.
2. **Failure recovery**: Long-running workflows (e.g., multi-step agent execution, deployment
   pipelines) must survive individual service failures and resume from checkpoints.
3. **Observability**: All cross-service transactions must be queryable with causal ordering
   (distributed tracing, event logs, state audit trails).
4. **Scalability**: Adding new services should not increase latency for existing transactions.
5. **Eventual consistency**: Services must reconcile state without human intervention when
   transient failures resolve.

---

## Decision

The Phenotype platform MUST adopt an **event-driven architecture** with **asynchronous
messaging** for all cross-service coordination. Distributed workflows involving 2+ services
MUST implement the **Saga pattern** for transactional semantics and failure recovery.

### 1. Event-Driven Architecture

All cross-service state changes MUST be published as immutable events to a shared event broker.
Services subscribe to events relevant to their domain and maintain eventual consistency through
event replay.

#### Event Broker Technology

**Decision**: Use **NATS JetStream** as the primary event broker for MVP and early production
phases.

| Attribute | Kafka | NATS JetStream | Redis Streams | RabbitMQ |
|-----------|-------|----------------|----------------|----------|
| Deployment | Requires JVM, ZK/Kraft consensus | Native binary, single process | Single process | Erlang |
| Latency | ~5–50ms at 99th percentile | <1ms | <1ms | ~10ms |
| Exactly-once delivery | Transactional (Raft consensus) | Exactly-once via ACK/NAK | Eventually-consistent | With plugins (RabbitMQ) |
| Consumer groups | Built-in (complex) | Durable subscribers | Consumer groups | Built-in |
| Operational burden | High (ZK, brokers, monitoring) | Low (single binary) | Low | Medium |
| Persistence | Disk-backed durability | Disk-backed durability | Optional AOF/RDB | Optional persistence |
| Multi-region federation | Multi-region cluster | Jetstream node cluster | Geo replication | Shovel/Federation |
| Ecosystem maturity | Production-grade for scale | Mature; used by Cloudflare, Synadia | Mature; Redis ecosystem | Production-grade |

**Rationale**: NATS JetStream provides exactly-once delivery semantics with minimal operational
burden. The native binary deployment (no JVM, no external consensus layer) fits Phenotype's
DevOps model. JetStream's durable subscribers and consumer groups support both publish/subscribe
and work-queue patterns. For MVP and early production, NATS JetStream offers the best
cost-to-reliability ratio. **Future evolution**: If event volume exceeds JetStream's
single-node throughput (typically >100k msgs/sec), migrate to a managed Kafka cloud service
(e.g., Confluent Cloud, AWS MSK) without changing application code — events remain unchanged.

**Import paths**:
- Go: `github.com/nats-io/nats.go v1.36+`
- Rust: `nats = "0.25"`

#### Event Schema and Registry

All events MUST conform to a shared event schema and be registered in a central event catalog.

**Event Schema** (OpenAPI 3.1 + AsyncAPI 3.0):

```yaml
# schemas/events/agent.event.yaml
components:
  schemas:
    AgentExecutionStarted:
      type: object
      required:
        - event_id
        - event_type
        - timestamp
        - agent_id
        - workflow_id
        - payload
      properties:
        event_id:
          type: string
          format: uuid
          description: "Idempotency key for deduplication"
        event_type:
          type: string
          enum: ["agent.execution.started"]
        timestamp:
          type: string
          format: date-time
        agent_id:
          type: string
        workflow_id:
          type: string
        payload:
          type: object
          properties:
            environment:
              type: object
              additionalProperties:
                type: string
            timeout_seconds:
              type: integer
            entrypoint:
              type: string
    
    AgentExecutionCompleted:
      type: object
      required:
        - event_id
        - event_type
        - timestamp
        - agent_id
        - workflow_id
        - exit_code
      properties:
        event_id:
          type: string
          format: uuid
        event_type:
          type: string
          enum: ["agent.execution.completed"]
        timestamp:
          type: string
          format: date-time
        agent_id:
          type: string
        workflow_id:
          type: string
        exit_code:
          type: integer
        stdout:
          type: string
        stderr:
          type: string
        execution_duration_ms:
          type: integer
```

**Event Catalog** (`docs/reference/EVENT_CATALOG.md`):
- Registry of all publishable events across Phenotype repos
- Event ID, type, schema location, publishing service, consuming services
- Backward-compatibility and versioning rules
- Example: `docs/reference/EVENT_CATALOG.md#agent` lists all agent-related events

---

### 2. Asynchronous Messaging Patterns

#### a. Publish/Subscribe (Domain Events)

Services publish **domain events** to notify interested parties of state changes.

**Pattern**: A workflow completes in `platforms/thegent`; it publishes `workflow.completed`
event. Both `heliosCLI` and `heliosApp` subscribe independently and update their local state.

```go
// platforms/thegent/internal/orchestrator/publish.go
type WorkflowCompletedEvent struct {
    EventID      string    `json:"event_id"`
    EventType    string    `json:"event_type"` // "workflow.completed"
    Timestamp    time.Time `json:"timestamp"`
    WorkflowID   string    `json:"workflow_id"`
    ExitCode     int       `json:"exit_code"`
    ResultURL    string    `json:"result_url"`
}

func (orch *Orchestrator) PublishWorkflowCompleted(ctx context.Context, event WorkflowCompletedEvent) error {
    payload, _ := json.Marshal(event)
    _, err := orch.nc.PublishAsync("phenotype.workflow.completed", payload, func(pa *nats.PubAck, err error) {
        if err != nil {
            log.Errorf("Workflow completed event not acked: %v", err)
        }
    })
    return err
}

// heliosCLI/cmd/run.go subscribes
type WorkflowSubscriber struct {
    nc *nats.Conn
    db LocalCache
}

func (s *WorkflowSubscriber) Subscribe(ctx context.Context) {
    s.nc.Subscribe("phenotype.workflow.completed", func(msg *nats.Msg) {
        var event WorkflowCompletedEvent
        json.Unmarshal(msg.Data, &event)
        s.db.CacheWorkflowResult(event.WorkflowID, event.ExitCode)
        msg.Ack()
    })
}
```

#### b. Work Queue (Task Distribution)

Requests to perform work are enqueued; multiple workers contend for and process items.

**Pattern**: `heliosCLI` enqueues a sandboxed test run. Multiple `platforms/thegent` workers
poll the queue, claim the job, and report results back.

```go
// heliosCLI/internal/queue/enqueue.go
func (cli *CLI) EnqueueSandboxRun(ctx context.Context, spec SandboxRunSpec) (jobID string, err error) {
    jobID = uuid.New().String()
    msg := map[string]interface{}{
        "job_id": jobID,
        "spec":   spec,
        "created_at": time.Now(),
    }
    payload, _ := json.Marshal(msg)

    // JetStream work-queue durables automatically ensure only one worker gets each message
    _, err = cli.js.PublishAsync("phenotype.sandbox-runs", payload)
    return jobID, err
}

// platforms/thegent/internal/workers/sandbox_worker.go
func (w *SandboxWorker) Poll(ctx context.Context) error {
    sub, _ := w.js.PullSubscribe("phenotype.sandbox-runs", "sandbox-worker-group")
    
    for {
        msgs, _ := sub.Fetch(1, nats.MaxWait(5*time.Second))
        for _, msg := range msgs {
            var job SandboxRunRequest
            json.Unmarshal(msg.Data, &job)
            
            result := w.executeSandbox(ctx, job)
            w.publishSandboxResult(ctx, result) // Publish result event
            
            msg.Ack()
        }
    }
}
```

---

### 3. Saga Pattern for Distributed Transactions

When a workflow involves multiple services and failure recovery is critical, use the **Saga
pattern**. A saga is a sequence of local transactions coordinated via events.

#### Saga Orchestration vs. Choreography

| Pattern | Coordinator | Control Flow | Observability | Complexity |
|---------|-------------|--------------|-------------------|------------|
| **Orchestration** | Explicit saga coordinator service | Centralized state machine | Excellent (audit trail) | Medium |
| **Choreography** | Events trigger next step implicitly | Distributed, event-driven | Poor (must infer flow) | Low |

**Decision**: Use **orchestration** for sagas with >3 steps or critical failure recovery
requirements. Use **choreography** for simple event chains (e.g., "agent starts" → "agent
completes" → "notify downstream").

#### Saga Orchestration Example: Multi-Service Deployment

A deployment saga coordinates phenotype-infrakit versioning, heliosApp agent initialization,
and heliosCLI cache invalidation.

```rust
// phenotype-infrakit/src/orchestration/saga.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentSagaStep {
    VersionPublished { version: String },
    AgentInitialized { agent_id: String },
    CacheInvalidated { cache_generation: u64 },
    Rollback { reason: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentSaga {
    pub saga_id: String,
    pub deployment_id: String,
    pub steps: Vec<DeploymentSagaStep>,
    pub state: SagaState,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SagaState {
    Pending,
    InProgress,
    Completed,
    Failed,
    Rolled,
}

// Saga Coordinator publishes step commands; services respond with step-completed events
pub async fn coordinate_deployment(
    js: &jetstream::Context,
    deployment_id: String,
) -> Result<DeploymentSaga> {
    let saga_id = Uuid::new_v4().to_string();
    let mut saga = DeploymentSaga {
        saga_id: saga_id.clone(),
        deployment_id: deployment_id.clone(),
        steps: vec![],
        state: SagaState::Pending,
        created_at: Utc::now(),
    };

    // Step 1: Publish version to infrakit
    saga.state = SagaState::InProgress;
    js.publish("phenotype.deployment.version-publish", json!({
        "saga_id": saga.saga_id,
        "deployment_id": saga.deployment_id,
        "version": "v2.0.0",
        "timestamp": Utc::now(),
    }).to_string().into()).await?;

    // Wait for version-published event (with timeout)
    let step1_event = wait_for_event(
        js,
        "phenotype.deployment.version-published",
        &saga.saga_id,
        Duration::from_secs(30),
    ).await?;

    saga.steps.push(DeploymentSagaStep::VersionPublished {
        version: step1_event["version"].as_str().unwrap().to_string(),
    });

    // Step 2: Initialize agents in heliosApp
    js.publish("phenotype.deployment.agent-init", json!({
        "saga_id": saga.saga_id,
        "deployment_id": saga.deployment_id,
        "version": "v2.0.0",
        "timestamp": Utc::now(),
    }).to_string().into()).await?;

    let step2_event = wait_for_event(
        js,
        "phenotype.deployment.agent-initialized",
        &saga.saga_id,
        Duration::from_secs(60),
    ).await?;

    saga.steps.push(DeploymentSagaStep::AgentInitialized {
        agent_id: step2_event["agent_id"].as_str().unwrap().to_string(),
    });

    // Step 3: Invalidate heliosCLI caches
    js.publish("phenotype.deployment.cache-invalidate", json!({
        "saga_id": saga.saga_id,
        "deployment_id": saga.deployment_id,
        "timestamp": Utc::now(),
    }).to_string().into()).await?;

    let step3_event = wait_for_event(
        js,
        "phenotype.deployment.cache-invalidated",
        &saga.saga_id,
        Duration::from_secs(30),
    ).await?;

    saga.steps.push(DeploymentSagaStep::CacheInvalidated {
        cache_generation: step3_event["generation"].as_i64().unwrap() as u64,
    });

    saga.state = SagaState::Completed;
    Ok(saga)
}

// Rollback compensation
pub async fn rollback_deployment(
    js: &jetstream::Context,
    saga: &DeploymentSaga,
) -> Result<()> {
    for step in saga.steps.iter().rev() {
        match step {
            DeploymentSagaStep::VersionPublished { version } => {
                js.publish("phenotype.deployment.version-rollback", json!({
                    "version": version,
                    "timestamp": Utc::now(),
                }).to_string().into()).await?;
            }
            DeploymentSagaStep::AgentInitialized { agent_id } => {
                js.publish("phenotype.deployment.agent-rollback", json!({
                    "agent_id": agent_id,
                    "timestamp": Utc::now(),
                }).to_string().into()).await?;
            }
            DeploymentSagaStep::CacheInvalidated { .. } => {
                js.publish("phenotype.deployment.cache-restore", json!({
                    "timestamp": Utc::now(),
                }).to_string().into()).await?;
            }
            _ => {}
        }
    }
    Ok(())
}
```

#### Saga State Persistence

Saga state MUST be persisted to survive coordinator crashes:

- **Storage**: Store saga state in `phenotype-infrakit` shared database (e.g., SQLite for MVP).
- **Recovery**: On coordinator restart, resume any in-flight sagas from persisted checkpoint.
- **Idempotency**: Every saga step MUST be idempotent; replaying step N multiple times
  produces the same result.

---

### 4. Dead Letter Handling

Messages that cannot be processed are routed to a **dead letter queue** for manual review:

```go
// platforms/thegent/internal/handlers/error_handler.go
func HandleDeadLetter(ctx context.Context, js nats.JetStreamContext, msg *nats.Msg) {
    var payload interface{}
    json.Unmarshal(msg.Data, &payload)
    
    // Log error with context
    log.WithFields(logrus.Fields{
        "subject":     msg.Subject,
        "redelivered": msg.Metadata().NumDelivered,
        "payload":     payload,
        "timestamp":   time.Now(),
    }).Error("Message failed processing; moving to dead letter")
    
    // Publish to dead letter queue for ops team
    js.PublishAsync("phenotype.dead-letter", msg.Data)
    
    // Send alert
    alerting.NotifyOps("Dead letter: " + msg.Subject)
}
```

---

### 5. Integration with Existing HTTP APIs

During the transition period, HTTP REST APIs coexist with event-driven patterns:

- **Outbound HTTP** (from Phenotype services to external APIs like GitHub, LLM providers):
  Continue using `go-retryablehttp` + `gobreaker` (from ADR-001). Do not migrate to events.
- **Inbound HTTP** (requests from clients to Phenotype services): Support both:
  - Synchronous REST for backward compatibility (deprecated post-migration)
  - Async job endpoints that enqueue work and return a job ID for polling
- **Polling pattern**: Client calls `GET /jobs/{job_id}` to poll event-sourced job status.

```go
// heliosCLI/api/jobs.go
func (s *Server) PollJobStatus(w http.ResponseWriter, r *http.Request) {
    jobID := chi.URLParam(r, "job_id")
    
    // Query event store for all events related to this job
    events, _ := s.eventStore.EventsForAggregate("workflow", jobID)
    
    // Reconstruct state from event stream
    status := ReconstructWorkflowStatus(events)
    json.NewEncoder(w).Encode(status)
}
```

---

## Consequences

### Positive

- **Loose coupling**: Services can be deployed, updated, and scaled independently. Adding new
  services requires only consuming relevant event topics.
- **Failure resilience**: Message delivery is guaranteed by JetStream durability. Failed
  services recover by replaying events from checkpoints. Saga compensation ensures
  transactional semantics for multi-step workflows.
- **Observable workflows**: Every cross-service action is recorded as an immutable event.
  Operators can query event logs to understand workflow causality and debug failures.
- **Scalability**: Work distribution via work queues (NATS JetStream) scales linearly.
  Adding workers increases throughput without changing application code.
- **Eventual consistency recovery**: Transient network partitions are healed automatically
  via event replay. Manual intervention is eliminated.
- **Testability**: Event-driven systems are easier to test; services can be tested in
  isolation by replaying recorded events.

### Negative

- **Operational complexity**: Running NATS JetStream introduces a new external dependency
  that must be monitored, backed up, and secured. Operators must understand JetStream
  durables, consumer groups, and ack semantics.
- **Debugging difficulty**: Distributed workflows are harder to debug than synchronous
  request chains. Tracing requires correlation IDs across all events.
- **Data consistency gaps**: Eventual consistency means services may temporarily diverge.
  Clients must handle stale reads (e.g., cached workflow status may lag reality by seconds).
- **Event versioning burden**: Changing event schemas requires care to avoid breaking
  older consumers. Versioning policies must be enforced (see Implementation Notes).
- **Latency increase**: Asynchronous messaging introduces latency compared to synchronous
  RPC (order of ~10–100ms per hop depending on NATS deployment). Latency-sensitive
  operations (e.g., interactive CLI commands) must use request/reply patterns instead.

---

## Alternatives Considered

### 1. Synchronous Request/Reply via HTTP (Retain)

**Rejected**. Current HTTP-based coordination creates temporal coupling and cascade failures.
Multi-service deployments require coordinated updates, and transient service outages block
all downstream requests. Sagas over HTTP require custom retry logic and lack standard
compensation semantics.

### 2. gRPC Streaming (Request/Reply)

**Considered**. gRPC provides lower latency and binary serialization compared to REST JSON.
However, gRPC requires code generation (protobuf), bidirectional streaming complexity, and
does not solve the loose-coupling problem. Rejected in favour of events.

### 3. Kafka (Event Broker)

**Considered**. Kafka is production-grade at scale and provides stronger ordering guarantees
than NATS. However, Kafka requires a JVM, ZooKeeper or Kraft consensus, and significant
operational overhead. For MVP and early production, NATS JetStream offers better
cost-to-reliability. **Future evolution path**: Migrate to Kafka when event throughput
exceeds NATS capacity (~100k msgs/sec).

### 4. Temporal Workflow Orchestration (Distributed Transactions)

**Considered**. Temporal is a dedicated system for long-running workflows with built-in
replay, compensation, and observability. It eliminates the need to hand-code saga
orchestrators. However, Temporal introduces a new operational dependency and vendor lock-in
(Temporal Cloud). Rejected for MVP; revisit if saga complexity becomes unmaintainable.

### 5. AWS SQS/SNS or Google Cloud Pub/Sub (Managed Services)

**Considered**. Managed event systems eliminate operational burden but introduce cloud
vendor lock-in. Phenotype's multi-cloud / on-premises deployment model requires
infrastructure-agnostic messaging. Rejected in favour of self-hosted NATS.

### 6. Event Sourcing as Primary Storage (Not Just Events)

**Considered**. Event sourcing (storing only events, deriving state via replay) provides
strong audit trails and temporal querying. However, it adds complexity to read paths (must
replay or maintain projections). For MVP, store events primarily for coordination; persist
snapshots for fast state reconstruction. Evaluate full event sourcing post-MVP.

---

## Implementation Notes

### Phase 1: Infrastructure Setup (Week 1)

1. **Provision NATS JetStream** — Deploy single-node JetStream in development environment
   (Docker Compose). Configure persistence (file-based), authentication (nkey), and
   monitoring (Prometheus metrics).
2. **Define Event Schema Registry** — Create `docs/reference/EVENT_CATALOG.md` and event
   OpenAPI specs in `schemas/events/`.
3. **Implement NATS Client Libraries** — Wrap `nats.go` in `phenotype-go-middleware` with
   helpers for publish, subscribe, request/reply, and error handling. Implement Rust
   equivalents in phenotype-infrakit.
4. **Create Shared Event Types** — Define canonical event structs/types for common workflows
   (agent execution, deployment, job completion).

### Phase 2: Pilot Event Stream (Week 2–3)

1. **Migrate Job Queue** — Move heliosCLI sandbox job enqueueing to NATS work queue
   (currently file-based polling).
2. **Publish Workflow Events** — Modify platforms/thegent orchestrator to publish
   workflow-completed events to NATS.
3. **Subscribe in heliosCLI** — Update heliosCLI to subscribe to workflow-completed events
   and cache results locally.
4. **Add Dead Letter Handling** — Implement error handler that routes processing failures
   to dead letter queue.
5. **Enable Tracing** — Add correlation IDs to all events; implement distributed tracing
   with jaeger or zipkin.

### Phase 3: Saga Deployment (Week 4–5)

1. **Implement Saga Coordinator** — Create saga orchestrator service (embedded in
   phenotype-infrakit or heliosApp) that manages multi-step workflows.
2. **Define Deployment Saga** — Model deployment workflow as saga (version publish →
   agent init → cache invalidate) with compensation steps.
3. **Test Saga Failure Recovery** — Simulate service failures and verify saga rollback.
4. **Document Saga Patterns** — Create runbooks for ops team on monitoring, alerting, and
   manual saga recovery.

### Phase 4: Production Hardening (Week 6+)

1. **HA NATS Cluster** — Deploy NATS JetStream as 3-node cluster for high availability.
2. **Backup/Recovery** — Implement backup of JetStream state; test disaster recovery.
3. **Performance Testing** — Load test event throughput; benchmark latency at scale.
4. **Schema Versioning Policy** — Document backward compatibility rules for event schema
   changes (additive only until deprecation period).
5. **Monitoring & Alerting** — Add dashboards for event latency, consumer lag, dead letter
   rate; configure PagerDuty alerts.

### Event Versioning Policy

- **Version in event_type**: `agent.execution.started@1.0`
- **Backward compatibility rule**: New event fields must be optional; removing fields
  requires a major version bump and deprecation period (6 weeks notice).
- **Consumer compatibility**: Consumers must gracefully handle unknown fields (use OpenAPI
  `additionalProperties: false` to prevent surprises).
- **Schema registry**: Central schema registry (e.g., Confluent Schema Registry, Buf BSR)
  may be added post-MVP if schema divergence becomes a problem.

### Observability and Tracing

All published events MUST include:

```json
{
    "event_id": "uuid-for-idempotency",
    "event_type": "agent.execution.started@1.0",
    "timestamp": "2026-03-30T12:34:56Z",
    "correlation_id": "workflow-abc123",
    "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
    "span_id": "00f067aa0ba902b7",
    "domain": "agent",
    "source": "platforms.thegent.orchestrator"
}
```

Implement correlation ID propagation across all service boundaries. Use OpenTelemetry to
emit traces; export to Jaeger or Zipkin for visualization.

### Code Examples

#### Go Publisher (platforms/thegent)

```go
// platforms/thegent/internal/events/publisher.go
package events

import (
    "context"
    "encoding/json"
    "time"
    "github.com/google/uuid"
    "github.com/nats-io/nats.go"
)

type Publisher struct {
    nc *nats.Conn
    js nats.JetStreamContext
}

type AgentExecutionStartedEvent struct {
    EventID      string                 `json:"event_id"`
    EventType    string                 `json:"event_type"`
    Timestamp    time.Time              `json:"timestamp"`
    CorrelationID string                `json:"correlation_id"`
    AgentID      string                 `json:"agent_id"`
    WorkflowID   string                 `json:"workflow_id"`
    Payload      map[string]interface{} `json:"payload"`
}

func (p *Publisher) PublishAgentExecutionStarted(
    ctx context.Context,
    agentID, workflowID string,
    payload map[string]interface{},
) (string, error) {
    eventID := uuid.New().String()
    event := AgentExecutionStartedEvent{
        EventID:       eventID,
        EventType:     "agent.execution.started@1.0",
        Timestamp:     time.Now().UTC(),
        CorrelationID: CorrelationIDFromContext(ctx),
        AgentID:       agentID,
        WorkflowID:    workflowID,
        Payload:       payload,
    }

    data, _ := json.Marshal(event)
    ack, err := p.js.PublishAsync("phenotype.agent.execution.started", data)
    if err != nil {
        return "", err
    }

    select {
    case <-ack.Ok():
        return eventID, nil
    case <-ack.Err():
        return "", ack.Err()
    case <-ctx.Done():
        return "", ctx.Err()
    }
}
```

#### Go Subscriber (heliosCLI)

```go
// heliosCLI/internal/events/subscriber.go
package events

import (
    "context"
    "encoding/json"
    "log"
    "github.com/nats-io/nats.go"
)

type Subscriber struct {
    nc *nats.Conn
    js nats.JetStreamContext
}

func (s *Subscriber) SubscribeToAgentExecutionEvents(ctx context.Context) error {
    // Create durable consumer for events
    _, err := s.js.Subscribe("phenotype.agent.execution.>", func(msg *nats.Msg) {
        var event map[string]interface{}
        json.Unmarshal(msg.Data, &event)

        log.Printf("Received event: %s, correlation: %s", 
            event["event_type"], 
            event["correlation_id"])

        // Process event (update local cache, trigger local workflows, etc.)
        handleAgentExecutionEvent(event)

        // Acknowledge successful processing
        msg.Ack()
    }, nats.Durable("helios-cli-agent-events"))

    return err
}
```

---

## References

- **NATS JetStream**: https://docs.nats.io/nats-concepts/jetstream
- **Event Sourcing Pattern**: https://martinfowler.com/eaaDev/EventSourcing.html
- **Saga Pattern**: https://chrisrichardson.net/post/microservices/2019/07/09/developing-sagas-part-1.html
- **Choreography vs Orchestration**: https://learn.microsoft.com/en-us/azure/architecture/patterns/saga
- **OpenAPI & AsyncAPI**: https://www.asyncapi.com/
- **Distributed Tracing**: https://opentelemetry.io/docs/
- **NATS vs Kafka**: https://natsbyexample.com/
- **Temporal Workflows**: https://temporal.io/ (for future consideration)
- **Event Versioning**: https://www.confluent.io/blog/event-streaming-patterns-in-kafka/ (schema evolution)
- **Dead Letter Queues**: https://docs.nats.io/nats-concepts/jetstream/consumer#dead-letter-policy
- **Idempotency**: https://stripe.com/docs/idempotency
- **Correlation IDs**: https://www.w3.org/TR/trace-context/
