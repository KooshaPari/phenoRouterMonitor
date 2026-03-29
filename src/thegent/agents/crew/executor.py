"""Task and Crew executors for orchestration."""

from collections.abc import Callable
from dataclasses import dataclass
from datetime import UTC, datetime
from typing import Any

from .agent import CrewAgent
from .crew import Crew, ExecutionMode
from .task import Task, TaskStatus


@dataclass
class ExecutionResult:
    """Result of task execution."""

    task_id: str
    success: bool
    result: Any = None
    error: str | None = None
    duration_seconds: float = 0.0
    tokens_used: int = 0
    cost_usd: float = 0.0


class TaskExecutor:
    """
    Executes tasks with dependency resolution.

    Handles:
    - Topological sorting of tasks
    - Dependency resolution
    - Task execution via agent_executor callback
    - Result aggregation
    """

    def __init__(
        self,
        max_retries: int = 2,
        timeout_seconds: int = 300,
        agent_executor: Callable[[str, str, dict[str, Any]], ExecutionResult] | None = None,
    ) -> None:
        """
        Initialize TaskExecutor.

        Args:
            max_retries: Maximum retries per task
            timeout_seconds: Timeout for task execution
            agent_executor: Callback function (agent_id, prompt, context) -> ExecutionResult
        """
        self.max_retries = max_retries
        self.timeout_seconds = timeout_seconds
        self.agent_executor = agent_executor
        self._task_results: dict[str, ExecutionResult] = {}

    def resolve_dependencies(self, tasks: list[Task]) -> list[Task]:
        """
        Resolve task dependencies using topological sort.

        Returns tasks in execution order (dependencies first).
        """
        # Build dependency graph
        task_map = {task.id: task for task in tasks}
        in_degree = {task.id: len(task.dependencies) for task in tasks}
        graph: dict[str, list[str]] = {task.id: [] for task in tasks}

        for task in tasks:
            for dep_id in task.dependencies:
                if dep_id in graph:
                    graph[dep_id].append(task.id)

        # Topological sort (Kahn's algorithm)
        queue = [task_id for task_id, degree in in_degree.items() if degree == 0]
        result = []

        while queue:
            task_id = queue.pop(0)
            result.append(task_map[task_id])

            for dependent_id in graph[task_id]:
                in_degree[dependent_id] -= 1
                if in_degree[dependent_id] == 0:
                    queue.append(dependent_id)

        # Check for cycles
        if len(result) != len(tasks):
            raise ValueError("Circular dependency detected in tasks")

        return result

    def get_task_input(self, task: Task, completed_tasks: dict[str, ExecutionResult]) -> dict[str, Any]:
        """
        Get input for task based on dependency results.

        Args:
            task: Task to get input for
            completed_tasks: Map of task_id -> ExecutionResult

        Returns:
            Context dictionary with dependency results
        """
        context: dict[str, Any] = {
            "task_id": task.id,
            "description": task.description,
            "dependencies": {},
        }

        # Add results from dependencies
        for dep_id in task.dependencies:
            if dep_id in completed_tasks:
                dep_result = completed_tasks[dep_id]
                context["dependencies"][dep_id] = {
                    "result": dep_result.result,
                    "success": dep_result.success,
                }

        return context

    def execute_task(
        self,
        task: Task,
        agent_id: str,
        context: dict[str, Any] | None = None,
    ) -> ExecutionResult:
        """
        Execute a single task.

        Args:
            task: Task to execute
            agent_id: Agent ID to execute task
            context: Optional context from dependencies

        Returns:
            ExecutionResult
        """
        if not self.agent_executor:
            raise ValueError("agent_executor callback not set")

        task.mark_running()
        start_time = datetime.now(UTC)

        try:
            # Build prompt with context
            prompt = task.description
            if context and context.get("dependencies"):
                prompt += "\n\nContext from previous tasks:\n"
                for dep_id, dep_data in context["dependencies"].items():
                    prompt += f"- {dep_id}: {dep_data.get('result', 'N/A')}\n"

            # Execute via agent_executor
            result = self.agent_executor(agent_id, prompt, context or {})

            # Update task status
            if result.success:
                task.mark_completed(result.result)
            else:
                task.mark_failed(result.error or "Task execution failed")

            duration = (datetime.now(UTC) - start_time).total_seconds()
            result.duration_seconds = duration

            self._task_results[task.id] = result
            return result

        except Exception as e:
            task.mark_failed(str(e))
            duration = (datetime.now(UTC) - start_time).total_seconds()
            return ExecutionResult(
                task_id=task.id,
                success=False,
                error=str(e),
                duration_seconds=duration,
            )

    def execute_all(
        self,
        tasks: list[Task],
        task_assignments: dict[str, str],  # task_id -> agent_id
    ) -> dict[str, ExecutionResult]:
        """
        Execute all tasks respecting dependencies.

        Args:
            tasks: List of tasks to execute
            task_assignments: Map of task_id -> agent_id

        Returns:
            Map of task_id -> ExecutionResult
        """
        # Resolve dependencies
        ordered_tasks = self.resolve_dependencies(tasks)
        completed_results: dict[str, ExecutionResult] = {}

        for task in ordered_tasks:
            # Get agent assignment
            agent_id = task_assignments.get(task.id)
            if not agent_id:
                # Skip unassigned tasks
                task.status = TaskStatus.SKIPPED
                continue

            # Get context from dependencies
            context = self.get_task_input(task, completed_results)

            # Execute task
            result = self.execute_task(task, agent_id, context)
            completed_results[task.id] = result

            # Stop on failure if configured
            if not result.success and not self.max_retries:
                break

        return completed_results


