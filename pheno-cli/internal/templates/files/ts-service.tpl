import { Entity, EntityNotFoundError, InvalidInputError } from '../domain/entity';
import { EntityRepository } from '../domain/ports';

export interface CreateEntityInput {
  name: string;
  description?: string;
}

export interface UpdateEntityInput {
  name: string;
  description?: string;
}

export class EntityService {
  constructor(private readonly repository: EntityRepository) {}

  async create(input: CreateEntityInput): Promise<Entity> {
    if (!input.name) {
      throw new InvalidInputError('Name is required');
    }

    const entity = new Entity(undefined, input.name, input.description);
    await this.repository.create(entity);
    return entity;
  }

  async getById(id: string): Promise<Entity> {
    const entity = await this.repository.findById(id);
    if (!entity) {
      throw new EntityNotFoundError(id);
    }
    return entity;
  }

  async update(id: string, input: UpdateEntityInput): Promise<Entity> {
    const entity = await this.getById(id);
    entity.update(input.name, input.description || '');
    await this.repository.update(entity);
    return entity;
  }

  async delete(id: string): Promise<void> {
    await this.repository.delete(id);
  }

  async list(): Promise<Entity[]> {
    return this.repository.list();
  }
}
