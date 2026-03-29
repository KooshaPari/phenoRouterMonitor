"""Plangent-style planning sub-agents for thegent.

Provides DAG-based task decomposition and structured plan execution.
The PlangentPlanner decomposes a goal into a directed acyclic graph (DAG)
of sub-tasks, and the PlangentExecutor dispatches each ready node to a
caller-supplied runner (sync or async).

When a plan is an OrchestrationPlan, execute_async() delegates to
SubAgentDispatcher.dispatch_plan() instead of the caller-supplied runner.
Results are collected via ResultAggregator and node statuses updated
accordingly.

LLMPlangentPlanner is a subclass of PlangentPlanner that overrides
_generate_sub_tasks() to use a FlashAgent LLM call for structured
decomposition. Output is validated against OrchestrationPlan schema.
The explicit fallback strategy (model unavailable → parent heuristic)
is documented: this is a deliberate design decision for the planning
path only, where partial decomposition is worse than no decomposition.

# @trace FR-AGT-020
# @trace WL-084
# @trace WL-087
"""

from __future__ import annotations

import asyncio
import orjson as json
import logging
import uuid
from collections.abc import Callable
from dataclasses import dataclass, field
from datetime import UTC, datetime
from typing import TYPE_CHECKING, Any, Literal

from thegent.agents.flash_agent import FlashAgent, FlashAgentConfig

if TYPE_CHECKING:
    from thegent.orchestration.dispatcher import SubAgentDispatcher

_log = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Data types
# ---------------------------------------------------------------------------

NodeStatus = Literal["pending", "running", "done", "failed"]


@dataclass
class PlanNode:
    """A single task node within a Plan DAG.

    Attributes:
        id: Unique node identifier (auto-generated UUID if not set).
        task: Natural-language description of the sub-task.
        depends_on: List of node IDs that must be ``done`` before this node
            can run.
        status: Current lifecycle status of the node.
        result: Output string produced when the node completes successfully.
        error: Error string stored when the node transitions to ``failed``.
        metadata: Arbitrary extra data (e.g., model hints, routing tags).
    """

    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    task: str = ""
    depends_on: list[str] = field(default_factory=list)
    status: NodeStatus = "pending"
    result: str | None = None
    error: str | None = None
    metadata: dict[str, Any] = field(default_factory=dict)

    def is_ready(self, done_ids: set[str]) -> bool:
        """Return True when all dependencies are satisfied."""
        return self.status == "pending" and all(dep in done_ids for dep in self.depends_on)

    def to_dict(self) -> dict[str, Any]:
        """Serialize to a plain dict for JSON / JSONL serialisation."""
        return {
            "id": self.id,
            "task": self.task,
            "depends_on": list(self.depends_on),
            "status": self.status,
            "result": self.result,
            "error": self.error,
            "metadata": dict(self.metadata),
        }


@dataclass
class Plan:
    """A complete execution plan composed of a DAG of PlanNodes.

    Attributes:
        id: Unique plan identifier.
        goal: High-level goal statement that was decomposed.
        nodes: Ordered list of :class:`PlanNode` objects.
        created_at: UTC timestamp when the plan was created.
        metadata: Arbitrary extra data attached to the plan.
    """

    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    goal: str = ""
    nodes: list[PlanNode] = field(default_factory=list)
    created_at: datetime = field(default_factory=lambda: datetime.now(UTC))
    metadata: dict[str, Any] = field(default_factory=dict)

    # ------------------------------------------------------------------
    # Convenience accessors
    # ------------------------------------------------------------------

    def get_node(self, node_id: str) -> PlanNode | None:
        """Return the node with the given ID, or ``None``."""
        for node in self.nodes:
            if node.id == node_id:
                return node
        return None

    @property
    def done_ids(self) -> set[str]:
        """Set of node IDs whose status is ``done``."""
        return {n.id for n in self.nodes if n.status == "done"}

    @property
    def failed_ids(self) -> set[str]:
        """Set of node IDs whose status is ``failed``."""
        return {n.id for n in self.nodes if n.status == "failed"}

    def to_dict(self) -> dict[str, Any]:
        """Serialize to a plain dict."""
        return {
            "id": self.id,
            "goal": self.goal,
            "nodes": [n.to_dict() for n in self.nodes],
            "created_at": self.created_at.isoformat(),
            "metadata": dict(self.metadata),
        }


