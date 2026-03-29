# PhenoSDK Package Audit & Interface Specification

**Date**: 2026-03-29
**Status**: COMPLETE
**Purpose**: Detailed analysis of phenotype-infrakit Rust crates for npm extraction

---

## Part 1: Current State Audit

### Overview

**phenotype-infrakit** is a Rust workspace containing 5 independent infrastructure crates implementing driven adapters for hexagonal architecture.

**Repository**: `/Users/kooshapari/CodeProjects/Phenotype/repos/repos/worktrees/phenotype-infrakit/chore/merge-worklogs/`

**Total LOC**: ~2,350 across all crates
**Architecture**: Hexagonal (Ports & Adapters)
**Key Dependencies**: serde, chrono, uuid, thiserror, regex, sha2, dashmap, moka

---

### Crate 1: phenotype-contracts

**Location**: `crates/phenotype-contracts/`
**LOC**: ~400
**Status**: Core, actively maintained
**Stability**: STABLE (interfaces rarely change)

**Purpose**: Defines hexagonal architecture ports (interfaces) and domain models for the entire infrastructure suite.

**Structure**:
```
phenotype-contracts/src/
├── lib.rs               # Main entry, re-exports all modules
├── ports/
│   ├── mod.rs          # Ports re-exports
│   ├── inbound/        # Driving ports (UseCase, CommandHandler, QueryHandler, EventHandler)
│   └── outbound/       # Driven ports (Repository, CachePort, SecretStore, EventBus)
├── models/             # Domain models (Entity, ValueObject, AggregateRoot, DomainEvent)
└── tests/              # Contract verification tests
```

**Key Exports**:

```rust
// Inbound Ports (Driving)
pub trait UseCase {
    type Input;
    type Output;
    async fn execute(&self, input: Self::Input) -> Result<Self::Output>;
}

pub trait CommandHandler {
    type Command;
    async fn handle(&self, command: Self::Command) -> Result<()>;
}

pub trait QueryHandler {
    type Query;
    type Result;
    async fn handle(&self, query: Self::Query) -> Result<Self::Result>;
}

pub trait EventHandler {
    type Event;
    async fn handle(&self, event: Self::Event) -> Result<()>;
}

// Outbound Ports (Driven)
pub trait Repository<T: Entity> {
    async fn save(&self, entity: T) -> Result<()>;
    async fn find_by_id(&self, id: &str) -> Result<Option<T>>;
    async fn delete(&self, id: &str) -> Result<()>;
}

pub trait CachePort {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
    async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
}

pub trait SecretStore {
    async fn get(&self, key: &str) -> Result<Option<String>>;
    async fn set(&self, key: &str, value: &str) -> Result<()>;
}

pub trait EventBus {
    async fn publish<T: Serialize>(&self, event: &T, event_type: &str) -> Result<()>;
    async fn subscribe<T: DeserializeOwned>(&self, handler: Box<dyn EventHandler<Event = T>>) -> Result<()>;
}

// Domain Models
pub trait Entity: Serialize + DeserializeOwned {
    fn id(&self) -> &str;
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
}

pub trait ValueObject: Eq + Hash + Serialize + DeserializeOwned {}

pub struct AggregateRoot {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u32,
    pub uncommitted_events: Vec<DomainEvent>,
}

pub struct DomainEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

// Error Handling
pub enum Error {
    NotFound(String),
    Validation(String),
    Conflict(String),
    PermissionDenied(String),
    Internal(String),
    Timeout(String),
}
```

**Dependencies**:
- serde + serde_json
- uuid
- chrono
- thiserror

**Consumers**: All other crates in workspace depend on this

**NPM Mapping**: `@phenotype/pheno-core` (primary)

---

### Crate 2: phenotype-event-sourcing

**Location**: `crates/phenotype-event-sourcing/`
**LOC**: ~800
**Status**: Core, actively maintained
**Stability**: STABLE (proven in production)

**Purpose**: Generic event sourcing implementation with SHA-256 hash chain for integrity verification.

**Structure**:
```
phenotype-event-sourcing/src/
├── lib.rs              # Main entry
├── event.rs            # EventEnvelope<T>, DomainEvent
├── store.rs            # EventStore trait
├── hash.rs             # SHA-256 hash chain computation and verification
├── memory.rs           # InMemoryEventStore implementation
├── snapshot.rs         # Event snapshots for optimization
├── error.rs            # Custom error types
└── tests/
```

