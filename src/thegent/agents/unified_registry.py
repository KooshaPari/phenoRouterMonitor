"""Unified Agent Registry API - Core Models and Service.

Consolidates agent management across the kush ecosystem.
Reference: docs/research/UNIFIED_AGENT_REGISTRY_API.md
"""

import logging
from datetime import datetime
from enum import StrEnum
from typing import Any

from pydantic import BaseModel, Field

logger = logging.getLogger(__name__)


class AgentStatus(StrEnum):
    """Agent status."""

    ACTIVE = "active"
    INACTIVE = "inactive"
    BUSY = "busy"
    ERROR = "error"
    MAINTENANCE = "maintenance"


class AgentCapability(StrEnum):
    """Agent capabilities."""

    CODE_GENERATION = "code_generation"
    CODE_REVIEW = "code_review"
    TESTING = "testing"
    DOCUMENTATION = "documentation"
    RESEARCH = "research"
    DEPLOYMENT = "deployment"
    MONITORING = "monitoring"
    SECURITY = "security"


class ProjectAssignment(BaseModel):
    """Project assignment."""

    project_id: str
    role: str = Field(..., description="primary, secondary, consultant")
    permissions: list[str] = Field(default_factory=list)
    assigned_at: datetime = Field(default_factory=datetime.now)
    last_active: datetime | None = None


class CollaborationRule(BaseModel):
    """Collaboration rules."""

    can_initiate_with: list[str] = Field(default_factory=list)
    must_consult_with: list[str] = Field(default_factory=list)
    ignore_agents: list[str] = Field(default_factory=list)
    auto_join_topics: list[str] = Field(default_factory=list)


class Availability(BaseModel):
    """Agent availability."""

    schedule: str | None = Field(None, description="Cron expression")
    timezone: str = "UTC"
    office_hours: dict[str, str] | None = None
    is_available: bool = True


class PerformanceMetrics(BaseModel):
    """Performance metrics."""

    total_tasks: int = 0
    completed_tasks: int = 0
    failed_tasks: int = 0
    average_response_time: float = 0.0
    success_rate: float = 0.0
    last_updated: datetime = Field(default_factory=datetime.now)


class Agent(BaseModel):
    """Unified agent model."""

    # Identity
    id: str = Field(..., description="Unique agent identifier")
    name: str
    description: str | None = None
    version: str = "1.0.0"

    # Capabilities
    capabilities: list[AgentCapability] = Field(default_factory=list)
    tools: list[str] = Field(default_factory=list)
    models: list[str] = Field(default_factory=list, description="Supported LLM models")

    # Configuration
    system_prompt: str | None = None
    temperature: float = 0.7
    max_tokens: int | None = None

    # Project assignments
    projects: list[ProjectAssignment] = Field(default_factory=list)

    # Collaboration
    collaboration_rules: CollaborationRule = Field(default_factory=CollaborationRule)

    # Availability
    availability: Availability = Field(default_factory=lambda: Availability(schedule=None))

    # State
    status: AgentStatus = AgentStatus.INACTIVE
    current_project: str | None = None
    last_active: datetime | None = None

    # Performance
    metrics: PerformanceMetrics = Field(default_factory=PerformanceMetrics)

    # Metadata
    metadata: dict[str, Any] = Field(default_factory=dict)
    created_at: datetime = Field(default_factory=datetime.now)
    updated_at: datetime = Field(default_factory=datetime.now)


class AgentRegistryService:
    """Service for managing the unified agent registry."""

    def __init__(self, storage_path: str | None = None) -> None:
        self.agents: dict[str, Agent] = {}
        # In a real impl, this would connect to a DB or load from file
        self.storage_path = storage_path

    def register_agent(self, agent: Agent) -> Agent:
        """Register a new agent."""
        self.agents[agent.id] = agent
        return agent

    def get_agent(self, agent_id: str) -> Agent | None:
        """Get agent by ID."""
        return self.agents.get(agent_id)

    def list_agents(
        self,
        status: AgentStatus | None = None,
        project_id: str | None = None,
        capability: AgentCapability | None = None,
    ) -> list[Agent]:
        """List agents with optional filtering."""
        result = list(self.agents.values())

        if status:
            result = [a for a in result if a.status == status]
        if project_id:
            result = [a for a in result if any(p.project_id == project_id for p in a.projects)]
        if capability:
            result = [a for a in result if capability in a.capabilities]

        return result

    def update_agent(self, agent_id: str, updates: dict[str, Any]) -> Agent | None:
        """Update agent metadata."""
        agent = self.get_agent(agent_id)
        if not agent:
            return None

        # Update fields
        data = agent.model_dump()
        data.update(updates)
        updated_agent = Agent(**data)
        updated_agent.updated_at = datetime.now()

        self.agents[agent_id] = updated_agent
        return updated_agent

    def delete_agent(self, agent_id: str) -> bool:
        """Delete agent."""
        if agent_id in self.agents:
            del self.agents[agent_id]
            return True
        return False

    def assign_to_project(self, agent_id: str, assignment: ProjectAssignment) -> Agent | None:
        """Assign an agent to a project."""
        agent = self.get_agent(agent_id)
        if not agent:
            return None

        # Check if already assigned
        for existing in agent.projects:
            if existing.project_id == assignment.project_id:
                existing.role = assignment.role
                existing.permissions = assignment.permissions
                existing.last_active = datetime.now()
                return agent

        agent.projects.append(assignment)
        agent.updated_at = datetime.now()
        return agent

    def update_collaboration_rules(self, agent_id: str, rules: CollaborationRule) -> Agent | None:
        """Update collaboration rules for an agent."""
        agent = self.get_agent(agent_id)
        if not agent:
            return None

        agent.collaboration_rules = rules
        agent.updated_at = datetime.now()
        return agent

    def update_metrics(self, agent_id: str, metrics_update: dict[str, Any]) -> PerformanceMetrics | None:
        """Update performance metrics for an agent."""
        agent = self.get_agent(agent_id)
        if not agent:
            return None

        data = agent.metrics.model_dump()
        data.update(metrics_update)
        agent.metrics = PerformanceMetrics(**data)
        agent.metrics.last_updated = datetime.now()
        agent.updated_at = datetime.now()

        return agent.metrics

    def discover_best_agent(
        self, task_description: str, required_capabilities: list[AgentCapability], project_id: str | None = None
    ) -> Agent | None:
        """Discovery logic to find best agent for task."""
        candidates = self.list_agents(status=AgentStatus.ACTIVE, project_id=project_id)

        # Filter by capabilities
        candidates = [a for a in candidates if all(cap in a.capabilities for cap in required_capabilities)]

        if not candidates:
            return None

        # Sort by success rate and response time
        candidates.sort(key=lambda a: (a.metrics.success_rate, -a.metrics.average_response_time), reverse=True)

        return candidates[0]
