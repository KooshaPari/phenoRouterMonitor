"""WorkflowEngine for multi-crew stages."""

from dataclasses import dataclass, field
from typing import Any

from .crew import Crew
from .executor import CrewExecutor, ExecutionResult


@dataclass
class CrewStage:
    """A stage in a workflow containing one or more crews."""

    id: str
    name: str
    crews: list[Crew] = field(default_factory=list)
    depends_on: list[str] = field(default_factory=list)  # Stage IDs
    parallel: bool = False  # Can run in parallel with other stages


class WorkflowEngine:
    """
    Manages multi-crew workflows with stages.

    Supports:
    - Multi-crew execution
    - Stage dependencies
    - Parallel stage execution
    - Result aggregation
    """

    def __init__(self) -> None:
        """Initialize WorkflowEngine."""
        self.stages: list[CrewStage] = []
        self.results: dict[str, Any] = {}  # stage_id -> {crew_id -> results}

    def add_stage(self, stage: CrewStage) -> None:
        """Add a stage to the workflow."""
        self.stages.append(stage)

    def _build_stage_graph(self) -> tuple[dict[str, CrewStage], dict[str, int], dict[str, list[str]]]:
        """Build and validate stage graph structures.

        Validation is intentionally strict: duplicate stage IDs, unknown
        dependencies, and self-dependencies fail fast before execution starts.
        """
        stage_map: dict[str, CrewStage] = {}
        for stage in self.stages:
            if not stage.id.strip():
                raise ValueError("Stage id cannot be empty")
            if stage.id in stage_map:
                raise ValueError(f"Duplicate stage id: {stage.id}")
            stage_map[stage.id] = stage

        in_degree = dict.fromkeys(stage_map, 0)
        graph: dict[str, list[str]] = {stage_id: [] for stage_id in stage_map}

        for stage in self.stages:
            for dep_id in stage.depends_on:
                if dep_id not in stage_map:
                    raise ValueError(f"Unknown dependency {dep_id!r} for stage {stage.id!r}")
                if dep_id == stage.id:
                    raise ValueError(f"Stage {stage.id!r} cannot depend on itself")
                graph[dep_id].append(stage.id)
                in_degree[stage.id] += 1

        return stage_map, in_degree, graph

    def _build_execution_plan(self) -> list[CrewStage]:
        """Create a validated execution plan before running any stages."""
        ordered_stages = self.resolve_stage_dependencies()

        for stage in ordered_stages:
            crew_ids = [crew.id for crew in stage.crews]
            if len(crew_ids) != len(set(crew_ids)):
                raise ValueError(f"Duplicate crew id in stage {stage.id!r}")

        return list(ordered_stages)

    def resolve_stage_dependencies(self) -> list[CrewStage]:
        """
        Resolve stage dependencies using topological sort on a validated graph.

        Returns stages in execution order.
        """
        stage_map, in_degree, graph = self._build_stage_graph()

        # Topological sort
        queue = [stage_id for stage_id, degree in in_degree.items() if degree == 0]
        result = []

        while queue:
            stage_id = queue.pop(0)
            result.append(stage_map[stage_id])

            for dependent_id in graph[stage_id]:
                in_degree[dependent_id] -= 1
                if in_degree[dependent_id] == 0:
                    queue.append(dependent_id)

        if len(result) != len(self.stages):
            raise ValueError("Circular dependency detected in stages")

        return result

    def execute_stage(self, stage: CrewStage) -> dict[str, dict[str, ExecutionResult]]:
        """
        Execute all crews in a stage.

        Args:
            stage: Stage to execute

        Returns:
            Map of crew_id -> {task_id -> ExecutionResult}
        """
        stage_results: dict[str, dict[str, ExecutionResult]] = {}

        for crew in stage.crews:
            executor = CrewExecutor(crew)
            results: dict[str, ExecutionResult] = executor.execute()
            stage_results[crew.id] = results

        self.results[stage.id] = stage_results
        return stage_results

    def execute(self) -> dict[str, dict[str, dict[str, ExecutionResult]]]:
        """
        Execute entire workflow respecting stage dependencies.

        Returns:
            Map of stage_id -> {crew_id -> {task_id -> ExecutionResult}}
        """
        ordered_stages = self._build_execution_plan()
        all_results: dict[str, dict[str, dict[str, ExecutionResult]]] = {}

        for stage in ordered_stages:
            stage_results = self.execute_stage(stage)
            all_results[stage.id] = stage_results

        return all_results
