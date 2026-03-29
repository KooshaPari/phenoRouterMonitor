"""DAG-based remediation plan generation for autonomous governance.

Converts prioritized findings from the analyzer into an executable DAG of
remediation tasks that the agent deployer can dispatch. Uses
graphlib.TopologicalSorter (stdlib) for DAG resolution and applies budget
constraints to produce a feasible execution plan.

The planner:
1. Groups findings by dimension to reduce redundant work
2. Estimates task effort and determines inter-task dependencies
3. Builds a DAG respecting dependencies
4. Applies budget constraints to limit total effort
5. Returns an ordered plan with execution instructions
"""

from __future__ import annotations

import json
import logging
from graphlib import TopologicalSorter
from pathlib import Path
from typing import TYPE_CHECKING, Any
from uuid import uuid4

from pydantic import BaseModel, Field

if TYPE_CHECKING:
    from thegent.governance.analyzer import Finding

_log = logging.getLogger(__name__)


class RemediationTask(BaseModel):
    """A single task in a remediation plan."""

    task_id: str = Field(default_factory=lambda: f"task_{uuid4().hex[:8]}")
    finding_id: str
    dimension: str
    priority: float = Field(ge=0.0)
    severity: float = Field(ge=0.0, le=1.0)
    description: str
    affected_files: list[str] = Field(default_factory=list)
    estimated_cost_calls: int = 1
    dependencies: list[str] = Field(default_factory=list)
    agent_role: str = "workhorse"
    prompt_template: str = ""
    todo_spec: str = ""
    acceptance_criteria: list[str] = Field(default_factory=list)


class RemediationPlan(BaseModel):
    """An executable DAG of remediation tasks."""

    plan_id: str = Field(default_factory=lambda: f"plan_{uuid4().hex[:8]}")
    tasks: list[RemediationTask] = Field(default_factory=list)
    dag_edges: dict[str, list[str]] = Field(default_factory=dict)
    total_estimated_calls: int = 0
    critical_path_calls: int = 0
    execution_order: list[str] = Field(default_factory=list)
    findings_count: int = 0
    budget_remaining_calls: int = 0