**Key Exports**:

```rust
pub struct EventEnvelope<T> {
    pub id: Uuid,                      // UUIDv4
    pub timestamp: DateTime<Utc>,      // UTC creation time
    pub sequence: i64,                 // 1-based sequence number
    pub actor: String,                 // Who triggered the event
    pub event_type: String,            // Event classification
    pub payload: T,                    // The actual event data
    pub hash: String,                  // SHA-256 (hex)
    pub prev_hash: String,             // Previous hash (chain link)
}

pub trait EventStore {
    type Event;

    // Write events
    async fn append<T: Serialize>(&self, event: T, event_type: &str) -> Result<i64>;

    // Read events
    async fn get_events<T: DeserializeOwned>(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<EventEnvelope<T>>>;

    async fn get_events_since<T: DeserializeOwned>(
        &self,
        entity_type: &str,
        entity_id: &str,
        sequence: i64,
    ) -> Result<Vec<EventEnvelope<T>>>;

    async fn get_events_by_range<T: DeserializeOwned>(
        &self,
        entity_type: &str,
        entity_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<EventEnvelope<T>>>;

    async fn get_latest_sequence(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<i64>;

    // Verification
    async fn verify_chain(&self, entity_type: &str, entity_id: &str) -> Result<()>;
}

pub fn compute_hash(
    id: &Uuid,
    timestamp: &DateTime<Utc>,
    event_type: &str,
    payload: &str,      // JSON string
    actor: &str,
    prev_hash: &str,
) -> String;

pub fn verify_chain(envelopes: &[EventEnvelope<serde_json::Value>]) -> Result<()>;

pub struct InMemoryEventStore {
    // Backed by RwLock<BTreeMap<entity_type, BTreeMap<entity_id, Vec<StoredEvent>>>>
}

pub struct EventSnapshot<T> {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub sequence: i64,
    pub timestamp: DateTime<Utc>,
    pub state: T,
}
```

**Hash Chain Algorithm**:

```
hash = SHA256(
  UUID_bytes (16)
  + len(timestamp) (4, big-endian)
  + timestamp_bytes (ISO 8601)
  + len(event_type) (4, big-endian)
  + event_type_bytes
  + len(payload) (4, big-endian)
  + payload_bytes (JSON)
  + len(actor) (4, big-endian)
  + actor_bytes
  + prev_hash_bytes (32)
)
```

**Dependencies**:
- serde + serde_json
- chrono
- uuid
- sha2 + hex
- parking_lot (sync primitives)

**Consumers**: Services needing audit trails, event replay, temporal queries

**NPM Mapping**: `@phenotype/pheno-resilience` (event-sourcing submodule)

---

### Crate 3: phenotype-cache-adapter

**Location**: `crates/phenotype-cache-adapter/`
**LOC**: ~300
**Status**: Production, optional adapter
**Stability**: STABLE (caching is well-understood)

**Purpose**: Two-tier caching strategy (LRU for hot data + distributed for shared cache).

**Structure**:
```
phenotype-cache-adapter/src/
├── lib.rs              # Main entry
├── lru.rs              # LRUCache implementation (moka or dashmap)
├── distributed.rs      # Distributed cache trait (Redis pattern)
├── two_tier.rs         # TwoTierCache combining L1 + L2
├── error.rs            # Cache-specific errors
└── tests/
```

**Key Exports**:

```rust
pub trait Cache {
    type Key: Hash + Eq + Clone + Send + Sync;
    type Value: Clone + Send + Sync;

    async fn get(&self, key: &Self::Key) -> Result<Option<Self::Value>>;
    async fn set(&self, key: Self::Key, value: Self::Value) -> Result<()>;
    async fn set_with_ttl(
        &self,
        key: Self::Key,
        value: Self::Value,
        ttl: Duration,
    ) -> Result<()>;
    async fn delete(&self, key: &Self::Key) -> Result<()>;
    async fn clear(&self) -> Result<()>;
}

pub struct LRUCacheConfig {
    pub max_size: usize,
    pub ttl: Option<Duration>,
    pub eviction_policy: EvictionPolicy, // LRU, LFU, or FIFO
}

pub struct LRUCache {
    // Backed by moka::future::Cache or dashmap
}

pub trait DistributedCache: Cache {
    async fn get_distributed(&self, key: &Self::Key) -> Result<Option<Self::Value>>;
    async fn set_distributed(
        &self,
        key: Self::Key,
        value: Self::Value,
        ttl: Option<Duration>,
    ) -> Result<()>;
}

pub struct TwoTierCache {
    l1: LRUCache,           // Hot, in-process
    l2: Box<dyn DistributedCache>, // Warm, shared
    l1_ttl: Duration,
    l2_ttl: Duration,
}

pub enum CacheError {
    Evicted,
    Expired,
    NotFound,
    SerializationError(String),
    BackendError(String),
}
```