class AgentAssigner:
    """Base class for agent assignment strategies."""

    def assign(self, tasks: list[Task], agents: list[CrewAgent]) -> dict[str, str]:
        """
        Assign tasks to agents.

        Args:
            tasks: List of tasks
            agents: List of available agents

        Returns:
            Map of task_id -> agent_id
        """
        raise TypeError(
            f"{self.__class__.__name__}.assign() is abstract and must be implemented by"
            " a concrete AgentAssigner subclass"
        )


class RoundRobinAssigner(AgentAssigner):
    """Round-robin task assignment."""

    def assign(self, tasks: list[Task], agents: list[CrewAgent]) -> dict[str, str]:
        """Assign tasks round-robin."""
        if not agents:
            return {}

        assignments: dict[str, str] = {}
        agent_index = 0

        for task in tasks:
            assignments[task.id] = agents[agent_index].id
            agent_index = (agent_index + 1) % len(agents)

        return assignments


class SkillBasedAssigner(AgentAssigner):
    """Assign tasks based on agent skills/role."""

    def assign(self, tasks: list[Task], agents: list[CrewAgent]) -> dict[str, str]:
        """Assign tasks to agents based on role/capability matching."""
        assignments: dict[str, str] = {}

        for task in tasks:
            # Find best matching agent
            best_agent = None
            best_score = 0

            for agent in agents:
                if agent.can_handle_task(task.description):
                    # Simple scoring: role match = 2, capability match = 1
                    score = 2 if agent.role.lower() in task.description.lower() else 1
                    if score > best_score:
                        best_score = score
                        best_agent = agent

            if best_agent:
                assignments[task.id] = best_agent.id
            elif agents:
                # Fallback to first agent
                assignments[task.id] = agents[0].id

        return assignments


class HierarchicalAssigner(AgentAssigner):
    """Assign tasks hierarchically (managers/leads get priority tasks)."""

    def assign(self, tasks: list[Task], agents: list[CrewAgent]) -> dict[str, str]:
        """Assign priority tasks to managers/leads, rest to workers."""
        # Sort agents: managers/leads first
        managers = [a for a in agents if "manager" in a.role.lower() or "lead" in a.role.lower()]
        workers = [a for a in agents if a not in managers]

        # If no managers, use all agents as workers
        if not managers:
            managers = workers[:1] if workers else []
            workers = workers[1:] if len(workers) > 1 else workers

        assignments: dict[str, str] = {}
        manager_index = 0
        worker_index = 0

        # Assign first N tasks to managers
        num_manager_tasks = min(len(tasks), len(managers) * 2) if managers else 0

        for i, task in enumerate(tasks):
            if i < num_manager_tasks and managers:
                assignments[task.id] = managers[manager_index % len(managers)].id
                manager_index += 1
            elif workers:
                assignments[task.id] = workers[worker_index % len(workers)].id
                worker_index += 1
            elif managers:
                # Fallback to managers if no workers
                assignments[task.id] = managers[manager_index % len(managers)].id
                manager_index += 1

        return assignments


class CrewExecutor:
    """
    Orchestrates agent-task execution in crews.

    Manages:
    - Task-to-agent assignment
    - Multi-agent coordination
    - Result consolidation
    - Execution orchestration
    """

    def __init__(
        self,
        crew: Crew,
        task_executor: TaskExecutor | None = None,
        agent_assigner: AgentAssigner | None = None,
    ) -> None:
        """
        Initialize CrewExecutor.

        Args:
            crew: Crew to execute
            task_executor: Optional TaskExecutor (creates default if None)
            agent_assigner: Optional AgentAssigner (creates based on execution_mode if None)
        """
        self.crew = crew
        self.task_executor = task_executor or TaskExecutor()
        self.agent_assigner = agent_assigner or self._create_assigner()
        self.assignments: dict[str, str] = {}

    def _create_assigner(self) -> AgentAssigner:
        """Create agent assigner based on crew execution mode."""
        if self.crew.execution_mode == ExecutionMode.HIERARCHICAL:
            return HierarchicalAssigner()
        if self.crew.execution_mode == ExecutionMode.CUSTOM:
            # Custom assigner from config
            assigner_class = self.crew.config.get("assigner_class")
            if assigner_class:
                return assigner_class()
            return SkillBasedAssigner()
        # Sequential mode: skill-based by default
        return SkillBasedAssigner()

    def assign_tasks_to_agents(self) -> dict[str, str]:
        """Assign tasks to agents based on execution mode."""
        self.assignments = self.agent_assigner.assign(self.crew.tasks, self.crew.agents)
        return self.assignments

    def execute(self) -> dict[str, ExecutionResult]:
        """
        Execute crew tasks.

        Returns:
            Map of task_id -> ExecutionResult
        """
        # Update crew status
        self.crew.status = "running"
        self.crew.execution_count += 1
        self.crew.last_executed_at = datetime.now(UTC)

        # Assign tasks to agents
        self.assign_tasks_to_agents()

        # Execute tasks
        results = self.task_executor.execute_all(self.crew.tasks, self.assignments)

        # Update crew status
        failed_tasks = [r for r in results.values() if not r.success]
        if failed_tasks:
            self.crew.status = "error"
        else:
            self.crew.status = "completed"

        # Aggregate metrics
        self.crew.total_tokens_used += sum(r.tokens_used for r in results.values())
        self.crew.total_cost_usd += sum(r.cost_usd for r in results.values())

        return results
