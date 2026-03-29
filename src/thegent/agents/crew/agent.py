"""Crew agent representation."""

import uuid
from dataclasses import dataclass, field
from typing import Any


@dataclass
class CrewAgent:
    """
    Agent representation in a crew.

    Attributes:
        id: Unique agent identifier
        role: Agent role (e.g., "planner", "researcher", "coder", "reviewer")
        name: Agent name
        description: Agent description
        capabilities: List of capabilities/tools
        model: Model to use (optional)
        config: Agent-specific configuration
    """

    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    role: str = ""
    name: str = ""
    description: str = ""
    capabilities: list[str] = field(default_factory=list)
    model: str | None = None
    config: dict[str, Any] = field(default_factory=dict)

    def __post_init__(self) -> None:
        """Set default name if not provided."""
        if not self.name:
            self.name = self.role or f"Agent-{self.id[:8]}"

    def can_handle_task(self, task_description: str) -> bool:
        """
        Check if agent can handle a task based on role/capabilities.

        Simple matching: checks if task description contains role keywords
        or capability keywords.
        """
        task_lower = task_description.lower()
        role_lower = self.role.lower()

        # Check role match
        if role_lower in task_lower or task_lower in role_lower:
            return True

        # Check capability match
        return any(capability.lower() in task_lower for capability in self.capabilities)
