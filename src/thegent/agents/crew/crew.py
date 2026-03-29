"""Crew data model - orchestrates agents and tasks."""

import uuid
from dataclasses import dataclass, field
from datetime import UTC, datetime
from enum import StrEnum
from typing import Any


class ExecutionMode(StrEnum):
    """Crew execution modes."""

    SEQUENTIAL = "sequential"
    HIERARCHICAL = "hierarchical"
    CUSTOM = "custom"


@dataclass
class Crew:
    """
    Orchestrates agents and tasks for complex problem solving.

    A crew manages a collection of agents and their tasks, coordinating
    their execution to achieve a goal.

    Attributes:
        id: Unique identifier
        name: Human-readable name
        description: What this crew does
        agents: List of agent IDs or Agent objects
        tasks: List of task IDs or Task objects
        execution_mode: How to execute (sequential, hierarchical, custom)
        config: Mode-specific configuration
        verbose: Enable detailed logging
        memory: Whether to use memory/history
        status: Current execution status
        execution_count: Number of times executed
        total_tokens_used: Total LLM tokens consumed
        total_cost_usd: Total cost in USD
        created_by: User/system that created this
        organization_id: Organization this belongs to
    """

    # Identity
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    name: str = ""
    description: str = ""
    version: str = "0.1.0"

    # Composition
    agents: list[Any] = field(default_factory=list)
    tasks: list[Any] = field(default_factory=list)

    # Execution
    execution_mode: ExecutionMode = ExecutionMode.SEQUENTIAL
    config: dict[str, Any] = field(default_factory=dict)
    verbose: bool = False
    memory: bool = True

    # Status
    status: str = "idle"  # idle, running, completed, error
    execution_count: int = 0
    total_tokens_used: int = 0
    total_cost_usd: float = 0.0

    # Relationships
    created_by: str | None = None
    organization_id: str | None = None

    # Metadata
    created_at: datetime = field(default_factory=lambda: datetime.now(UTC))
    last_executed_at: datetime | None = None
    tags: list[str] = field(default_factory=list)
    metadata: dict[str, Any] = field(default_factory=dict)

    def __post_init__(self) -> None:
        """Validate crew configuration after initialization."""
        if not self.name:
            self.name = f"Crew-{self.id[:8]}"

    def add_agent(self, agent: Any) -> None:
        """Add agent to crew."""
        if agent not in self.agents:
            self.agents.append(agent)

    def add_task(self, task: Any) -> None:
        """Add task to crew."""
        if task not in self.tasks:
            self.tasks.append(task)

    def get_agent_by_id(self, agent_id: str) -> Any | None:
        """Get agent by ID."""
        for agent in self.agents:
            if hasattr(agent, "id") and agent.id == agent_id:
                return agent
            if hasattr(agent, "agent_id") and agent.agent_id == agent_id:
                return agent
        return None

    def get_task_by_id(self, task_id: str) -> Any | None:
        """Get task by ID."""
        for task in self.tasks:
            if hasattr(task, "id") and task.id == task_id:
                return task
            if hasattr(task, "task_id") and task.task_id == task_id:
                return task
        return None
