// Canonical Error Types
// wraps: phenotype-error-core (Rust crate) — 5 canonical error types

/**
 * Base Phenotype error class
 * Consolidates 85+ error enums from across the codebase
 */
export class PhenotypeError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly cause?: Error,
  ) {
    super(message);
    this.name = 'PhenotypeError';
    Object.setPrototypeOf(this, PhenotypeError.prototype);
  }

  toJSON() {
    return {
      name: this.name,
      code: this.code,
      message: this.message,
      cause: this.cause?.message,
    };
  }
}

/**
 * Resource not found error (HTTP 404)
 */
export class NotFoundError extends PhenotypeError {
  constructor(entity: string, id: string) {
    super(`${entity} not found: ${id}`, 'NOT_FOUND');
    this.name = 'NotFoundError';
    Object.setPrototypeOf(this, NotFoundError.prototype);
  }
}

/**
 * Validation error (HTTP 400)
 * Used for input validation, schema validation, constraint violations
 */
export class ValidationError extends PhenotypeError {
  constructor(
    message: string,
    public readonly field?: string,
    public readonly constraints?: Record<string, string>,
  ) {
    super(message, 'VALIDATION_ERROR');
    this.name = 'ValidationError';
    Object.setPrototypeOf(this, ValidationError.prototype);
  }

  toJSON() {
    return {
      ...super.toJSON(),
      field: this.field,
      constraints: this.constraints,
    };
  }
}

/**
 * Conflict error (HTTP 409)
 * Used for duplicate resources, state conflicts, optimistic lock failures
 */
export class ConflictError extends PhenotypeError {
  constructor(
    message: string,
    public readonly existingId?: string,
  ) {
    super(message, 'CONFLICT');
    this.name = 'ConflictError';
    Object.setPrototypeOf(this, ConflictError.prototype);
  }

  toJSON() {
    return {
      ...super.toJSON(),
      existingId: this.existingId,
    };
  }
}

/**
 * Authorization error (HTTP 401 / 403)
 * Used for missing credentials, invalid tokens, insufficient permissions
 */
export class UnauthorizedError extends PhenotypeError {
  constructor(
    message = 'Unauthorized',
    public readonly reason?: 'missing_credentials' | 'invalid_token' | 'insufficient_permissions',
  ) {
    super(message, 'UNAUTHORIZED');
    this.name = 'UnauthorizedError';
    Object.setPrototypeOf(this, UnauthorizedError.prototype);
  }

  toJSON() {
    return {
      ...super.toJSON(),
      reason: this.reason,
    };
  }
}

/**
 * Serialization/deserialization error
 * Used for JSON parsing, YAML parsing, codec errors
 */
export class SerializationError extends PhenotypeError {
  constructor(
    message: string,
    public readonly format?: string,
  ) {
    super(message, 'SERIALIZATION_ERROR');
    this.name = 'SerializationError';
    Object.setPrototypeOf(this, SerializationError.prototype);
  }

  toJSON() {
    return {
      ...super.toJSON(),
      format: this.format,
    };
  }
}

/**
 * Type guard to check if an error is a PhenotypeError
 */
export function isPhenotypeError(error: unknown): error is PhenotypeError {
  return error instanceof PhenotypeError;
}

/**
 * Helper to wrap unknown errors
 */
export function wrap(error: unknown, context?: string): PhenotypeError {
  if (error instanceof PhenotypeError) {
    return error;
  }

  const message = error instanceof Error ? error.message : String(error);
  const fullMessage = context ? `${context}: ${message}` : message;

  return new PhenotypeError(
    fullMessage,
    'INTERNAL_ERROR',
    error instanceof Error ? error : undefined,
  );
}
