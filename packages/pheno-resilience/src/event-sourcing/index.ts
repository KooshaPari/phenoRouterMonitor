// Event Sourcing Module
// wraps: phenotype-event-sourcing (Rust crate)

import type { DomainEvent } from '../../../pheno-core/src/ports/index';

// ============================================================================
// CORE INTERFACES
// ============================================================================

/**
 * Event store interface for persisting and retrieving domain events
 * Implements event sourcing pattern with snapshots and aggregates
 */
export interface EventStore {
  /**
   * Append an event to the event stream
   */
  append<T = unknown>(aggregateId: string, event: DomainEvent<T>): Promise<void>;

  /**
   * Get all events for an aggregate
   */
  getEvents(aggregateId: string, fromVersion?: number): Promise<DomainEvent[]>;

  /**
   * Get events in a range (for replaying)
   */
  getEventRange(
    aggregateId: string,
    fromVersion: number,
    toVersion: number,
  ): Promise<DomainEvent[]>;

  /**
   * Save a snapshot of aggregate state
   */
  saveSnapshot(aggregateId: string, snapshot: Snapshot): Promise<void>;

  /**
   * Get the latest snapshot for an aggregate
   */
  getLatestSnapshot(aggregateId: string): Promise<Snapshot | null>;

  /**
   * Subscribe to events of a specific type
   */
  subscribe(eventType: string, handler: (event: DomainEvent) => Promise<void>): void;
}

/**
 * Snapshot of aggregate state at a specific version
 */
export interface Snapshot {
  readonly aggregateId: string;
  readonly version: number;
  readonly state: Record<string, unknown>;
  readonly timestamp: Date;
}

/**
 * Event projection for rebuilding read models
 */
export interface EventProjection {
  /**
   * Replay events to build/rebuild the projection
   */
  replay(events: DomainEvent[]): Promise<void>;

  /**
   * Handle a single event
   */
  handle<T = unknown>(event: DomainEvent<T>): Promise<void>;

  /**
   * Get the current state of the projection
   */
  getState(): Promise<Record<string, unknown>>;
}

/**
 * Configuration for event sourcing
 */
export interface EventSourcingConfig {
  snapshotThreshold: number; // Create snapshot every N events
  retentionDays?: number; // Keep events for N days
  enableCompression?: boolean; // Enable event compression
  maxBatchSize?: number; // Max events per batch operation
}

// ============================================================================
// MEMORY IMPLEMENTATION (For testing/development)
// ============================================================================

/**
 * In-memory event store implementation
 * Suitable for testing and local development
 */
export class InMemoryEventStore implements EventStore {
  private events: Map<string, DomainEvent[]> = new Map();
  private snapshots: Map<string, Snapshot> = new Map();
  private subscribers: Map<string, Set<(event: DomainEvent) => Promise<void>>> = new Map();

  async append<T = unknown>(aggregateId: string, event: DomainEvent<T>): Promise<void> {
    if (!this.events.has(aggregateId)) {
      this.events.set(aggregateId, []);
    }
    this.events.get(aggregateId)!.push(event);

    // Notify subscribers
    const handlers = this.subscribers.get(event.eventType);
    if (handlers) {
      for (const handler of handlers) {
        await handler(event);
      }
    }
  }

  async getEvents(
    aggregateId: string,
    fromVersion?: number,
  ): Promise<DomainEvent[]> {
    const events = this.events.get(aggregateId) ?? [];
    return fromVersion
      ? events.filter((e) => e.version! >= fromVersion)
      : events;
  }

  async getEventRange(
    aggregateId: string,
    fromVersion: number,
    toVersion: number,
  ): Promise<DomainEvent[]> {
    const events = this.events.get(aggregateId) ?? [];
    return events.filter((e) => e.version! >= fromVersion && e.version! <= toVersion);
  }

  async saveSnapshot(aggregateId: string, snapshot: Snapshot): Promise<void> {
    this.snapshots.set(aggregateId, snapshot);
  }

  async getLatestSnapshot(aggregateId: string): Promise<Snapshot | null> {
    return this.snapshots.get(aggregateId) ?? null;
  }

  subscribe(eventType: string, handler: (event: DomainEvent) => Promise<void>): void {
    if (!this.subscribers.has(eventType)) {
      this.subscribers.set(eventType, new Set());
    }
    this.subscribers.get(eventType)!.add(handler);
  }
}

// ============================================================================
// UTILITIES
// ============================================================================

/**
 * Replay events to reconstruct aggregate state
 */
export async function replayEvents<T>(
  events: DomainEvent[],
  initialState: T,
  handler: (state: T, event: DomainEvent) => T,
): Promise<T> {
  return events.reduce((state, event) => handler(state, event), initialState);
}
