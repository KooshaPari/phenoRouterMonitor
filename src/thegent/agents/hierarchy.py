"""Agent Hierarchy MVP using SmolAgents framework.

Implements an orchestrator -> specialist hierarchy manager that:
- Manages a tree of agents (orchestrator -> specialists)
- Routes tasks to appropriate agents by capability
- Supports parallel agent execution via asyncio
- Tracks agent state and results
- Integrates with SmolAgents CodeAgent / ToolCallingAgent / MultiStepAgent
"""

from __future__ import annotations

import asyncio
import logging
import time
import uuid
from dataclasses import dataclass, field
from enum import Enum, auto
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import Callable

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Enums
# ---------------------------------------------------------------------------


class AgentCapability(Enum):
    """Domain capabilities an agent can handle."""

    CODE = auto()
    RESEARCH = auto()
    REVIEW = auto()
    TEST = auto()
    DEPLOY = auto()
    PLAN = auto()
    SECURITY = auto()
    DATA = auto()
    DOCUMENTATION = auto()
    ORCHESTRATE = auto()


class AgentState(Enum):
    """Lifecycle state of a single agent node."""

    IDLE = "idle"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"


class RoutingStrategy(Enum):
    """How the hierarchy manager selects an agent for a task."""

    CAPABILITY_MATCH = "capability_match"  # Pick first agent with matching capability
    ROUND_ROBIN = "round_robin"  # Cycle through capable agents
    LEAST_LOADED = "least_loaded"  # Pick agent with fewest active tasks


# ---------------------------------------------------------------------------
# Data classes
# ---------------------------------------------------------------------------


@dataclass
class TaskResult:
    """Outcome of a single task dispatched to an agent."""

    task_id: str
    agent_id: str
    success: bool
    output: Any = None
    error: str | None = None
    duration_seconds: float = 0.0
    metadata: dict[str, Any] = field(default_factory=dict)


@dataclass
class AgentNode:
    """A node in the agent hierarchy tree.

    Attributes:
        agent_id: Unique identifier for this node.
        capabilities: Set of domain capabilities this agent provides.
        model: Model name / identifier passed to SmolAgents (e.g., "HfApiModel").
        name: Human-readable name (used as SmolAgents managed-agent name).
        description: Brief description of this agent's purpose.
        parent_id: Parent agent node id, None for root/orchestrator.
        children: Direct child node ids.
        smolagent: Optional live SmolAgents agent instance.
        state: Current lifecycle state.
        active_task_count: Number of currently running tasks.
        total_tasks_completed: Historical count of completed tasks.
        metadata: Arbitrary extra data.
    """

    agent_id: str = field(default_factory=lambda: str(uuid.uuid4()))
    capabilities: set[AgentCapability] = field(default_factory=set)
    model: str | None = None
    name: str = ""
    description: str = ""
    parent_id: str | None = None
    children: list[str] = field(default_factory=list)
    smolagent: Any | None = field(default=None, compare=False, repr=False)
    state: AgentState = AgentState.IDLE
    active_task_count: int = 0
    total_tasks_completed: int = 0
    metadata: dict[str, Any] = field(default_factory=dict)

    def __post_init__(self) -> None:
        if not self.name:
            self.name = f"agent-{self.agent_id[:8]}"

    def has_capability(self, capability: AgentCapability) -> bool:
        """Return True if this agent has the given capability."""
        return capability in self.capabilities

    def has_any_capability(self, capabilities: set[AgentCapability]) -> bool:
        """Return True if this agent has any of the given capabilities."""
        return bool(self.capabilities & capabilities)

    def to_dict(self) -> dict[str, Any]:
        """Serialize to plain dict (excludes live smolagent instance)."""
        return {
            "agent_id": self.agent_id,
            "capabilities": [c.name for c in self.capabilities],
            "model": self.model,
            "name": self.name,
            "description": self.description,
            "parent_id": self.parent_id,
            "children": list(self.children),
            "state": self.state.value,
            "active_task_count": self.active_task_count,
            "total_tasks_completed": self.total_tasks_completed,
            "metadata": self.metadata,
        }


# ---------------------------------------------------------------------------
# AgentHierarchyManager
# ---------------------------------------------------------------------------


