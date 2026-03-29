"""SmolAgent — base class for the SmolGents lightweight agent hierarchy.

Design principles
-----------------
* **Smol by design**: no network calls, no LLM API keys, no heavy deps.
  The agent executes tasks by selecting and calling :class:`~thegent.agents.smolgents.tools.Tool`
  objects registered at construction time.
* **Composable**: a ``SmolAgent`` can delegate subtasks to child agents
  via :py:meth:`delegate`.  Children are standalone ``SmolAgent`` instances
  created on demand (or supplied explicitly).
* **Stateful memory**: per-instance dict-based memory accessible via
  :py:meth:`remember` / :py:meth:`recall`.
* **AgentRunner compatible**: the ``run(task)`` signature is intentionally
  kept compatible with the broader thegent ``AgentRunner`` pattern (same
  return type ``str``), enabling a ``SmolAgent`` to be plugged into an
  :class:`~thegent.agents.hierarchy.AgentNode` as its ``smolagent`` field.
"""

from __future__ import annotations

import logging
import uuid
from dataclasses import dataclass, field
from datetime import UTC, datetime
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from thegent.agents.smolgents.tools import Tool

logger = logging.getLogger(__name__)


@dataclass
class SmolGentJob:
    """A structured task/job for a SmolAgent to execute.

    Attributes:
        id: Unique job identifier.
        description: Natural-language task description.
        metadata: Additional job metadata.
        created_at: When the job was created.
    """

    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    description: str = ""
    metadata: dict[str, Any] = field(default_factory=dict)
    created_at: datetime = field(default_factory=lambda: datetime.now(UTC))


@dataclass
class SmolGentResult:
    """The result of a SmolGentJob execution.

    Attributes:
        job_id: ID of the job that produced this result.
        output: String output from the agent.
        success: Whether the job completed successfully.
        error: Error message if failed.
        completed_at: When the job finished.
        metadata: Additional result metadata.
    """

    job_id: str
    output: str = ""
    success: bool = True
    error: str | None = None
    completed_at: datetime = field(default_factory=lambda: datetime.now(UTC))
    metadata: dict[str, Any] = field(default_factory=dict)


