import { Entity } from '../domain/entity';
import { EntityRepository } from '../domain/ports';

export class InMemoryEntityRepository implements EntityRepository {
  private entities: Map<string, Entity> = new Map();

  async create(entity: Entity): Promise<void> {
    this.entities.set(entity.id, entity);
  }

  async findById(id: string): Promise<Entity | null> {
    return this.entities.get(id) || null;
  }

  async update(entity: Entity): Promise<void> {
    this.entities.set(entity.id, entity);
  }

  async delete(id: string): Promise<void> {
    this.entities.delete(id);
  }

  async list(): Promise<Entity[]> {
    return Array.from(this.entities.values());
  }
}