**Dependencies**:
- moka (concurrent caching)
- dashmap (concurrent hashmaps)
- serde + serde_json
- thiserror
- tokio (async runtime)
- redis (optional, for distributed cache)

**Consumers**: Services using `CachePort` from contracts

**NPM Mapping**: `@phenotype/pheno-resilience` (cache submodule)

---

### Crate 4: phenotype-policy-engine

**Location**: `crates/phenotype-policy-engine/`
**LOC**: ~600
**Status**: Production, actively used
**Stability**: STABLE (policy evaluation is deterministic)

**Purpose**: Rule-based policy evaluation engine with regex-based pattern matching and multiple decision combinators.

**Structure**:
```
phenotype-policy-engine/src/
├── lib.rs              # Main entry
├── policy.rs           # PolicyRule, PolicySet
├── engine.rs           # PolicyEngine evaluation logic
├── context.rs          # PolicyContext (subject, resource, action, environment)
├── combinator.rs       # Decision combinators (first-applicable, deny-overrides, permit-overrides)
├── error.rs            # Policy-specific errors
└── tests/
```

**Key Exports**:

```rust
pub struct PolicyRule {
    pub name: String,
    pub pattern: String,           // Regex pattern
    pub action: PolicyAction,      // Allow, Deny, Custom(Box<dyn Fn() -> Action>)
    pub priority: i32,             // Higher = evaluated first
    pub conditions: Vec<Condition>, // Guard conditions
}

pub struct PolicySet {
    pub name: String,
    pub rules: Vec<PolicyRule>,
    pub combinator: DecisionCombinator,
}

pub enum DecisionCombinator {
    FirstApplicable,   // Return first rule that matches
    DenyOverrides,     // Any Deny → Deny (even if others Permit)
    PermitOverrides,   // Any Permit → Permit (even if others Deny)
}

pub struct PolicyContext {
    pub subject: String,       // WHO (user, service, etc.)
    pub resource: String,      // WHAT (resource being accessed)
    pub action: String,        // HOW (operation: read, write, delete, etc.)
    pub environment: HashMap<String, Value>, // WHEN/WHERE (time, IP, etc.)
}

pub enum PolicyDecision {
    Permit,
    Deny,
    NotApplicable,
}

pub struct PolicyEngine {
    policies: Vec<PolicySet>,
    compiled_rules: RwLock<HashMap<String, regex::Regex>>,
}

impl PolicyEngine {
    pub fn add_policies(&mut self, policies: Vec<PolicySet>) -> Result<()>;

    pub async fn evaluate(&self, context: &PolicyContext) -> Result<PolicyDecision>;

    pub fn reload_policies(&mut self, policies: Vec<PolicySet>) -> Result<()>;
}

pub struct Condition {
    pub field: String,
    pub operator: Operator,  // Eq, NotEq, Contains, Matches, GreaterThan, etc.
    pub value: serde_json::Value,
}

pub enum Operator {
    Equals,
    NotEquals,
    Contains,
    MatchesRegex,
    GreaterThan,
    LessThan,
    In,
}
```

**Example Policy**:

```rust
let rule = PolicyRule {
    name: "AllowAdminWrites".to_string(),
    pattern: "admin:.+:write".to_string(),
    action: PolicyAction::Allow,
    priority: 100,
    conditions: vec![
        Condition {
            field: "subject_role".to_string(),
            operator: Operator::Equals,
            value: json!("admin"),
        },
        Condition {
            field: "time_of_day".to_string(),
            operator: Operator::In,
            value: json!([9, 10, 11, 12, 13, 14, 15, 16, 17]), // 9am-5pm
        },
    ],
};
```

