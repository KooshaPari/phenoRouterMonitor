"""In-memory repository implementation."""

from typing import Dict, List, Optional
from {{.RepoName}}.domain.entities import Entity
from {{.RepoName}}.domain.ports import EntityRepository


class InMemoryEntityRepository(EntityRepository):
    """In-memory implementation of entity repository."""
    
    def __init__(self):
        self._entities: Dict[str, Entity] = {}
    
    def create(self, entity: Entity) -> None:
        """Create a new entity."""
        self._entities[entity.id] = entity
    
    def get_by_id(self, entity_id: str) -> Optional[Entity]:
        """Get entity by ID."""
        return self._entities.get(entity_id)
    
    def update(self, entity: Entity) -> None:
        """Update an existing entity."""
        self._entities[entity.id] = entity
    
    def delete(self, entity_id: str) -> None:
        """Delete an entity by ID."""
        if entity_id in self._entities:
            del self._entities[entity_id]
    
    def list_all(self) -> List[Entity]:
        """List all entities."""
        return list(self._entities.values())
