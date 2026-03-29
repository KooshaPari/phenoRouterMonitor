from dataclasses import dataclass, field
from enum import StrEnum
from typing import Any


class TaskCategory(StrEnum):
    """Task complexity classification."""

    FAST = "FAST"
    NORMAL = "NORMAL"
    COMPLEX = "COMPLEX"
    HIGH_COMPLEX = "HIGH_COMPLEX"


@dataclass
class TaskMetadata:
    """Routing and classification metadata for a task."""

    category: TaskCategory
    complexity_score: float = 0.0
    estimated_tokens: int = 0
    estimated_cost: float = 0.0
    estimated_duration_s: float = 0.0
    reasoning: str = ""
    signals: dict[str, Any] = field(default_factory=dict)
    resolved_provider: str | None = None
    resolved_model_alias: str | None = None


@dataclass
class RoutingConstraint:
    """A constraint for routing a task."""

    name: str
    target_value: Any
    actual_value: Any
    passed: bool
    message: str = ""