class SmolAgent:
    """Minimalist agent that executes tasks via registered tools.

    Args:
        name: Unique human-readable identifier for this agent.
        tools: List of :class:`~thegent.agents.smolgents.tools.Tool` objects
            available to this agent during ``run`` and ``delegate``.
        memory: Optional pre-populated dict to use as the agent's memory
            store.  If *None*, an empty dict is created.
        parent: Optional parent ``SmolAgent`` in the delegation hierarchy.

    Example::

        def echo(msg: str) -> str:
            return msg

        agent = SmolAgent(
            name="echo-agent",
            tools=[Tool("echo", "Echo a message", echo)],
        )
        result = agent.run("echo hello")
        # result == "hello"
    """

    def __init__(
        self,
        name: str,
        tools: list[Tool],
        *,
        memory: dict[str, Any] | None = None,
        parent: SmolAgent | None = None,
    ) -> None:
        self.name = name
        self._tools: dict[str, Tool] = {t.name: t for t in tools}
        self._memory: dict[str, Any] = memory if memory is not None else {}
        self._parent: SmolAgent | None = parent
        self._children: list[SmolAgent] = []

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def run(self, task: str | SmolGentJob) -> str | SmolGentResult:
        """Execute *task* by selecting an appropriate tool.

        The selection algorithm is intentionally simple: the first tool
        whose *name* appears as a token in the task string is chosen.  If
        no match is found, the task string itself is returned unchanged
        (pass-through behaviour).

        If a :class:`SmolGentJob` is provided, the agent will execute it
        and return a :class:`SmolGentResult` instead of a string.

        Args:
            task: Natural-language task description or structured job.

        Returns:
            String output from the matched tool (if task is a string),
            or :class:`SmolGentResult` (if task is a :class:`SmolGentJob`).
        """
        if isinstance(task, SmolGentJob):
            return self.execute_job(task)

        logger.debug("SmolAgent '%s' running task: %s", self.name, task)
        tool = self._select_tool(task)
        if tool is None:
            logger.debug(
                "SmolAgent '%s': no matching tool for task '%s'; returning task as-is",
                self.name,
                task,
            )
            return task

        logger.debug("SmolAgent '%s': selected tool '%s'", self.name, tool.name)
        result = tool(task)
        return str(result)

    def execute_job(self, job: SmolGentJob) -> SmolGentResult:
        """Execute a structured :class:`SmolGentJob`.

        This method is the structured alternative to :py:meth:`run`.

        Args:
            job: The :class:`SmolGentJob` to execute.

        Returns:
            A :class:`SmolGentResult` wrapping the agent's output.
        """
        logger.debug("SmolAgent '%s' executing job '%s'", self.name, job.id)
        try:
            # Re-use run() for the core logic (extract description as task)
            output = self.run(job.description)
            # If run returned a result (e.g. delegated job), extract its output
            if isinstance(output, SmolGentResult):
                output = output.output

            return SmolGentResult(
                job_id=job.id,
                output=str(output),
                success=True,
            )
        except Exception as e:
            logger.error("SmolAgent '%s': job '%s' failed: %s", self.name, job.id, e)
            return SmolGentResult(
                job_id=job.id,
                output="",
                success=False,
                error=str(e),
            )

    def delegate(self, sub_task: str | SmolGentJob) -> str | SmolGentResult:
        """Spawn a child SmolAgent to handle *sub_task*.

        A new ``SmolAgent`` is created with the same tool set as this
        agent and with *self* as its parent.  The child is appended to
        ``self._children`` for later inspection (e.g. by
        :class:`~thegent.agents.smolgents.hierarchy.AgentTree`).

        Args:
            sub_task: The subtask to hand off to the child agent.

        Returns:
            The string result or :class:`SmolGentResult` produced by the child.
        """
        child_name = f"{self.name}.child-{len(self._children)}"
        child = SmolAgent(
            name=child_name,
            tools=list(self._tools.values()),
            parent=self,
        )
        self._children.append(child)
        logger.debug(
            "SmolAgent '%s': delegating sub_task to child '%s'",
            self.name,
            child_name,
        )
        return child.run(sub_task)

    def remember(self, key: str, value: Any) -> None:
        """Store *value* under *key* in this agent's memory.

        Args:
            key: Memory key (string).
            value: Arbitrary value to store.
        """
        self._memory[key] = value
        logger.debug("SmolAgent '%s': remember key='%s'", self.name, key)

    def recall(self, key: str) -> Any:
        """Retrieve the value stored under *key*, or *None* if absent.

        Args:
            key: Memory key to look up.

        Returns:
            The stored value, or *None* if the key is not in memory.
        """
        return self._memory.get(key)

    # ------------------------------------------------------------------
    # Hierarchy wiring (called by AgentTree.register)
    # ------------------------------------------------------------------

    def set_parent(self, parent: SmolAgent) -> None:
        """Set the parent reference.  Called by :class:`AgentTree` during registration.

        Args:
            parent: The parent ``SmolAgent`` to link to this agent.
        """
        self._parent = parent

    # ------------------------------------------------------------------
    # Accessors
    # ------------------------------------------------------------------

    @property
    def parent(self) -> SmolAgent | None:
        """The parent agent in the hierarchy, or *None* if this is a root."""
        return self._parent

    @property
    def children(self) -> list[SmolAgent]:
        """Children spawned via :py:meth:`delegate` (shallow copy)."""
        return list(self._children)

    @property
    def tools(self) -> list[Tool]:
        """All tools available to this agent."""
        return list(self._tools.values())

    def get_tool(self, name: str) -> Tool | None:
        """Return the tool with the given *name*, or *None*."""
        return self._tools.get(name)

    def add_tool(self, tool: Tool) -> None:
        """Register an additional tool at runtime.

        Args:
            tool: The :class:`~thegent.agents.smolgents.tools.Tool` to add.

        Raises:
            ValueError: If a tool with the same name already exists.
        """
        if tool.name in self._tools:
            raise ValueError(f"Tool '{tool.name}' already registered on agent '{self.name}'")
        self._tools[tool.name] = tool

    @property
    def memory(self) -> dict[str, Any]:
        """Shallow copy of the agent's current memory dict."""
        return dict(self._memory)

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _select_tool(self, task: str) -> Tool | None:
        """Select the first tool whose name appears in *task*.

        Performs a case-insensitive substring search so that natural-language
        tasks like ``"Please use the search tool to find X"`` will match a
        tool named ``"search"``.

        Args:
            task: Task description to search for tool names in.

        Returns:
            Matching :class:`~thegent.agents.smolgents.tools.Tool`, or *None*.
        """
        task_lower = task.lower()
        for tool_name, tool in self._tools.items():
            if tool_name.lower() in task_lower:
                return tool
        return None

    def __repr__(self) -> str:
        return (
            f"SmolAgent(name={self.name!r}, "
            f"tools={list(self._tools.keys())!r}, "
            f"parent={self._parent.name if self._parent else None!r})"
        )