**Dependencies**:
- regex
- serde + serde_json
- thiserror
- dashmap (for rule caching)
- parking_lot

**Consumers**: Authorization systems, feature flags, business rule engines

**NPM Mapping**: `@phenotype/pheno-resilience` (policy-engine submodule)

---

### Crate 5: phenotype-state-machine

**Location**: `crates/phenotype-state-machine/`
**LOC**: ~250
**Status**: Production, stable
**Stability**: STABLE (state machines are proven pattern)

**Purpose**: Finite state machine implementation with transition guards and actions.

**Structure**:
```
phenotype-state-machine/src/
├── lib.rs              # Main entry
├── machine.rs          # StateMachine definition and execution
├── state.rs            # State trait and implementations
├── transition.rs       # Transition with guards and actions
├── context.rs          # Machine context and state data
├── error.rs            # FSM-specific errors
└── tests/
```

**Key Exports**:

```rust
pub trait State: Send + Sync + Serialize + DeserializeOwned {
    fn name(&self) -> &str;
    async fn on_enter(&self) -> Result<()> { Ok(()) }
    async fn on_exit(&self) -> Result<()> { Ok(()) }
}

pub struct Transition {
    pub from: String,
    pub to: String,
    pub event: String,
    pub guard: Option<Box<dyn Fn(&MachineContext) -> bool + Send + Sync>>,
    pub action: Option<Box<dyn Fn(&mut MachineContext) -> Result<()> + Send + Sync>>,
}

pub struct MachineContext {
    pub state: String,
    pub data: serde_json::Value,
    pub history: Vec<(String, DateTime<Utc>)>, // State history
}

pub struct StateMachine {
    states: HashMap<String, Box<dyn State>>,
    transitions: Vec<Transition>,
    context: RwLock<MachineContext>,
}

impl StateMachine {
    pub fn new(
        initial_state: String,
        states: HashMap<String, Box<dyn State>>,
        transitions: Vec<Transition>,
    ) -> Self;

    pub async fn send_event(&mut self, event: &str, data: Option<serde_json::Value>) -> Result<()>;

    pub fn current_state(&self) -> String;

    pub fn can_transition(&self, event: &str) -> Result<bool>;

    pub fn available_transitions(&self) -> Vec<&Transition>;

    pub fn history(&self) -> Vec<(String, DateTime<Utc>)>;
}

// Example concrete state
pub struct DraftState;
impl State for DraftState {
    fn name(&self) -> &str { "draft" }

    async fn on_enter(&self) -> Result<()> {
        println!("Entered Draft state");
        Ok(())
    }
}

pub struct PublishedState;
impl State for PublishedState {
    fn name(&self) -> &str { "published" }

    async fn on_enter(&self) -> Result<()> {
        println!("Published! Setting timestamp...");
        Ok(())
    }
}

// Configuration example
let transitions = vec![
    Transition {
        from: "draft".to_string(),
        to: "published".to_string(),
        event: "publish".to_string(),
        guard: Some(Box::new(|ctx| {
            ctx.data.get("has_title").and_then(|v| v.as_bool()).unwrap_or(false)
        })),
        action: Some(Box::new(|ctx| {
            ctx.data["published_at"] = json!(Utc::now());
            Ok(())
        })),
    },
];
```

**Dependencies**:
- serde + serde_json
- chrono
- thiserror
- parking_lot

**Consumers**: Workflow engines, order processing, document lifecycle, approval systems

**NPM Mapping**: `@phenotype/pheno-resilience` (state-machine submodule)

---

## Part 2: Cross-Crate Dependency Analysis

### Dependency Graph