# ---------------------------------------------------------------------------
# PlangentPlanner
# ---------------------------------------------------------------------------


class PlangentPlanner:
    """Decomposes a goal into a DAG of sub-tasks.

    The default ``decompose`` implementation produces a simple deterministic
    breakdown that requires no LLM call.  Subclass and override
    ``_generate_sub_tasks`` to inject an LLM-backed decomposition strategy.
    """

    def __init__(self, *, separator: str = ".", max_nodes_per_level: int = 5) -> None:
        """Initialise the planner.

        Args:
            separator: Character used to split compound goal strings during
                heuristic decomposition.
            max_nodes_per_level: Maximum sub-tasks per depth level when
                decomposing a compound goal.
        """
        self._separator = separator
        self._max_nodes_per_level = max_nodes_per_level

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def decompose(self, goal: str, max_depth: int = 3) -> Plan:
        """Break *goal* into a :class:`Plan` with a DAG of :class:`PlanNode`.

        Each node inherits the previous node as a dependency, forming a
        simple linear chain by default.  Override ``_generate_sub_tasks`` to
        produce arbitrary DAG shapes.

        Args:
            goal: Natural-language goal to decompose.
            max_depth: Maximum depth of the resulting DAG.  Ignored by the
                default heuristic implementation but forwarded to
                ``_generate_sub_tasks``.

        Returns:
            A :class:`Plan` instance with all nodes in ``pending`` status.
        """
        if not goal or not goal.strip():
            raise ValueError("goal must be a non-empty string")

        sub_tasks = self._generate_sub_tasks(goal.strip(), max_depth)
        plan = Plan(goal=goal.strip())

        prev_id: str | None = None
        for task_text in sub_tasks[: self._max_nodes_per_level * max_depth]:
            node = PlanNode(task=task_text, depends_on=[prev_id] if prev_id else [])
            plan.nodes.append(node)
            prev_id = node.id

        _log.debug("Decomposed goal=%r into %d nodes", goal, len(plan.nodes))
        return plan

    def next_ready_tasks(self, plan: Plan) -> list[PlanNode]:
        """Return all nodes that are ready to execute.

        A node is *ready* when its status is ``pending`` and every dependency
        is ``done``.

        Args:
            plan: The plan to evaluate.

        Returns:
            List of nodes that can start immediately (may be empty).
        """
        done = plan.done_ids
        return [n for n in plan.nodes if n.is_ready(done)]

    def mark_done(self, plan: Plan, node_id: str, result: str) -> None:
        """Mark a node as successfully completed.

        Args:
            plan: The plan containing the node.
            node_id: ID of the node to update.
            result: Output produced by the node.

        Raises:
            ValueError: If ``node_id`` is not found in the plan.
        """
        node = plan.get_node(node_id)
        if node is None:
            raise ValueError(f"Node '{node_id}' not found in plan '{plan.id}'")
        node.status = "done"
        node.result = result
        _log.debug("Node %s marked done (plan %s)", node_id, plan.id)

    def mark_failed(self, plan: Plan, node_id: str, error: str) -> None:
        """Mark a node as failed.

        Args:
            plan: The plan containing the node.
            node_id: ID of the node to update.
            error: Error message explaining the failure.

        Raises:
            ValueError: If ``node_id`` is not found in the plan.
        """
        node = plan.get_node(node_id)
        if node is None:
            raise ValueError(f"Node '{node_id}' not found in plan '{plan.id}'")
        node.status = "failed"
        node.error = error
        _log.debug("Node %s marked failed (plan %s): %s", node_id, plan.id, error)

    def is_complete(self, plan: Plan) -> bool:
        """Return ``True`` when every node is ``done`` or ``failed``.

        Args:
            plan: The plan to evaluate.
        """
        return all(n.status in ("done", "failed") for n in plan.nodes)

    def to_work_stream_rows(self, plan: Plan) -> list[dict[str, Any]]:
        """Convert a Plan into WORK_STREAM-compatible row dicts.

        Each row has keys: ``id``, ``title``, ``source``, ``priority``,
        ``depends``, ``status``.

        Args:
            plan: The plan to convert.

        Returns:
            List of dicts, one per node.
        """
        rows: list[dict[str, Any]] = []
        for node in plan.nodes:
            rows.append(
                {
                    "id": node.id,
                    "title": node.task,
                    "source": f"plan:{plan.id}",
                    "priority": "P2",
                    "depends": ",".join(node.depends_on) if node.depends_on else "-",
                    "status": node.status,
                }
            )
        return rows

    # ------------------------------------------------------------------
    # Overridable hook
    # ------------------------------------------------------------------

    def _generate_sub_tasks(self, goal: str, max_depth: int) -> list[str]:
        """Produce sub-task descriptions for a goal.

        The default implementation performs a simple heuristic split:
        - If the goal contains the separator character, split on it.
        - Otherwise, produce a minimal 5-step breakdown.

        Override this method in a subclass to inject an LLM-powered
        decomposition strategy.

        Args:
            goal: The (stripped) goal string.
            max_depth: Hint for how deep the decomposition should go.

        Returns:
            Ordered list of sub-task strings.
        """
        # max_depth is intentionally unused in the heuristic; subclasses use it.
        _ = max_depth
        if self._separator in goal:
            parts = [p.strip() for p in goal.split(self._separator) if p.strip()]
            return parts or [goal]

        return [
            f"Analyse and understand the scope of: {goal}",
            f"Design the solution approach for: {goal}",
            f"Implement: {goal}",
            f"Test and validate: {goal}",
            f"Document and finalise: {goal}",
        ]


