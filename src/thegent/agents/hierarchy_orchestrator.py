"""Hierarchy orchestrator extending the plangent runner pattern.

Supports delegating to sub-agents with different personas,
manages context passing between hierarchy levels, and uses
structlog for structured logging.

# @trace FR-AGT-025
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

import structlog

from thegent.agents.plangent import Plan, PlangentExecutor, PlangentPlanner, PlanNode

log = structlog.get_logger(__name__)


@dataclass
class SubAgentConfig:
    """Configuration for a sub-agent persona.

    Attributes:
        name: Unique name identifying this sub-agent.
        persona: Description of the persona/role for prompting.
        model: Optional model override for this sub-agent.
        metadata: Arbitrary extra configuration.
    """

    name: str
    persona: str
    model: str | None = None
    metadata: dict[str, Any] = field(default_factory=dict)


class HierarchyOrchestrator:
    """Orchestrate work across sub-agents using the plangent DAG pattern.

    The orchestrator maintains a registry of sub-agent personas and
    delegates plan nodes to the appropriate sub-agent based on
    metadata tags. Context is passed down through the hierarchy
    via a shared context dict that accumulates results.

    Args:
        planner: PlangentPlanner for plan decomposition.
        executor: PlangentExecutor for plan execution.
    """

    def __init__(
        self,
        planner: PlangentPlanner | None = None,
        executor: PlangentExecutor | None = None,
    ) -> None:
        self._planner = planner or PlangentPlanner()
        self._executor = executor or PlangentExecutor(self._planner)
        self._agents: dict[str, SubAgentConfig] = {}
        self._context: dict[str, Any] = {}

    def register_agent(self, config: SubAgentConfig) -> None:
        """Register a sub-agent persona.

        Args:
            config: Sub-agent configuration.

        Raises:
            ValueError: If an agent with the same name is already registered.
        """
        if config.name in self._agents:
            raise ValueError(f"Agent '{config.name}' already registered")
        self._agents[config.name] = config
        log.info("agent_registered", agent=config.name, persona=config.persona)

    def get_agent(self, name: str) -> SubAgentConfig:
        """Retrieve a registered sub-agent by name.

        Args:
            name: Sub-agent name.

        Returns:
            The SubAgentConfig.

        Raises:
            KeyError: If the agent is not registered.
        """
        if name not in self._agents:
            raise KeyError(f"Agent '{name}' not registered")
        return self._agents[name]

    def list_agents(self) -> list[SubAgentConfig]:
        """Return all registered sub-agent configs."""
        return list(self._agents.values())

    def set_context(self, key: str, value: Any) -> None:
        """Set a context value to be passed down the hierarchy.

        Args:
            key: Context key.
            value: Context value.
        """
        self._context[key] = value
        log.debug("context_set", key=key)

    def get_context(self) -> dict[str, Any]:
        """Return the current shared context dict."""
        return dict(self._context)

    def decompose(self, goal: str, max_depth: int = 3) -> Plan:
        """Decompose a goal into a plan DAG.

        Args:
            goal: Natural-language goal.
            max_depth: Maximum decomposition depth.

        Returns:
            A Plan with pending nodes.
        """
        plan = self._planner.decompose(goal, max_depth)
        log.info("plan_decomposed", goal=goal, node_count=len(plan.nodes))
        return plan

    def execute(self, plan: Plan, runner: Any) -> Plan:
        """Execute a plan, delegating nodes to sub-agents.

        Args:
            plan: The Plan to execute.
            runner: Callable (PlanNode) -> str invoked for each node.

        Returns:
            The mutated plan with updated statuses.
        """
        log.info("plan_execution_start", plan_id=plan.id, nodes=len(plan.nodes))

        def _wrapped_runner(node: PlanNode) -> str:
            agent_name = node.metadata.get("agent")
            if agent_name and agent_name in self._agents:
                agent = self._agents[agent_name]
                log.info("delegating_to_agent", agent=agent.name, node=node.id)
                node.metadata["persona"] = agent.persona
                node.metadata["context"] = dict(self._context)
            result = runner(node)
            self._context[f"result:{node.id}"] = result
            return result

        completed_plan = self._executor.execute(plan, _wrapped_runner)
        log.info(
            "plan_execution_complete",
            plan_id=plan.id,
            done=len(completed_plan.done_ids),
            failed=len(completed_plan.failed_ids),
        )
        return completed_plan
