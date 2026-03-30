// Hexagonal Architecture Ports (Interfaces)
// wraps: phenotype-contracts Rust crate port definitions

// ============================================================================
// INBOUND PORTS (Driving / Primary Ports)
// ============================================================================

/** Generic use case port for encapsulating business logic */
export interface UseCase<TInput, TOutput> {
  execute(input: TInput): Promise<TOutput>;
}

/** Handler for commands (state-changing operations) */
export interface CommandHandler<TCommand, TResult = void> {
  handle(command: TCommand): Promise<TResult>;
}

/** Handler for queries (read-only operations) */
export interface QueryHandler<TQuery, TResult> {
  handle(query: TQuery): Promise<TResult>;
}

/** Handler for domain events */
export interface EventHandler<T = unknown> {
  handle(event: DomainEvent<T>): Promise<void>;
}

// ============================================================================
// OUTBOUND PORTS (Driven / Secondary Ports)
// ============================================================================

/** Generic repository port for persistence */
export interface Repository<T, TId = string> {
  findById(id: TId): Promise<T | null>;
  save(entity: T): Promise<void>;
  delete(id: TId): Promise<void>;
  findAll(): Promise<T[]>;
}

/** Cache port for managing temporary state */
export interface CachePort<T> {
  get(key: string): Promise<T | null>;
  set(key: string, value: T, ttlMs?: number): Promise<void>;
  delete(key: string): Promise<void>;
  clear(): Promise<void>;
}

/** Event bus port for publishing and subscribing to domain events */
export interface EventBus {
  publish<T = unknown>(event: DomainEvent<T>): Promise<void>;
  subscribe<T = unknown>(
    eventType: string,
    handler: (event: DomainEvent<T>) => Promise<void>,
  ): void;
  unsubscribe(eventType: string, handler: Function): void;
}

/** Secret/credential management port */
export interface SecretPort {
  get(key: string): Promise<string | null>;
  set(key: string, value: string): Promise<void>;
  delete(key: string): Promise<void>;
}

/** Health check port for diagnostics */
export interface HealthChecker {
  check(): Promise<HealthStatus>;
}

export interface HealthStatus {
  status: 'healthy' | 'degraded' | 'unhealthy';
  checks: Record<string, CheckResult>;
}

export interface CheckResult {
  status: 'pass' | 'warn' | 'fail';
  message?: string;
  details?: Record<string, unknown>;
}

// ============================================================================
// DOMAIN EVENTS
// ============================================================================

/** Base domain event interface */
export interface DomainEvent<T = unknown> {
  readonly eventId: string;
  readonly eventType: string;
  readonly timestamp: Date;
  readonly payload: T;
  readonly metadata?: Record<string, unknown>;
  readonly aggregateId?: string;
  readonly version?: number;
}

/** Factory for creating domain events */
export interface EventFactory {
  create<T = unknown>(
    eventType: string,
    payload: T,
    metadata?: Record<string, unknown>,
  ): DomainEvent<T>;
}
