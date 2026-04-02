import { Entity } from './entity';

export interface EntityRepository {
  create(entity: Entity): Promise<void>;
  findById(id: string): Promise<Entity | null>;
  update(entity: Entity): Promise<void>;
  delete(id: string): Promise<void>;
  list(): Promise<Entity[]>;
}