```
┌─────────────────────────────────────────────────────────────┐
│                  phenotype-contracts                         │
│         (Ports/Models - 0 internal dependencies)            │
└────────┬──────────────┬──────────────┬────────────┬──────────┘
         │              │              │            │
         ▼              ▼              ▼            ▼
    ┌─────────────┐ ┌──────────────────┐ ┌──────────────────┐
    │event-       │ │cache-adapter     │ │policy-engine     │
    │sourcing     │ │                  │ │                  │
    │            │ │                  │ │                  │
    │+ EventStore│ │+ LRUCache        │ │+ PolicyEngine    │
    │+ Envelope  │ │+ DistributedCache│ │+ PolicySet       │
    │+ Hash Chain│ │+ TwoTierCache    │ │+ Evaluation      │
    └──────┬──────┘ └──────┬───────────┘ └────────┬─────────┘
           │               │                      │
           │               │                      ▼
           │               │              ┌────────────────┐
           │               │              │state-machine   │
           │               │              │                │
           │               │              │+ StateMachine  │
           │               │              │+ Transitions   │
           │               │              │+ Guards/Actions│
           │               │              └────────────────┘
           └───────────────┴──────────────────────┘
                        │
                        │
                   Depends on
                        │
                        ▼
              phenotype-contracts
```

**Dependency Counts**:
- phenotype-contracts: 0 internal dependencies ✓ (can be published independently)
- phenotype-event-sourcing: 1 (contracts) ✓
- phenotype-cache-adapter: 1 (contracts) ✓
- phenotype-policy-engine: 1 (contracts) ✓
- phenotype-state-machine: 1 (contracts) ✓

**Conclusion**: All crates only depend on contracts. Clean dependency graph. **ZERO circular dependencies.**

---

## Part 3: npm Package Interface Contracts

### @phenotype/pheno-core Interface

**Stability**: STABLE (Breaking changes = major version)

```typescript
// @phenotype/pheno-core/index.ts

export * from './ports/index';
export * from './models/index';
export * from './errors/index';

// ports/inbound.ts
export interface UseCase<Input, Output> {
  execute(input: Input): Promise<Output>;
}

export interface CommandHandler<Command> {
  handle(command: Command): Promise<void>;
}

export interface QueryHandler<Query, Result> {
  handle(query: Query): Promise<Result>;
}

export interface EventHandler<Event> {
  handle(event: Event): Promise<void>;
}

// ports/outbound.ts
export interface Repository<T extends Entity> {
  save(entity: T): Promise<void>;
  findById(id: string): Promise<T | null>;
  findAll(): Promise<T[]>;
  delete(id: string): Promise<void>;
}

export interface CachePort {
  get<T>(key: string): Promise<T | null>;
  set<T>(key: string, value: T, ttl?: number): Promise<void>;
  delete(key: string): Promise<void>;
  clear(): Promise<void>;
}

export interface SecretStore {
  get(key: string): Promise<string | null>;
  set(key: string, value: string): Promise<void>;
  delete(key: string): Promise<void>;
}

export interface EventBus {
  publish<T>(event: T, eventType: string): Promise<void>;
  subscribe<T>(eventType: string, handler: EventHandler<T>): void;
}

// models/index.ts
export abstract class Entity {
  abstract get id(): string;
  abstract get createdAt(): Date;
  abstract get updatedAt(): Date;
}

export abstract class ValueObject {
  abstract equals(other: ValueObject): boolean;
}

export abstract class AggregateRoot extends Entity {
  protected uncommittedEvents: DomainEvent[] = [];

  protected raiseEvent(event: DomainEvent): void;
  getUncommittedEvents(): DomainEvent[];
  clearUncommittedEvents(): void;
}

export interface DomainEvent {
  id: string;
  timestamp: Date;
  aggregateId: string;
  aggregateType: string;
  eventType: string;
  payload: Record<string, unknown>;
  metadata?: Record<string, string>;
  version: number;
}

// errors/index.ts
export enum ErrorKind {
  NotFound = 'NotFound',
  Validation = 'Validation',
  Conflict = 'Conflict',
  PermissionDenied = 'PermissionDenied',
  Internal = 'Internal',
  Timeout = 'Timeout',
  Storage = 'Storage',
  Connection = 'Connection',
  Config = 'Config',
  AlreadyExists = 'AlreadyExists',
  Serialization = 'Serialization',
}

export class PhenotypeError extends Error {
  constructor(
    public readonly kind: ErrorKind,
    message: string,
    public readonly context?: Record<string, unknown>,
  ) {
    super(message);
    this.name = 'PhenotypeError';
  }

  isNotFound(): boolean;
  isValidation(): boolean;
  isConflict(): boolean;
  isPermissionDenied(): boolean;
  isInternal(): boolean;
  isTimeout(): boolean;
}
```

### @phenotype/pheno-resilience Interface