# ---------------------------------------------------------------------------
# Internal helpers for OrchestrationPlan execution path
# ---------------------------------------------------------------------------


@dataclass
class _AggregatorMessage:
    """Minimal duck-type compatible with ResultAggregator.add() expectations.

    ResultAggregator reads ``.message_type`` from each item it aggregates.
    Using this lightweight dataclass avoids importing InterAgentMessage here
    (which would create a circular dependency via the orchestration package).

    # @trace WL-084
    """

    sender_id: str
    recipient_id: str
    message_type: str
    payload: dict[str, Any]


def _make_aggregator_message(
    *,
    sender_id: str,
    recipient_id: str,
    message_type: str,
    payload: dict[str, Any],
) -> _AggregatorMessage:
    """Construct an _AggregatorMessage for use with ResultAggregator.

    # @trace WL-084
    """
    return _AggregatorMessage(
        sender_id=sender_id,
        recipient_id=recipient_id,
        message_type=message_type,
        payload=payload,
    )


# ---------------------------------------------------------------------------
# PlangentExecutor
# ---------------------------------------------------------------------------

RunnerType = Callable[[PlanNode], str]


class PlangentExecutor:
    """Executes a Plan by dispatching sub-tasks to thegent agents.

    The executor iterates over ready nodes, invokes the caller-supplied
    *runner* callback, and updates node status (``done`` / ``failed``).
    It repeats until the plan is complete or a blocking failure is detected.

    Attributes:
        planner: The :class:`PlangentPlanner` used to query plan state.
        fail_fast: When ``True``, execution stops immediately on the first
            failed node.  When ``False``, execution continues, skipping nodes
            whose dependencies have failed.
    """

    def __init__(
        self,
        planner: PlangentPlanner | None = None,
        *,
        fail_fast: bool = False,
    ) -> None:
        """Initialise the executor.

        Args:
            planner: :class:`PlangentPlanner` instance used to inspect plan
                state.  A default ``PlangentPlanner()`` is created if not
                provided.
            fail_fast: Stop on first failure when ``True``.
        """
        self.planner = planner or PlangentPlanner()
        self.fail_fast = fail_fast

    # ------------------------------------------------------------------
    # Sync execution
    # ------------------------------------------------------------------

    def execute(self, plan: Plan, runner: RunnerType) -> Plan:
        """Execute *plan* synchronously by dispatching each ready node.

        The method loops until the plan is complete (all nodes done/failed)
        or until no progress can be made (deadlock — remaining pending nodes
        have unsatisfied dependencies due to failures).

        Args:
            plan: The :class:`Plan` to execute.
            runner: Callable ``(PlanNode) -> str`` invoked for each ready
                node.  Must return the result string on success or raise an
                exception on failure.

        Returns:
            The mutated *plan* with updated node statuses.
        """
        _log.info("Executing plan %s (%d nodes) goal=%r", plan.id, len(plan.nodes), plan.goal)

        while not self.planner.is_complete(plan):
            ready = self.planner.next_ready_tasks(plan)
            if not ready:
                _log.warning("No ready tasks; plan %s may be deadlocked", plan.id)
                break

            for node in ready:
                node.status = "running"
                try:
                    result = runner(node)
                    self.planner.mark_done(plan, node.id, result)
                except Exception as exc:
                    error_msg = str(exc)
                    _log.error("Node %s failed: %s", node.id, error_msg)
                    self.planner.mark_failed(plan, node.id, error_msg)
                    if self.fail_fast:
                        _log.info("fail_fast=True; stopping plan %s after node %s failure", plan.id, node.id)
                        return plan

        _log.info(
            "Plan %s complete: %d done, %d failed",
            plan.id,
            len(plan.done_ids),
            len(plan.failed_ids),
        )
        return plan

    # ------------------------------------------------------------------
    # Async execution
    # ------------------------------------------------------------------

    async def execute_async(
        self,
        plan: Plan,
        runner: Callable[[PlanNode], Any],
        dispatcher: SubAgentDispatcher | None = None,
    ) -> Plan:
        """Execute *plan* asynchronously.

        When *plan* is an :class:`~thegent.orchestration.plan.OrchestrationPlan`,
        execution is delegated to a
        :class:`~thegent.orchestration.dispatcher.SubAgentDispatcher`.  The
        dispatcher replaces the inline *runner* callback for this path.  If
        *dispatcher* is not provided, one is built automatically using
        :meth:`~thegent.agents.capability_index.CapabilityIndex.get`.
        Results are collected via
        :class:`~thegent.orchestration.aggregator.ResultAggregator` and the
        aggregation summary is stored in ``plan.metadata["aggregation"]``.

        When *plan* is a plain :class:`Plan`, the caller-supplied *runner*
        callback is used unchanged via ``asyncio.gather`` (original behaviour).

        Args:
            plan: The :class:`Plan` to execute.
            runner: Async coroutine ``async (PlanNode) -> str`` or sync
                ``(PlanNode) -> str`` invoked for each ready node when plan
                is a plain :class:`Plan`.  Ignored for
                :class:`OrchestrationPlan`.
            dispatcher: Optional
                :class:`~thegent.orchestration.dispatcher.SubAgentDispatcher`
                used when *plan* is an
                :class:`~thegent.orchestration.plan.OrchestrationPlan`.  When
                ``None`` and the plan is an ``OrchestrationPlan``, a dispatcher
                is constructed automatically.

        Returns:
            The mutated *plan* with updated node statuses.

        # @trace WL-084
        """
        # Inline import to avoid circular imports: the orchestration package
        # imports from thegent.agents.plangent, so a top-level import of
        # thegent.orchestration.plan here would create a cycle.
        from thegent.orchestration.plan import OrchestrationPlan  # noqa: PLC0415

        if isinstance(plan, OrchestrationPlan):
            return await self._execute_orchestration_async(plan, dispatcher)

        return await self._execute_plain_async(plan, runner)

    # ------------------------------------------------------------------
    # Private: plain Plan path (original runner-callback behaviour)
    # ------------------------------------------------------------------

    async def _execute_plain_async(
        self,
        plan: Plan,
        runner: Callable[[PlanNode], Any],
    ) -> Plan:
        """Execute a plain :class:`Plan` using the caller-supplied *runner*.

        # @trace WL-084
        """
        _log.info("Async-executing plan %s (%d nodes)", plan.id, len(plan.nodes))

        while not self.planner.is_complete(plan):
            ready = self.planner.next_ready_tasks(plan)
            if not ready:
                _log.warning("No ready tasks; plan %s may be deadlocked (async)", plan.id)
                break

            # Mark all ready nodes as running before dispatching.
            for node in ready:
                node.status = "running"

            async def _run_node(n: PlanNode) -> tuple[str, str | Exception]:
                try:
                    if asyncio.iscoroutinefunction(runner):
                        res = await runner(n)
                    else:
                        res = await asyncio.to_thread(runner, n)
                    return n.id, res
                except Exception as exc:
                    return n.id, exc

            outcomes = await asyncio.gather(*[_run_node(n) for n in ready])

            for node_id, outcome in outcomes:
                if isinstance(outcome, Exception):
                    _log.error("Node %s failed (async): %s", node_id, outcome)
                    self.planner.mark_failed(plan, node_id, str(outcome))
                    if self.fail_fast:
                        _log.info("fail_fast=True; stopping plan %s", plan.id)
                        return plan
                else:
                    self.planner.mark_done(plan, node_id, outcome)

        _log.info(
            "Plan %s async-complete: %d done, %d failed",
            plan.id,
            len(plan.done_ids),
            len(plan.failed_ids),
        )
        return plan

    # ------------------------------------------------------------------
    # Private: OrchestrationPlan path via SubAgentDispatcher
    # ------------------------------------------------------------------

    async def _execute_orchestration_async(
        self,
        plan: Any,  # OrchestrationPlan; typed Any to satisfy forward-ref constraint
        dispatcher: SubAgentDispatcher | None,
    ) -> Plan:
        """Execute an OrchestrationPlan via SubAgentDispatcher + ResultAggregator.

        Delegates all node dispatch to ``SubAgentDispatcher.dispatch_plan()``,
        feeds each ``DispatchResult`` into a ``ResultAggregator``, maps results
        back onto plan node statuses (``done`` / ``failed``), and stores the
        aggregation summary in ``plan.metadata["aggregation"]``.

        Errors are propagated loudly — no silent failure handling.

        # @trace WL-084
        """
        from thegent.orchestration.aggregator import ResultAggregator  # noqa: PLC0415
        from thegent.orchestration.dispatcher import (  # noqa: PLC0415
            SubAgentDispatcher as _SubAgentDispatcher,
        )

        if dispatcher is None:
            from thegent.agents.capability_index import CapabilityIndex  # noqa: PLC0415

            dispatcher = _SubAgentDispatcher(capability_index=CapabilityIndex.get())

        _log.info(
            "Orchestration-dispatching plan %s (%d nodes) via SubAgentDispatcher",
            plan.id,
            len(plan.nodes),
        )

        dispatch_results = await dispatcher.dispatch_plan(plan)

        aggregator = ResultAggregator()

        for node in plan.nodes:
            dispatch_result = dispatch_results.get(node.id)
            if dispatch_result is None:
                # A node may be missing from dispatch_results if the plan has
                # no nodes or a topological deadlock left some unreachable.
                # Fail loudly — do not silently skip.
                error_msg = f"No DispatchResult returned for node {node.id} in plan {plan.id}"
                _log.error(error_msg)
                self.planner.mark_failed(plan, node.id, error_msg)
                aggregator.add(
                    _make_aggregator_message(
                        sender_id=node.id,
                        recipient_id=plan.id,
                        message_type="error",
                        payload={"error": error_msg, "node_id": node.id},
                    ),
                    node_id=node.id,
                )
                if self.fail_fast:
                    plan.metadata["aggregation"] = aggregator.aggregate()
                    _log.info("fail_fast=True; stopping orchestration plan %s", plan.id)
                    return plan
                continue

            if dispatch_result.success:
                aggregator.add(
                    _make_aggregator_message(
                        sender_id=node.id,
                        recipient_id=plan.id,
                        message_type="result",
                        payload={"output": dispatch_result.output, "node_id": node.id},
                    ),
                    node_id=node.id,
                )
                self.planner.mark_done(plan, node.id, dispatch_result.output)
            else:
                error_msg = dispatch_result.error or f"Dispatch failed for node {node.id}"
                aggregator.add(
                    _make_aggregator_message(
                        sender_id=node.id,
                        recipient_id=plan.id,
                        message_type="error",
                        payload={"error": error_msg, "node_id": node.id},
                    ),
                    node_id=node.id,
                )
                self.planner.mark_failed(plan, node.id, error_msg)
                _log.error("Node %s failed in orchestration plan %s: %s", node.id, plan.id, error_msg)
                if self.fail_fast:
                    plan.metadata["aggregation"] = aggregator.aggregate()
                    _log.info(
                        "fail_fast=True; stopping orchestration plan %s after node %s",
                        plan.id,
                        node.id,
                    )
                    return plan

        agg_result = aggregator.aggregate()
        plan.metadata["aggregation"] = agg_result

        _log.info(
            "Orchestration plan %s complete: %d done, %d failed — aggregation passed=%s",
            plan.id,
            len(plan.done_ids),
            len(plan.failed_ids),
            agg_result["passed"],
        )
        return plan