class AgentHierarchyManager:
    """Manages a tree of agents, routes tasks, and collects results.

    Architecture
    ------------
    The manager owns an in-memory registry of AgentNode objects that form a
    DAG (typically a tree rooted at an *orchestrator* agent).  It exposes:

    * ``spawn_agent`` — register a new node, optionally wrapping a SmolAgents
      instance so that the orchestrator can delegate to it.
    * ``route_task`` — select the best-fit agent for a task given required
      capabilities and the configured RoutingStrategy.
    * ``execute_task`` — run a task synchronously on a chosen agent and record
      the result in the internal ledger.
    * ``execute_parallel`` — run multiple tasks concurrently using asyncio.
    * ``collect_results`` — retrieve all stored TaskResults, optionally filtered.

    SmolAgents Integration
    ----------------------
    When ``smolagent`` is supplied to :py:meth:`spawn_agent`, the node is treated
    as a real SmolAgents ``MultiStepAgent`` (or subclass).  The orchestrator can
    then call ``smolagent.run(prompt)`` automatically inside ``execute_task``.
    If no ``smolagent`` is attached, the manager uses the optional
    ``task_executor`` callback (useful for testing and simulation).
    """

    def __init__(
        self,
        routing_strategy: RoutingStrategy = RoutingStrategy.CAPABILITY_MATCH,
        task_executor: Callable[[AgentNode, str, dict[str, Any]], TaskResult] | None = None,
    ) -> None:
        """Initialise the hierarchy manager.

        Args:
            routing_strategy: Algorithm used to select agents when routing tasks.
            task_executor: Optional callback executed when running a task on a
                node.  Signature: ``(node, task_description, context) -> TaskResult``.
                If *None* and no ``smolagent`` is attached, ``execute_task``
                raises ``RuntimeError``.
        """
        self._nodes: dict[str, AgentNode] = {}
        self._root_id: str | None = None
        self._results: dict[str, TaskResult] = {}
        self.routing_strategy = routing_strategy
        self.task_executor = task_executor
        self._round_robin_index: dict[AgentCapability, int] = {}

    # ------------------------------------------------------------------
    # Node management
    # ------------------------------------------------------------------

    def spawn_agent(
        self,
        capabilities: set[AgentCapability],
        *,
        agent_id: str | None = None,
        parent_id: str | None = None,
        name: str = "",
        description: str = "",
        model: str | None = None,
        smolagent: Any | None = None,
        metadata: dict[str, Any] | None = None,
    ) -> AgentNode:
        """Create and register a new agent node.

        Args:
            capabilities: Set of domain capabilities this agent provides.
            agent_id: Optional explicit ID; auto-generated if not supplied.
            parent_id: ID of the parent node; if *None* and no root exists yet,
                this node becomes the root.
            name: Human-readable label.
            description: Short description of the agent's role.
            model: Model identifier forwarded to SmolAgents.
            smolagent: Optional live SmolAgents agent instance.
            metadata: Extra key/value pairs attached to the node.

        Returns:
            The newly created :class:`AgentNode`.

        Raises:
            ValueError: If ``parent_id`` is specified but does not exist.
        """
        if agent_id and agent_id in self._nodes:
            raise ValueError(f"Agent '{agent_id}' already registered")

        if parent_id and parent_id not in self._nodes:
            raise ValueError(f"Parent agent '{parent_id}' not found in hierarchy")

        node = AgentNode(
            agent_id=agent_id or str(uuid.uuid4()),
            capabilities=capabilities,
            model=model,
            name=name or f"agent-{(agent_id or '')[:8]}",
            description=description,
            parent_id=parent_id,
            smolagent=smolagent,
            metadata=metadata or {},
        )

        self._nodes[node.agent_id] = node

        # Wire parent <-> child
        if parent_id:
            self._nodes[parent_id].children.append(node.agent_id)
        elif self._root_id is None:
            # First registered node becomes the root orchestrator
            self._root_id = node.agent_id

        logger.debug("Spawned agent %s (caps=%s)", node.agent_id, capabilities)
        return node

    def get_node(self, agent_id: str) -> AgentNode | None:
        """Return a node by ID, or *None* if not found."""
        return self._nodes.get(agent_id)

    def get_root(self) -> AgentNode | None:
        """Return the root orchestrator node, or *None* if empty."""
        if self._root_id:
            return self._nodes.get(self._root_id)
        return None

    def list_agents(self) -> list[AgentNode]:
        """Return all registered agent nodes."""
        return list(self._nodes.values())

    def remove_agent(self, agent_id: str) -> bool:
        """Remove a node (and unlink from parent/children).

        Returns *True* if the node existed and was removed.
        """
        node = self._nodes.pop(agent_id, None)
        if not node:
            return False

        # Unlink from parent
        if node.parent_id and node.parent_id in self._nodes:
            parent = self._nodes[node.parent_id]
            if agent_id in parent.children:
                parent.children.remove(agent_id)

        # Reparent children to removed node's parent (or mark as orphan)
        for child_id in node.children:
            child = self._nodes.get(child_id)
            if child:
                child.parent_id = node.parent_id

        if self._root_id == agent_id:
            self._root_id = None

        return True

    # ------------------------------------------------------------------
    # Routing
    # ------------------------------------------------------------------

    def route_task(
        self,
        required_capabilities: set[AgentCapability],
        *,
        exclude_ids: set[str] | None = None,
    ) -> AgentNode | None:
        """Select the best-fit agent for a set of required capabilities.

        Args:
            required_capabilities: Capabilities the chosen agent must have.
            exclude_ids: Optional set of agent IDs to exclude from selection.

        Returns:
            The selected :class:`AgentNode`, or *None* if no match found.
        """
        exclude = exclude_ids or set()
        candidates = [
            n
            for n in self._nodes.values()
            if n.agent_id not in exclude
            and n.state in (AgentState.IDLE, AgentState.RUNNING)
            and n.has_any_capability(required_capabilities)
        ]

        if not candidates:
            return None

        if self.routing_strategy == RoutingStrategy.CAPABILITY_MATCH:
            # Prefer exact full-match first, then partial
            full_match = [c for c in candidates if required_capabilities.issubset(c.capabilities)]
            return full_match[0] if full_match else candidates[0]

        if self.routing_strategy == RoutingStrategy.LEAST_LOADED:
            return min(candidates, key=lambda n: n.active_task_count)

        if self.routing_strategy == RoutingStrategy.ROUND_ROBIN:
            # Use the first required capability as the bucket key
            key = next(iter(required_capabilities))
            idx = self._round_robin_index.get(key, 0) % len(candidates)
            self._round_robin_index[key] = idx + 1
            return candidates[idx]

        return candidates[0]

    # ------------------------------------------------------------------
    # Task execution
    # ------------------------------------------------------------------

    def execute_task(
        self,
        agent_id: str,
        task_description: str,
        context: dict[str, Any] | None = None,
        *,
        task_id: str | None = None,
    ) -> TaskResult:
        """Execute a task on a specific agent synchronously.

        Execution order:
        1. If the node has a live ``smolagent``, call ``smolagent.run(task_description)``.
        2. Else if a ``task_executor`` callback is configured, use it.
        3. Otherwise raise ``RuntimeError``.

        The node's state and counters are updated around execution.

        Args:
            agent_id: ID of the target agent node.
            task_description: Natural-language task prompt.
            context: Optional dict passed to the executor.
            task_id: Optional explicit task ID; auto-generated if absent.

        Returns:
            :class:`TaskResult` with success/failure information.

        Raises:
            ValueError: If the agent ID is not registered.
            RuntimeError: If neither smolagent nor task_executor is available.
        """
        node = self._nodes.get(agent_id)
        if not node:
            raise ValueError(f"Agent '{agent_id}' not found")

        tid = task_id or str(uuid.uuid4())
        ctx = context or {}
        start = time.monotonic()

        node.active_task_count += 1
        node.state = AgentState.RUNNING

        # Validate configuration before entering the try/except so that
        # RuntimeError (programming errors) propagates to the caller rather
        # than being swallowed as a task failure.
        if node.smolagent is None and self.task_executor is None:
            node.active_task_count = max(0, node.active_task_count - 1)
            raise RuntimeError(f"No smolagent attached and no task_executor configured for agent '{agent_id}'")

        try:
            if node.smolagent is not None:
                # SmolAgents integration path
                raw_output = node.smolagent.run(task_description, **(ctx.get("smolagent_kwargs", {})))
                result = TaskResult(
                    task_id=tid,
                    agent_id=agent_id,
                    success=True,
                    output=raw_output,
                    duration_seconds=time.monotonic() - start,
                )
            else:
                # task_executor path (validated above)
                result = self.task_executor(node, task_description, ctx)  # type: ignore[misc]
                result.task_id = tid
                result.agent_id = agent_id
                result.duration_seconds = time.monotonic() - start

            node.state = AgentState.IDLE if result.success else AgentState.FAILED
            if result.success:
                node.total_tasks_completed += 1

        except Exception as exc:
            result = TaskResult(
                task_id=tid,
                agent_id=agent_id,
                success=False,
                error=str(exc),
                duration_seconds=time.monotonic() - start,
            )
            node.state = AgentState.FAILED
        finally:
            node.active_task_count = max(0, node.active_task_count - 1)

        self._results[tid] = result
        logger.debug(
            "Task %s on agent %s: success=%s duration=%.2fs",
            tid,
            agent_id,
            result.success,
            result.duration_seconds,
        )
        return result

    async def _execute_task_async(
        self,
        agent_id: str,
        task_description: str,
        context: dict[str, Any] | None = None,
        task_id: str | None = None,
    ) -> TaskResult:
        """Async wrapper around execute_task (runs in executor thread)."""
        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(
            None,
            lambda: self.execute_task(agent_id, task_description, context, task_id=task_id),
        )

    def execute_parallel(
        self,
        tasks: list[dict[str, Any]],
    ) -> list[TaskResult]:
        """Execute multiple tasks concurrently using asyncio.

        Each element of ``tasks`` is a dict with:
        * ``agent_id`` (str, required) — target agent.
        * ``task_description`` (str, required) — prompt.
        * ``context`` (dict, optional) — extra context.
        * ``task_id`` (str, optional) — explicit task ID.

        Args:
            tasks: List of task dicts as described above.

        Returns:
            List of :class:`TaskResult` objects in the same order as ``tasks``.
        """

        async def _run_all() -> list[TaskResult]:
            coros = [
                self._execute_task_async(
                    t["agent_id"],
                    t["task_description"],
                    t.get("context"),
                    t.get("task_id"),
                )
                for t in tasks
            ]
            return list(await asyncio.gather(*coros, return_exceptions=False))

        try:
            loop = asyncio.get_event_loop()
            if loop.is_running():
                # In async context, create a new loop in thread
                import concurrent.futures

                with concurrent.futures.ThreadPoolExecutor(max_workers=1) as pool:
                    future = pool.submit(asyncio.run, _run_all())
                    return future.result()
            return loop.run_until_complete(_run_all())
        except RuntimeError:
            return asyncio.run(_run_all())

    # ------------------------------------------------------------------
    # Result collection
    # ------------------------------------------------------------------

    def collect_results(
        self,
        *,
        agent_id: str | None = None,
        success_only: bool = False,
    ) -> list[TaskResult]:
        """Return collected task results.

        Args:
            agent_id: If set, filter results to this agent only.
            success_only: If True, return only successful results.

        Returns:
            Filtered list of :class:`TaskResult`.
        """
        results = list(self._results.values())
        if agent_id:
            results = [r for r in results if r.agent_id == agent_id]
        if success_only:
            results = [r for r in results if r.success]
        return results

    def get_result(self, task_id: str) -> TaskResult | None:
        """Retrieve a single result by task ID."""
        return self._results.get(task_id)

    def clear_results(self) -> None:
        """Remove all stored task results."""
        self._results.clear()

    # ------------------------------------------------------------------
    # Tree traversal helpers
    # ------------------------------------------------------------------

    def get_children(self, agent_id: str) -> list[AgentNode]:
        """Return direct children of the given agent."""
        node = self._nodes.get(agent_id)
        if not node:
            return []
        return [self._nodes[c] for c in node.children if c in self._nodes]

    def get_ancestors(self, agent_id: str) -> list[AgentNode]:
        """Return all ancestors from direct parent up to root."""
        ancestors: list[AgentNode] = []
        current = self._nodes.get(agent_id)
        while current and current.parent_id:
            parent = self._nodes.get(current.parent_id)
            if not parent:
                break
            ancestors.append(parent)
            current = parent
        return ancestors

    def get_descendants(self, agent_id: str) -> list[AgentNode]:
        """Return all descendants (recursive) of the given agent."""
        result: list[AgentNode] = []

        def _collect(nid: str) -> None:
            node = self._nodes.get(nid)
            if not node:
                return
            for cid in node.children:
                child = self._nodes.get(cid)
                if child:
                    result.append(child)
                    _collect(cid)

        _collect(agent_id)
        return result

    def get_hierarchy_tree(self, root_id: str | None = None) -> dict[str, Any]:
        """Return a nested dict representation of the hierarchy tree.

        Args:
            root_id: Starting node ID; defaults to the registered root.

        Returns:
            Nested dict with ``agent_id``, ``name``, ``capabilities``,
            ``state``, and ``children`` keys.
        """
        start_id = root_id or self._root_id
        if not start_id or start_id not in self._nodes:
            return {}

        def _build(nid: str) -> dict[str, Any]:
            node = self._nodes[nid]
            return {
                "agent_id": nid,
                "name": node.name,
                "capabilities": [c.name for c in node.capabilities],
                "state": node.state.value,
                "children": [_build(cid) for cid in node.children if cid in self._nodes],
            }

        return _build(start_id)

    # ------------------------------------------------------------------
    # Metrics / summary
    # ------------------------------------------------------------------

    def summary(self) -> dict[str, Any]:
        """Return a high-level summary of the hierarchy state."""
        all_results = list(self._results.values())
        return {
            "total_agents": len(self._nodes),
            "root_id": self._root_id,
            "routing_strategy": self.routing_strategy.value,
            "total_tasks_run": len(all_results),
            "total_tasks_succeeded": sum(1 for r in all_results if r.success),
            "total_tasks_failed": sum(1 for r in all_results if not r.success),
            "agents": [n.to_dict() for n in self._nodes.values()],
        }