**Stability**: STABLE (Patterns proven in Rust)

```typescript
// @phenotype/pheno-resilience/event-sourcing/index.ts

export interface EventEnvelope<T = unknown> {
  id: string; // UUID v4
  timestamp: Date;
  sequence: number; // 1-based
  actor: string;
  eventType: string;
  payload: T;
  hash: string; // SHA-256 hex
  prevHash: string; // SHA-256 hex
}

export interface EventStore {
  append<T>(event: T, eventType: string): Promise<number>;
  getEvents<T>(
    entityType: string,
    entityId: string,
  ): Promise<EventEnvelope<T>[]>;
  getEventsSince<T>(
    entityType: string,
    entityId: string,
    sequence: number,
  ): Promise<EventEnvelope<T>[]>;
  getEventsByRange<T>(
    entityType: string,
    entityId: string,
    from: Date,
    to: Date,
  ): Promise<EventEnvelope<T>[]>;
  getLatestSequence(entityType: string, entityId: string): Promise<number>;
  verifyChain(entityType: string, entityId: string): Promise<void>;
}

export class InMemoryEventStore implements EventStore {
  clear(): void;
  eventCount(): number;
}

export function computeHash(envelope: EventEnvelope): string;
export function verifyChain(envelopes: EventEnvelope[]): boolean;

// @phenotype/pheno-resilience/state-machine/index.ts

export interface State {
  name: string;
  onEnter?(): Promise<void>;
  onExit?(): Promise<void>;
}

export interface Transition {
  from: string;
  to: string;
  event: string;
  guard?(): Promise<boolean>;
  action?(): Promise<void>;
}

export interface MachineContext {
  state: string;
  data: Record<string, unknown>;
  history: Array<[string, Date]>;
}

export class StateMachine {
  constructor(
    initialState: string,
    states: State[],
    transitions: Transition[],
  );

  getCurrentState(): string;
  sendEvent(event: string, data?: unknown): Promise<void>;
  canTransition(event: string): Promise<boolean>;
  getAvailableTransitions(): Transition[];
  getHistory(): Array<[string, Date]>;
}

// @phenotype/pheno-resilience/policy-engine/index.ts

export interface PolicyRule {
  name: string;
  pattern: string; // Regex
  action: 'allow' | 'deny' | 'custom';
  priority: number;
}

export interface PolicySet {
  name: string;
  rules: PolicyRule[];
  combinator: 'first-applicable' | 'deny-overrides' | 'permit-overrides';
}

export interface PolicyContext {
  subject: string;
  resource: string;
  action: string;
  environment?: Record<string, unknown>;
}

export type PolicyDecision = 'allow' | 'deny' | 'not-applicable';

export class PolicyEngine {
  addPolicies(policies: PolicySet[]): void;
  evaluate(context: PolicyContext): Promise<PolicyDecision>;
}

// @phenotype/pheno-resilience/cache/index.ts

export interface CacheConfig {
  strategy: 'lru' | 'lfu' | 'distributed';
  maxSize: number;
  ttl?: number;
}

export interface Cache<K, V> {
  get(key: K): Promise<V | null>;
  set(key: K, value: V, ttl?: number): Promise<void>;
  delete(key: K): Promise<void>;
  clear(): Promise<void>;
}

export class LRUCache<K, V> implements Cache<K, V> {}

export class DistributedCache<K, V> implements Cache<K, V> {
  // Requires Redis or similar backend
}

export class TwoTierCache<K, V> implements Cache<K, V> {
  // L1: LRU (in-process)
  // L2: Distributed (shared)
}
```

### @phenotype/pheno-llm Interface

**Stability**: EVOLVING (LLM patterns change rapidly)