# ---------------------------------------------------------------------------
# LLMPlangentPlanner
# ---------------------------------------------------------------------------

#: JSON schema that LLM output must satisfy.  Each node entry requires an
#: ``id`` (str), ``task`` (str), ``agent_hint`` (str | null),
#: ``deps`` (list[str]), and ``budget_tokens`` (int | null).
_LLM_NODE_REQUIRED_KEYS: frozenset[str] = frozenset({"id", "task", "agent_hint", "deps", "budget_tokens"})

#: System prompt template instructing the model to produce structured JSON.
#: Curly braces used as JSON example must be doubled for str.format() escaping.
_DECOMPOSITION_SYSTEM_PROMPT = """\
You are a precise task decomposition assistant.
Decompose the given goal into a sequence of concrete sub-tasks for an AI agent team.
IMPORTANT: Respond ONLY with a single JSON object — no markdown, no prose before or after.

Required format:
{{
  "nodes": [
    {{
      "id": "<unique short id, e.g. t1>",
      "task": "<concise task description>",
      "agent_hint": "<suggested agent persona, e.g. 'coder', 'researcher', or null>",
      "deps": ["<id of dependency>"],
      "budget_tokens": <integer token budget or null>
    }}
  ]
}}

Rules:
- ids must be unique within the list.
- deps must reference ids that appear earlier in the list (DAG, no cycles).
- Produce between 2 and {max_depth} nodes.
- If you cannot decompose meaningfully, produce at least 2 generic steps.
"""


