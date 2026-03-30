// Domain Model Base Types
// wraps: phenotype-contracts Rust crate model definitions

import type { DomainEvent } from '../ports/index';

// ============================================================================
// ENTITY TYPES
// ============================================================================

/** Base entity with identity and timestamps */
export interface Entity<TId = string> {
  readonly id: TId;
  readonly createdAt: Date;
  readonly updatedAt: Date;
}

/** Aggregate root with event sourcing support */
export interface AggregateRoot<TId = string> extends Entity<TId> {
  readonly version: number;
  readonly domainEvents: DomainEvent[];
  clearEvents(): void;
  addEvent<T = unknown>(event: DomainEvent<T>): void;
}

// ============================================================================
// VALUE OBJECTS
// ============================================================================

/** Immutable value object base type */
export interface ValueObject<T> {
  readonly value: T;
  equals(other: ValueObject<T>): boolean;
  toString(): string;
}

/** Generic value object implementation */
export class ValueObjectImpl<T> implements ValueObject<T> {
  constructor(readonly value: T) {}

  equals(other: ValueObject<T>): boolean {
    return JSON.stringify(this.value) === JSON.stringify(other.value);
  }

  toString(): string {
    return `ValueObject(${JSON.stringify(this.value)})`;
  }
}

// ============================================================================
// RESULT TYPES (Railway-oriented programming)
// ============================================================================

export type Result<T, E = Error> = Ok<T> | Err<E>;

export class Ok<T> {
  readonly kind = 'ok' as const;
  constructor(readonly value: T) {}

  isOk(): this is Ok<T> {
    return true;
  }

  isErr(): this is Err<never> {
    return false;
  }

  map<U>(fn: (value: T) => U): Result<U> {
    return new Ok(fn(this.value));
  }

  flatMap<U, E>(fn: (value: T) => Result<U, E>): Result<U, E> {
    return fn(this.value);
  }
}

export class Err<E> {
  readonly kind = 'err' as const;
  constructor(readonly error: E) {}

  isOk(): this is Ok<never> {
    return false;
  }

  isErr(): this is Err<E> {
    return true;
  }

  map<U>(_fn: (value: never) => U): Result<never, E> {
    return (this as unknown) as Result<never, E>;
  }

  flatMap<U, F>(_fn: (value: never) => Result<U, F>): Result<U, E | F> {
    return (this as unknown) as Result<U, E | F>;
  }
}

// ============================================================================
// OPTION TYPES
// ============================================================================

export type Option<T> = Some<T> | None;

export class Some<T> {
  readonly kind = 'some' as const;
  constructor(readonly value: T) {}

  isSome(): this is Some<T> {
    return true;
  }

  isNone(): this is None {
    return false;
  }

  map<U>(fn: (value: T) => U): Option<U> {
    return new Some(fn(this.value));
  }

  flatMap<U>(fn: (value: T) => Option<U>): Option<U> {
    return fn(this.value);
  }

  getOrElse(defaultValue: T): T {
    return this.value;
  }
}

export class None {
  readonly kind = 'none' as const;

  isSome(): this is Some<never> {
    return false;
  }

  isNone(): this is None {
    return true;
  }

  map<U>(_fn: (value: never) => U): Option<never> {
    return this as Option<never>;
  }

  flatMap<U>(_fn: (value: never) => Option<U>): Option<U> {
    return this as Option<U>;
  }

  getOrElse<T>(defaultValue: T): T {
    return defaultValue;
  }
}

export const none = new None();

export function some<T>(value: T): Option<T> {
  return new Some(value);
}

export function option<T>(value: T | null | undefined): Option<T> {
  return value == null ? (none as Option<T>) : new Some(value);
}