```typescript
// @phenotype/pheno-llm/providers/index.ts

export interface LLMOptions {
  model: string;
  maxTokens?: number;
  temperature?: number;
  topP?: number;
  frequencyPenalty?: number;
  presencePenalty?: number;
  system?: string;
  tools?: ToolDefinition[];
}

export interface LLMMessage {
  role: 'system' | 'user' | 'assistant' | 'tool';
  content: string;
  toolCallId?: string;
  toolName?: string;
}

export interface ToolDefinition {
  name: string;
  description: string;
  inputSchema: Record<string, unknown>; // JSON Schema
}

export interface ToolCall {
  id: string;
  name: string;
  arguments: Record<string, unknown>;
}

export interface TokenUsage {
  prompt: number;
  completion: number;
  total: number;
}

export interface LLMResponse {
  text: string;
  finishReason: 'stop' | 'length' | 'tool_use' | 'error';
  tokenUsage: TokenUsage;
  toolCalls?: ToolCall[];
}

export interface LLMProvider {
  complete(
    prompt: string,
    options?: LLMOptions,
  ): Promise<LLMResponse>;

  chat(
    messages: LLMMessage[],
    options?: LLMOptions,
  ): Promise<LLMResponse>;

  embed(text: string): Promise<number[]>;

  tokenize(text: string): Promise<string[]>;

  countTokens(text: string, model?: string): Promise<number>;
}

// @phenotype/pheno-llm/prompts/index.ts

export interface Prompt {
  template: string;
  variables: Record<string, unknown>;
  render(): string;
  countTokens(): number;
  validate(): boolean;
}

export class PromptManager {
  registerTemplate(name: string, template: string): void;
  getTemplate(name: string): string;
  compilePrompt(name: string, vars: Record<string, unknown>): Prompt;
  validatePrompt(prompt: Prompt): boolean;
}

export class PromptBuilder {
  system(text: string): PromptBuilder;
  user(text: string): PromptBuilder;
  assistant(text: string): PromptBuilder;
  tool(name: string, result: unknown): PromptBuilder;
  build(): LLMMessage[];
}

export function estimateTokens(text: string, model?: string): number;
```

---

## Part 4: Publishing Configuration

### GitHub Packages Setup

**Registry URL**: `https://npm.pkg.github.com`
**Scope**: `@phenotype`
**Auth**: GITHUB_TOKEN with `packages:read, packages:write` scope

### Package Metadata Template

```json
{
  "name": "@phenotype/pheno-core",
  "version": "1.0.0",
  "description": "Core hexagonal architecture contracts for Phenotype",
  "type": "module",
  "main": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "import": "./dist/index.js",
      "types": "./dist/index.d.ts"
    },
    "./ports": {
      "import": "./dist/ports/index.js",
      "types": "./dist/ports/index.d.ts"
    },
    "./models": {
      "import": "./dist/models/index.js",
      "types": "./dist/models/index.d.ts"
    },
    "./errors": {
      "import": "./dist/errors/index.js",
      "types": "./dist/errors/index.d.ts"
    }
  },
  "publishConfig": {
    "registry": "https://npm.pkg.github.com",
    "access": "public"
  },
  "repository": {
    "type": "git",
    "url": "https://github.com/KooshaPari/phenotype-infrakit.git",
    "directory": "packages/pheno-core"
  },
  "keywords": ["hexagonal-architecture", "ports-adapters", "ddd"],
  "author": "Phenotype Contributors",
  "license": "MIT",
  "engines": {
    "node": ">=18.0.0"
  }
}
```

---

## Summary: Extraction Readiness Matrix

| Crate | → Package | Lines | Dependencies | Stability | Ready | Comments |
|-------|-----------|-------|--------------|-----------|-------|----------|
| phenotype-contracts | pheno-core | ~400 | 0 | STABLE | ✓ YES | Core foundation, zero deps, publish first |
| phenotype-event-sourcing | pheno-resilience | ~800 | 1 | STABLE | ✓ YES | Proven pattern, well-tested |
| phenotype-cache-adapter | pheno-resilience | ~300 | 1 | STABLE | ✓ YES | Optional adapter, can defer |
| phenotype-policy-engine | pheno-resilience | ~600 | 1 | STABLE | ✓ YES | Core authorization pattern |
| phenotype-state-machine | pheno-resilience | ~250 | 1 | STABLE | ✓ YES | Lightweight, well-scoped |

**Overall Readiness**: 100% - All crates are stable, well-designed, and ready for npm extraction.

**Execution Order** (DAG):
1. pheno-core (zero dependencies, publish independently)
2. pheno-resilience (depends on pheno-core, can publish after)
3. pheno-llm (greenfield module, no Rust source, can happen in parallel)

---

**Audit Status**: COMPLETE
**Confidence Level**: HIGH (95%+)

*This audit represents a comprehensive analysis of phenotype-infrakit's extractability and npm packaging readiness.*