@dataclass
class _LLMNodeSpec:
    """Intermediate parsed representation of one LLM-produced node spec.

    # @trace WL-087
    """

    id: str
    task: str
    agent_hint: str | None
    deps: list[str]
    budget_tokens: int | None


def _parse_llm_response(raw: str) -> list[_LLMNodeSpec]:
    """Parse and validate the raw LLM JSON response into node specs.

    Args:
        raw: Raw string returned by the LLM.

    Returns:
        List of :class:`_LLMNodeSpec` objects.

    Raises:
        ValueError: If the JSON is malformed, missing the ``nodes`` key,
            or any node entry is missing required keys or has wrong types.

    # @trace WL-087
    """
    raw = raw.strip()
    try:
        parsed = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise ValueError(f"LLM output is not valid JSON: {exc}\nRaw output: {raw!r}") from exc

    if not isinstance(parsed, dict):
        raise ValueError(f"LLM output must be a JSON object, got {type(parsed).__name__!r}")

    nodes_raw = parsed.get("nodes")
    if nodes_raw is None:
        raise ValueError("LLM output JSON is missing required key 'nodes'")
    if not isinstance(nodes_raw, list):
        raise ValueError(f"LLM output 'nodes' must be a list, got {type(nodes_raw).__name__!r}")
    if len(nodes_raw) == 0:
        raise ValueError("LLM output 'nodes' list must not be empty")

    specs: list[_LLMNodeSpec] = []
    for idx, entry in enumerate(nodes_raw):
        if not isinstance(entry, dict):
            raise ValueError(f"nodes[{idx}] must be a JSON object, got {type(entry).__name__!r}")

        missing = _LLM_NODE_REQUIRED_KEYS - entry.keys()
        if missing:
            raise ValueError(f"nodes[{idx}] is missing required keys: {sorted(missing)}")

        node_id = entry["id"]
        if not isinstance(node_id, str) or not node_id.strip():
            raise ValueError(f"nodes[{idx}].id must be a non-empty string, got {node_id!r}")

        task = entry["task"]
        if not isinstance(task, str) or not task.strip():
            raise ValueError(f"nodes[{idx}].task must be a non-empty string, got {task!r}")

        agent_hint = entry["agent_hint"]
        if agent_hint is not None and not isinstance(agent_hint, str):
            raise ValueError(f"nodes[{idx}].agent_hint must be a string or null, got {agent_hint!r}")

        deps = entry["deps"]
        if not isinstance(deps, list):
            raise ValueError(f"nodes[{idx}].deps must be a list, got {type(deps).__name__!r}")
        for dep in deps:
            if not isinstance(dep, str):
                raise ValueError(f"nodes[{idx}].deps entries must be strings, got {dep!r}")

        budget_tokens = entry["budget_tokens"]
        if budget_tokens is not None and not isinstance(budget_tokens, int):
            raise ValueError(f"nodes[{idx}].budget_tokens must be an int or null, got {budget_tokens!r}")

        specs.append(
            _LLMNodeSpec(
                id=node_id.strip(),
                task=task.strip(),
                agent_hint=agent_hint,
                deps=deps,
                budget_tokens=budget_tokens,
            )
        )

    return specs