class RemediationPlanner:
    """Converts findings into an executable remediation DAG.

    Uses graphlib.TopologicalSorter for DAG ordering and applies
    budget constraints to produce a feasible plan.
    """

    def __init__(self, health_targets_path: Path) -> None:
        """Initialize the planner with health targets configuration.

        Args:
            health_targets_path: Path to contracts/health-targets.json
        """
        with open(health_targets_path) as fh:
            data = json.load(fh)
        self._targets: dict[str, dict] = data["dimensions"]
        self._health_targets_path = health_targets_path
        _log.debug(
            "RemediationPlanner initialized with targets from %s",
            health_targets_path,
        )

    def plan(
        self,
        findings: list[Finding],
        budget_remaining_calls: int,
    ) -> RemediationPlan:
        """Build a remediation plan from findings within budget constraints.

        Args:
            findings: List of Finding objects from HealthAnalyzer, sorted by priority.
            budget_remaining_calls: Remaining API calls available for this cycle.

        Returns:
            A RemediationPlan with tasks, dependencies, and execution order.
        """
        if not findings:
            _log.info("No findings to plan")
            return RemediationPlan()

        # Step 1: Group findings by dimension to reduce redundant work
        grouped = self._group_by_dimension(findings)

        # Step 2: Create initial tasks from grouped findings
        tasks = self._create_tasks(grouped)

        # Step 3: Determine inter-task dependencies
        dag_edges = self._build_dependency_dag(tasks)

        # Step 4: Apply budget constraints
        tasks, total_effort = self._apply_budget_constraints(
            tasks,
            dag_edges,
            budget_remaining_calls,
        )

        # Step 5: Compute execution order using topological sort
        execution_order = self._compute_execution_order(tasks, dag_edges)

        # Step 6: Compute critical path
        critical_path = self._compute_critical_path(
            tasks,
            {t.task_id: t for t in tasks},
            dag_edges,
        )

        plan = RemediationPlan(
            tasks=tasks,
            dag_edges=dag_edges,
            total_estimated_calls=total_effort,
            critical_path_calls=critical_path,
            execution_order=execution_order,
            findings_count=len(findings),
            budget_remaining_calls=budget_remaining_calls - total_effort,
        )

        _log.info(
            "Plan created: %d tasks, %d calls, critical_path=%d, budget_remaining=%d",
            len(tasks),
            total_effort,
            critical_path,
            plan.budget_remaining_calls,
        )

        return plan

    # -----------------------------------------------------------------------
    # private helpers
    # -----------------------------------------------------------------------

    @staticmethod
    def _group_by_dimension(
        findings: list[Finding],
    ) -> dict[str, list[Finding]]:
        """Group findings by dimension to reduce task count.

        Within each dimension group, findings are sorted by priority
        (highest first) to ensure most critical work is attempted first.
        """
        grouped: dict[str, list[Finding]] = {}
        for finding in findings:
            if finding.dimension not in grouped:
                grouped[finding.dimension] = []
            grouped[finding.dimension].append(finding)

        # Sort each group by priority descending
        for dim_findings in grouped.values():
            dim_findings.sort(key=lambda f: f.priority, reverse=True)

        return grouped

    def _create_tasks(
        self,
        grouped_findings: dict[str, list[Finding]],
    ) -> list[RemediationTask]:
        """Create RemediationTask objects from grouped findings.

        Creates one task per dimension (not per finding) to reduce
        redundant work. The task aggregates all findings in that dimension.
        """
        tasks: list[RemediationTask] = []

        for dimension, findings in grouped_findings.items():
            # Take highest-priority finding as representative
            primary = findings[0]

            # Aggregate all affected files from all findings in this group
            all_files = set()
            for f in findings:
                all_files.update(f.affected_files)

            # Sum up estimated effort
            total_effort = sum(f.estimated_effort_tool_calls for f in findings)

            # Generate acceptance criteria
            acceptance_criteria = [
                f"Remediate {dimension} findings: {len(findings)} item(s)",
                f"Target: {primary.target_value:.1f}",
                f"Current: {primary.current_value:.1f}",
                "Verify improvement in next scan",
            ]

            # Generate task description
            description = self._generate_task_description(
                dimension,
                findings,
                primary,
            )

            # Generate prompt template
            prompt_template = self._generate_prompt_template(
                dimension,
                findings,
                primary,
            )

            task = RemediationTask(
                finding_id=primary.finding_id,
                dimension=dimension,
                priority=primary.priority,
                severity=primary.severity,
                description=description,
                affected_files=sorted(list(all_files)),
                estimated_cost_calls=total_effort,
                agent_role=self._resolve_agent_role(dimension),
                prompt_template=prompt_template,
                acceptance_criteria=acceptance_criteria,
            )

            tasks.append(task)

        return tasks

    def _build_dependency_dag(
        self,
        tasks: list[RemediationTask],
    ) -> dict[str, list[str]]:
        """Build a dependency DAG between tasks.

        Currently implements these rules:
        1. doc_disorganization tasks must run before missing_specs tasks
        2. lint_violations tasks must run before technical_debt tasks
        3. No other implicit dependencies

        Returns:
            DAG as dict mapping task_id -> list of dependency task_ids.
        """
        task_by_dimension = {t.dimension: t for t in tasks}
        dag: dict[str, list[str]] = {}

        for task in tasks:
            dag[task.task_id] = []

        # Define dimension-level dependencies
        dependency_rules = [
            ("doc_disorganization", "missing_specs"),
            ("lint_violations", "technical_debt"),
        ]

        for predecessor_dim, successor_dim in dependency_rules:
            pred_task = task_by_dimension.get(predecessor_dim)
            succ_task = task_by_dimension.get(successor_dim)

            if pred_task and succ_task:
                dag[succ_task.task_id].append(pred_task.task_id)
                succ_task.dependencies = [pred_task.task_id]

        return dag

    @staticmethod
    def _apply_budget_constraints(
        tasks: list[RemediationTask],
        dag_edges: dict[str, list[str]],
        budget_remaining_calls: int,
    ) -> tuple[list[RemediationTask], int]:
        """Apply budget constraints to prune low-priority tasks.

        Tasks are pruned in reverse priority order until the total
        estimated effort fits within the budget. Respects task dependencies:
        if a task is kept, its predecessors are also kept.

        Returns:
            (pruned_tasks, total_effort)
        """
        # Track which tasks are kept
        kept_tasks: set[str] = set()
        kept_task_objs: dict[str, RemediationTask] = {t.task_id: t for t in tasks}

        # Sort tasks by priority (descending) for budget allocation
        sorted_tasks = sorted(
            tasks,
            key=lambda t: t.priority,
            reverse=True,
        )

        total_effort = 0
        for task in sorted_tasks:
            task_effort = task.estimated_cost_calls

            # Check if adding this task fits the budget
            if total_effort + task_effort <= budget_remaining_calls:
                kept_tasks.add(task.task_id)
                total_effort += task_effort
            else:
                # Task is too expensive, skip it
                _log.debug(
                    "Pruning task %s (effort=%d, cumulative=%d, budget=%d)",
                    task.task_id,
                    task_effort,
                    total_effort,
                    budget_remaining_calls,
                )

        # Ensure all dependencies of kept tasks are also kept
        for task_id in list(kept_tasks):
            task = kept_task_objs[task_id]
            for dep_id in task.dependencies:
                if dep_id not in kept_tasks:
                    kept_tasks.add(dep_id)
                    total_effort += kept_task_objs[dep_id].estimated_cost_calls

        # Filter to kept tasks and update their dependencies
        result_tasks = [
            t for t in tasks if t.task_id in kept_tasks
        ]

        # Clean up dependencies to only reference kept tasks
        for task in result_tasks:
            task.dependencies = [
                d for d in task.dependencies if d in kept_tasks
            ]

        return result_tasks, total_effort

    @staticmethod
    def _compute_execution_order(
        tasks: list[RemediationTask],
        dag_edges: dict[str, list[str]],
    ) -> list[str]:
        """Compute topologically sorted execution order for tasks.

        Uses graphlib.TopologicalSorter to produce a valid execution order
        that respects all dependencies.
        """
        if not tasks:
            return []

        # Build a clean DAG with only kept tasks
        clean_dag = {}
        task_ids = {t.task_id for t in tasks}

        for task in tasks:
            clean_dag[task.task_id] = [
                d for d in task.dependencies if d in task_ids
            ]

        try:
            sorter = TopologicalSorter(clean_dag)
            sorter.prepare()
            order: list[str] = []
            for _ in range(len(tasks)):
                ready = sorter.get_ready()
                order.extend(sorted(ready))  # Sort for determinism
                for task_id in ready:
                    sorter.done(task_id)
            return order
        except Exception as e:
            _log.warning("Failed to compute topological order: %s", e)
            # Fallback: return tasks sorted by priority
            return [t.task_id for t in sorted(
                tasks,
                key=lambda t: t.priority,
                reverse=True,
            )]

    @staticmethod
    def _compute_critical_path(
        tasks: list[RemediationTask],
        task_map: dict[str, RemediationTask],
        dag_edges: dict[str, list[str]],
    ) -> int:
        """Compute critical path (longest path) in the task DAG.

        Uses a forward-pass DP algorithm: longest_path[task] =
        effort(task) + max(longest_path[dependency] for each dependency).
        """
        if not tasks:
            return 0

        # Compute longest path to each task
        longest: dict[str, int] = {}

        def compute_longest_to(task_id: str) -> int:
            """Compute longest path ending at task_id."""
            if task_id in longest:
                return longest[task_id]

            task = task_map.get(task_id)
            if not task:
                return 0

            # Base case: leaf tasks
            deps = task.dependencies or []
            if not deps:
                longest[task_id] = task.estimated_cost_calls
                return longest[task_id]

            # Recursive case: max(dependencies) + self effort
            max_dep_path = max(
                (compute_longest_to(dep_id) for dep_id in deps),
                default=0,
            )
            longest[task_id] = max_dep_path + task.estimated_cost_calls
            return longest[task_id]

        # Compute for all tasks
        for task in tasks:
            compute_longest_to(task.task_id)

        # Critical path is the longest among all tasks
        return max(longest.values()) if longest else 0

    @staticmethod
    def _resolve_agent_role(dimension: str) -> str:
        """Map dimension to agent role for task execution.

        Maps dimensions to specialized agents (or defaults to workhorse).
        """
        role_map = {
            "test_coverage": "test_engineer",
            "lint_violations": "code_cleaner",
            "doc_disorganization": "doc_curator",
            "missing_specs": "spec_writer",
            "technical_debt": "refactorer",
            "stale_items": "janitor",
            "agent_failure": "debugger",
        }
        return role_map.get(dimension, "workhorse")

    @staticmethod
    def _generate_task_description(
        dimension: str,
        findings: list[Finding],
        primary: Finding,
    ) -> str:
        """Generate a human-readable task description."""
        descriptions = {
            "test_coverage": (
                f"Improve test coverage from {primary.current_value:.0f}% to "
                f"{primary.target_value:.0f}% ({len(findings)} finding(s))"
            ),
            "lint_violations": (
                f"Reduce lint violations from {primary.current_value:.0f} to "
                f"{primary.target_value:.0f} ({len(findings)} finding(s))"
            ),
            "doc_disorganization": (
                f"Organize {primary.current_value:.0f} missing doc directories "
                f"(target: {primary.target_value:.0f})"
            ),
            "fragmented_research": (
                f"Consolidate {primary.current_value:.0f} research files "
                f"outside docs/research/"
            ),
            "missing_specs": (
                f"Create {primary.current_value:.0f} spec files "
                f"(target: {primary.target_value:.0f})"
            ),
            "technical_debt": (
                f"Reduce cyclomatic complexity from {primary.current_value:.1f} "
                f"to {primary.target_value:.0f}"
            ),
            "stale_items": (
                f"Remove {primary.current_value:.0f} stale items in specs/ "
                f"(target: {primary.target_value:.0f})"
            ),
            "agent_failure": (
                f"Resolve {primary.current_value:.0f} open circuit breakers "
                f"(target: {primary.target_value:.0f})"
            ),
        }
        return descriptions.get(
            dimension,
            f"Remediate {dimension} ({len(findings)} finding(s))",
        )

    @staticmethod
    def _generate_prompt_template(
        dimension: str,
        findings: list[Finding],
        primary: Finding,
    ) -> str:
        """Generate a task prompt template for agent execution.

        This is a template that will be customized per finding.
        Agents will replace {placeholder} values with specific details.
        """
        prompts = {
            "test_coverage": (
                "Increase test coverage for {affected_files}. "
                f"Current: {primary.current_value:.0f}%, Target: {primary.target_value:.0f}%. "
                "Write unit and integration tests to cover untested paths."
            ),
            "lint_violations": (
                "Fix lint violations in {affected_files}. "
                f"Current violations: {primary.current_value:.0f}, Target: {primary.target_value:.0f}. "
                "Run linters and fix issues systematically."
            ),
            "doc_disorganization": (
                "Organize documentation structure: create missing directories "
                f"(target: {primary.target_value:.0f}). "
                "Ensure docs/ contains all required subdirectories: guides/, reference/, reports/, checklists/."
            ),
            "missing_specs": (
                "Create missing specification files for approved features. "
                f"Currently missing: {primary.current_value:.0f}, Target: {primary.target_value:.0f}. "
                "Write PRD.md, FUNCTIONAL_REQUIREMENTS.md, PLAN.md, ADR.md as needed."
            ),
            "technical_debt": (
                "Refactor code to reduce cyclomatic complexity in {affected_files}. "
                f"Current: {primary.current_value:.1f}, Target: {primary.target_value:.0f}. "
                "Break down complex functions and simplify control flow."
            ),
            "stale_items": (
                "Remove stale items from specs/ directory. "
                f"Currently stale: {primary.current_value:.0f}, Target: {primary.target_value:.0f}. "
                "Archive completed specs to .archive/ and remove obsolete items."
            ),
            "agent_failure": (
                "Debug and resolve open circuit breakers. "
                f"Current: {primary.current_value:.0f}, Target: {primary.target_value:.0f}. "
                "Investigate failure patterns and implement fixes."
            ),
        }

        template = prompts.get(
            dimension,
            f"Remediate {dimension} findings in {{affected_files}}. "
            f"Current: {primary.current_value}, Target: {primary.target_value}.",
        )

        return template
