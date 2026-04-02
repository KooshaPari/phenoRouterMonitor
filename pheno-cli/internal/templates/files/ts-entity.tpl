import { v4 as uuidv4 } from 'uuid';

export class Entity {
  constructor(
    public readonly id: string = uuidv4(),
    public name: string,
    public description: string = '',
    public readonly createdAt: Date = new Date(),
    public updatedAt: Date = new Date()
  ) {}

  update(name: string, description: string): void {
    this.name = name;
    this.description = description;
    this.touch();
  }

  touch(): void {
    this.updatedAt = new Date();
  }

  toJSON() {
    return {
      id: this.id,
      name: this.name,
      description: this.description,
      createdAt: this.createdAt.toISOString(),
      updatedAt: this.updatedAt.toISOString(),
    };
  }
}

export class DomainError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'DomainError';
  }
}

export class EntityNotFoundError extends DomainError {
  constructor(id: string) {
    super(`Entity not found: ${id}`);
    this.name = 'EntityNotFoundError';
  }
}

export class InvalidInputError extends DomainError {
  constructor(message: string) {
    super(message);
    this.name = 'InvalidInputError';
  }
}