def _specs_to_plan_nodes(specs: list[_LLMNodeSpec]) -> list[PlanNode]:
    """Convert validated LLM node specs into :class:`PlanNode` objects.

    The LLM uses its own short ``id`` namespace (e.g. ``"t1"``).  This
    function allocates real UUIDs for each node and remaps dependency
    references accordingly.

    Args:
        specs: Validated list of :class:`_LLMNodeSpec` from
            :func:`_parse_llm_response`.

    Returns:
        List of :class:`PlanNode` ready for insertion into a :class:`Plan`.

    # @trace WL-087
    """
    # Build mapping from LLM-local id → real UUID so deps can be resolved.
    llm_id_to_uuid: dict[str, str] = {spec.id: str(uuid.uuid4()) for spec in specs}

    nodes: list[PlanNode] = []
    for spec in specs:
        resolved_deps = [llm_id_to_uuid[dep] for dep in spec.deps if dep in llm_id_to_uuid]
        metadata: dict[str, Any] = {}
        if spec.agent_hint is not None:
            metadata["agent_hint"] = spec.agent_hint
        if spec.budget_tokens is not None:
            metadata["budget_tokens"] = spec.budget_tokens

        node = PlanNode(
            id=llm_id_to_uuid[spec.id],
            task=spec.task,
            depends_on=resolved_deps,
            metadata=metadata,
        )
        nodes.append(node)

    return nodes


