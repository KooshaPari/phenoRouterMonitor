"""Task data model with dependency support."""

import uuid
from dataclasses import dataclass, field
from datetime import UTC, datetime
from enum import StrEnum
from typing import Any


class TaskStatus(StrEnum):
    """Task execution status."""

    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    SKIPPED = "skipped"


@dataclass
class Task:
    """
    Task definition with dependencies.

    Tasks can depend on other tasks, forming a DAG that is resolved
    topologically before execution.

    Attributes:
        id: Unique task identifier
        description: Task description/prompt
        agent_id: Optional assigned agent ID
        dependencies: List of task IDs this task depends on
        status: Current execution status
        result: Task execution result
        error: Error message if failed
        metadata: Additional task metadata
    """

    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    description: str = ""
    agent_id: str | None = None
    dependencies: list[str] = field(default_factory=list)
    status: TaskStatus = TaskStatus.PENDING

    # Execution results
    result: Any | None = None
    error: str | None = None
    started_at: datetime | None = None
    completed_at: datetime | None = None

    # Metadata
    metadata: dict[str, Any] = field(default_factory=dict)
    created_at: datetime = field(default_factory=lambda: datetime.now(UTC))

    def add_dependency(self, task_id: str) -> None:
        """Add a dependency on another task."""
        if task_id not in self.dependencies:
            self.dependencies.append(task_id)

    def remove_dependency(self, task_id: str) -> None:
        """Remove a dependency."""
        if task_id in self.dependencies:
            self.dependencies.remove(task_id)

    def is_ready(self, completed_tasks: set[str]) -> bool:
        """Check if all dependencies are completed."""
        return all(dep_id in completed_tasks for dep_id in self.dependencies)

    def mark_running(self) -> None:
        """Mark task as running."""
        self.status = TaskStatus.RUNNING
        self.started_at = datetime.now(UTC)

    def mark_completed(self, result: Any = None) -> None:
        """Mark task as completed."""
        self.status = TaskStatus.COMPLETED
        self.result = result
        self.completed_at = datetime.now(UTC)

    def mark_failed(self, error: str) -> None:
        """Mark task as failed."""
        self.status = TaskStatus.FAILED
        self.error = error
        self.completed_at = datetime.now(UTC)
