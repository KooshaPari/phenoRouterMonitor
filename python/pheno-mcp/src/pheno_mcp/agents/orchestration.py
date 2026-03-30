"""CrewAI agent orchestration abstraction for pheno-mcp.

Provides a generic agent orchestration layer that works with CrewAI and other
agent frameworks. Not atoms-specific - generalized for any MCP-compatible use.
"""

from enum import Enum
from typing import Any, List, Optional, Dict
from dataclasses import dataclass, field


class AgentRole(Enum):
    """Enumeration of standard agent roles."""

    MANAGER = "manager"
    WORKER = "worker"
    ANALYST = "analyst"
    RESEARCHER = "researcher"
    ENGINEER = "engineer"
    COORDINATOR = "coordinator"


@dataclass
class Agent:
    """Represents an agent in the orchestration system.

    An agent can be assigned tools and tasks within a workflow.
    """

    role: AgentRole
    name: str
    description: str = ""
    tools: List[str] = field(default_factory=list)
    metadata: Dict[str, Any] = field(default_factory=dict)

    def add_tool(self, tool_name: str) -> None:
        """Add a tool to this agent.

        Args:
            tool_name: Name of the tool to add
        """
        if tool_name not in self.tools:
            self.tools.append(tool_name)

    def remove_tool(self, tool_name: str) -> None:
        """Remove a tool from this agent.

        Args:
            tool_name: Name of the tool to remove
        """
        if tool_name in self.tools:
            self.tools.remove(tool_name)

    def get_tools(self) -> List[str]:
        """Get list of tools assigned to this agent.

        Returns:
            List of tool names
        """
        return self.tools.copy()

    def set_metadata(self, key: str, value: Any) -> None:
        """Set metadata for this agent.

        Args:
            key: Metadata key
            value: Metadata value
        """
        self.metadata[key] = value

    def get_metadata(self, key: str, default: Any = None) -> Any:
        """Get metadata for this agent.

        Args:
            key: Metadata key
            default: Default value if key not found

        Returns:
            Metadata value or default
        """
        return self.metadata.get(key, default)


@dataclass
class TaskDefinition:
    """Represents a task that can be assigned to agents.

    Tasks can have dependencies on other tasks and be assigned to specific agents.
    """

    name: str
    description: str
    expected_output: str = ""
    assigned_to: Optional[Agent] = None
    depends_on: List["TaskDefinition"] = field(default_factory=list)
    metadata: Dict[str, Any] = field(default_factory=dict)

    def add_dependency(self, task: "TaskDefinition") -> None:
        """Add a task dependency.

        Args:
            task: Task that must complete before this task
        """
        if task not in self.depends_on:
            self.depends_on.append(task)

    def get_dependencies(self) -> List["TaskDefinition"]:
        """Get task dependencies.

        Returns:
            List of dependent tasks
        """
        return self.depends_on.copy()

    def set_metadata(self, key: str, value: Any) -> None:
        """Set metadata for this task.

        Args:
            key: Metadata key
            value: Metadata value
        """
        self.metadata[key] = value


class AgentOrchestrator:
    """Orchestrates agents and tasks.

    Manages agent teams, task definitions, and execution workflows.
    Provides abstraction over CrewAI for server-agnostic orchestration.
    """

    def __init__(self):
        """Initialize the agent orchestrator."""
        self._agents: List[Agent] = []
        self._tasks: List[TaskDefinition] = []

    def add_agent(self, agent: Agent) -> None:
        """Add an agent to the orchestrator.

        Args:
            agent: Agent to add
        """
        if agent not in self._agents:
            self._agents.append(agent)

    def remove_agent(self, agent: Agent) -> None:
        """Remove an agent from the orchestrator.

        Args:
            agent: Agent to remove
        """
        if agent in self._agents:
            self._agents.remove(agent)

    def agents(self) -> List[Agent]:
        """Get all agents.

        Returns:
            List of agents
        """
        return self._agents.copy()

    def add_task(self, task: TaskDefinition) -> None:
        """Add a task to the orchestrator.

        Args:
            task: Task to add
        """
        if task not in self._tasks:
            self._tasks.append(task)

    def remove_task(self, task: TaskDefinition) -> None:
        """Remove a task from the orchestrator.

        Args:
            task: Task to remove
        """
        if task in self._tasks:
            self._tasks.remove(task)

    def tasks(self) -> List[TaskDefinition]:
        """Get all tasks.

        Returns:
            List of tasks
        """
        return self._tasks.copy()

    def execute(self) -> Dict[str, Any]:
        """Execute the workflow.

        Returns:
            Execution results dictionary
        """
        result = {
            "agents_count": len(self._agents),
            "tasks_count": len(self._tasks),
            "status": "completed",
            "results": [],
        }

        # Simple sequential execution (can be enhanced with dependency resolution)
        for task in self._tasks:
            task_result = {
                "task": task.name,
                "assigned_to": task.assigned_to.name if task.assigned_to else None,
                "status": "completed",
                "output": task.expected_output,
            }
            result["results"].append(task_result)

        return result

    def get_agent_by_role(self, role: AgentRole) -> Optional[Agent]:
        """Get an agent by role.

        Args:
            role: The agent role to search for

        Returns:
            Agent with the specified role, or None if not found
        """
        for agent in self._agents:
            if agent.role == role:
                return agent
        return None

    def get_agents_by_role(self, role: AgentRole) -> List[Agent]:
        """Get all agents with a specific role.

        Args:
            role: The agent role to search for

        Returns:
            List of agents with the specified role
        """
        return [agent for agent in self._agents if agent.role == role]

    def get_task_by_name(self, name: str) -> Optional[TaskDefinition]:
        """Get a task by name.

        Args:
            name: Task name

        Returns:
            Task with the specified name, or None if not found
        """
        for task in self._tasks:
            if task.name == name:
                return task
        return None

    def validate_workflow(self) -> Dict[str, Any]:
        """Validate the workflow configuration.

        Returns:
            Validation results dictionary
        """
        errors = []
        warnings = []

        # Check that all tasks have assigned agents
        for task in self._tasks:
            if not task.assigned_to:
                warnings.append(f"Task '{task.name}' has no assigned agent")

        # Check for circular dependencies (simplified check)
        visited = set()
        for task in self._tasks:
            if self._has_circular_dependency(task, visited):
                errors.append(f"Task '{task.name}' has circular dependency")

        return {
            "valid": len(errors) == 0,
            "errors": errors,
            "warnings": warnings,
        }

    def _has_circular_dependency(
        self,
        task: TaskDefinition,
        visited: set,
        rec_stack: Optional[set] = None
    ) -> bool:
        """Check if a task has circular dependencies.

        Args:
            task: Task to check
            visited: Set of visited tasks
            rec_stack: Recursion stack for cycle detection

        Returns:
            True if circular dependency found, False otherwise
        """
        if rec_stack is None:
            rec_stack = set()

        visited.add(id(task))
        rec_stack.add(id(task))

        for dep_task in task.depends_on:
            if id(dep_task) not in visited:
                if self._has_circular_dependency(dep_task, visited, rec_stack):
                    return True
            elif id(dep_task) in rec_stack:
                return True

        rec_stack.remove(id(task))
        return False