class LLMPlangentPlanner(PlangentPlanner):
    """PlangentPlanner subclass that uses a FlashAgent LLM call for decomposition.

    Overrides ``_generate_sub_tasks()`` to call a cheap model (haiku/flash)
    via :class:`~thegent.agents.flash_agent.FlashAgent`, parse its JSON
    output, and validate it against the OrchestrationPlan node schema.

    Fallback strategy (explicit, documented):
        When the model is unavailable (FlashAgent returns ``success=False``
        due to timeout or connectivity), the planner falls back to the
        parent :class:`PlangentPlanner` heuristic decomposition.  This is
        an intentional design decision: for the planning path, a degraded
        heuristic plan is acceptable when the LLM is unreachable.  The
        fallback is logged at WARNING level so it is always visible.
        Schema validation failures are NOT subject to fallback — they raise
        ``ValueError`` immediately (fail loud, fail fast).

    # @trace WL-087
    """

    def __init__(
        self,
        *,
        model: str = "claude-haiku-4.5",
        timeout_s: float = 30.0,
        max_tokens: int = 1024,
        separator: str = ".",
        max_nodes_per_level: int = 5,
    ) -> None:
        """Initialise the LLM-backed planner.

        Args:
            model: LiteLLM model identifier for the decomposition call.
                Defaults to ``claude-haiku-4.5`` (cheap, fast).
            timeout_s: Timeout in seconds for the FlashAgent call.
            max_tokens: Maximum tokens the LLM may produce.
            separator: Forwarded to :class:`PlangentPlanner`.
            max_nodes_per_level: Forwarded to :class:`PlangentPlanner`.

        # @trace WL-087
        """
        super().__init__(separator=separator, max_nodes_per_level=max_nodes_per_level)
        self._model = model
        self._timeout_s = timeout_s
        self._max_tokens = max_tokens

    # ------------------------------------------------------------------
    # Override
    # ------------------------------------------------------------------

    def _generate_sub_tasks(self, goal: str, max_depth: int) -> list[str]:
        """Decompose *goal* into sub-task strings via an LLM call.

        Calls :class:`~thegent.agents.flash_agent.FlashAgent` with a
        structured prompt that asks for a JSON breakdown of the goal.
        The JSON is parsed and validated; on success the task strings are
        extracted and returned to the parent :meth:`decompose` method.

        Fallback (model unavailable only):
            If FlashAgent returns ``success=False`` (timeout / connectivity),
            the parent heuristic is used and a WARNING is emitted.

        Raises:
            ValueError: If the LLM responds but its output fails schema
                validation.  This is a hard failure (no fallback).

        Args:
            goal: The (stripped) goal string.
            max_depth: Hint for maximum decomposition depth; forwarded to
                the prompt.

        Returns:
            Ordered list of sub-task strings ready for :meth:`decompose`.

        # @trace WL-087
        """
        nodes = asyncio.run(self._generate_plan_nodes(goal, max_depth))
        if nodes is None:
            # Model unavailable — explicit fallback to heuristic (documented above).
            _log.warning(
                "LLMPlangentPlanner: FlashAgent unavailable for goal=%r; "
                "falling back to heuristic decomposition (model=%s, timeout=%.1fs)",
                goal,
                self._model,
                self._timeout_s,
            )
            return super()._generate_sub_tasks(goal, max_depth)

        # Schema validation succeeded; return task strings.
        return [node.task for node in nodes]

    # ------------------------------------------------------------------
    # Async helper — can be called directly for OrchestrationPlan output
    # ------------------------------------------------------------------

    async def decompose_to_orchestration_plan(self, goal: str, max_depth: int = 3) -> "Any":
        """Decompose *goal* directly into an OrchestrationPlan with full metadata.

        Unlike :meth:`decompose`, which loses agent_hint / budget_tokens when
        converting back through the string list, this method builds the plan
        directly from the validated :class:`_LLMNodeSpec` objects.

        Args:
            goal: High-level goal statement.
            max_depth: Hint for maximum decomposition depth.

        Returns:
            An :class:`~thegent.orchestration.plan.OrchestrationPlan` with
            nodes populated from the LLM response, or a plan produced by the
            parent heuristic when the model is unavailable.

        Raises:
            ValueError: If the LLM output fails schema validation.

        # @trace WL-087
        """
        from thegent.orchestration.plan import OrchestrationPlan  # noqa: PLC0415

        if not goal or not goal.strip():
            raise ValueError("goal must be a non-empty string")

        goal = goal.strip()
        nodes = await self._generate_plan_nodes(goal, max_depth)

        if nodes is None:
            _log.warning(
                "LLMPlangentPlanner: FlashAgent unavailable for goal=%r; "
                "falling back to heuristic for OrchestrationPlan (model=%s)",
                goal,
                self._model,
            )
            # Heuristic path: call parent's _generate_sub_tasks directly to
            # avoid asyncio.run() nesting (decompose() calls asyncio.run inside
            # _generate_sub_tasks(), which would fail in an async context).
            sub_tasks = PlangentPlanner._generate_sub_tasks(self, goal, max_depth)
            oplan = OrchestrationPlan(goal=goal)
            prev_id: str | None = None
            for task_text in sub_tasks[: self._max_nodes_per_level * max_depth]:
                node = PlanNode(task=task_text, depends_on=[prev_id] if prev_id else [])
                oplan.nodes.append(node)
                prev_id = node.id
            return oplan

        oplan = OrchestrationPlan(goal=goal, nodes=nodes)
        _log.debug(
            "LLMPlangentPlanner: decomposed goal=%r into %d OrchestrationPlan nodes",
            goal,
            len(oplan.nodes),
        )
        return oplan

    # ------------------------------------------------------------------
    # Private
    # ------------------------------------------------------------------

    async def _generate_plan_nodes(self, goal: str, max_depth: int) -> list[PlanNode] | None:
        """Call FlashAgent, parse response, and return validated PlanNodes.

        Returns ``None`` when the model is unavailable (FlashAgent timeout /
        connectivity failure).  Raises ``ValueError`` on schema validation
        failures — those are hard errors regardless of model availability.

        Args:
            goal: The (stripped) goal string.
            max_depth: Depth hint forwarded to the prompt.

        Returns:
            List of :class:`PlanNode` on success, or ``None`` when the model
            is unavailable.

        Raises:
            ValueError: If LLM output fails schema validation.

        # @trace WL-087
        """
        prompt = _DECOMPOSITION_SYSTEM_PROMPT.format(max_depth=max(2, max_depth)) + f"\n\nGoal to decompose:\n{goal}"

        config = FlashAgentConfig(
            task_prompt=prompt,
            model=self._model,
            timeout_s=self._timeout_s,
            max_tokens=self._max_tokens,
        )

        agent = FlashAgent()
        result = await agent.run(config)

        if not result.success:
            # Model unavailable — caller decides on fallback.
            _log.warning(
                "LLMPlangentPlanner: FlashAgent call unsuccessful (agent_id=%s, elapsed=%.2fs)",
                result.agent_id,
                result.elapsed_s,
            )
            return None

        _log.debug(
            "LLMPlangentPlanner: FlashAgent returned %d chars (agent_id=%s)",
            len(result.output),
            result.agent_id,
        )

        # Parse + validate — ValueError propagates to caller as hard failure.
        specs = _parse_llm_response(result.output)
        nodes = _specs_to_plan_nodes(specs)

        _log.debug(
            "LLMPlangentPlanner: validated %d nodes from LLM output",
            len(nodes),
        )
        return nodes
