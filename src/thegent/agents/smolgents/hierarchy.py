"""AgentTree — parent/child tracking for SmolGents hierarchies.

:class:`AgentTree` is a lightweight registry that maintains the relationship
between :class:`~thegent.agents.smolgents.base.SmolAgent` instances.  It is
intentionally separate from :class:`SmolAgent` so that the base class stays
pure (no circular imports, easy to test in isolation).

Integration note
----------------
For production use with capability-based routing and parallel dispatch, wire
the root :class:`SmolAgent` into the existing
:class:`~thegent.agents.hierarchy.AgentHierarchyManager` by attaching it as
the ``smolagent`` field of an :class:`~thegent.agents.hierarchy.AgentNode`.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from thegent.agents.smolgents.base import SmolAgent


class AgentTree:
    """Flat registry that tracks parent/child relationships between SmolAgents.

    The tree does **not** own the agents — callers are responsible for keeping
    references.  The tree only maintains a mapping of ``name -> agent`` and the
    directed parent/child edges so that traversal and lookup remain O(1).

    Usage::

        tree = AgentTree()
        parent = SmolAgent("orchestrator", tools=[...])
        child  = SmolAgent("specialist",   tools=[...])

        tree.register(parent)
        tree.register(child, parent_name="orchestrator")

        assert tree.get("specialist") is child
        assert tree.get_children("orchestrator") == [child]
        assert tree.get_parent("specialist") is parent
    """

    def __init__(self) -> None:
        self._agents: dict[str, SmolAgent] = {}
        self._parent: dict[str, str] = {}  # child_name -> parent_name
        self._children: dict[str, list[str]] = {}  # parent_name -> [child_names]

    # ------------------------------------------------------------------
    # Registration
    # ------------------------------------------------------------------

    def register(
        self,
        agent: SmolAgent,
        *,
        parent_name: str | None = None,
    ) -> None:
        """Register an agent, optionally linking it to a parent.

        Args:
            agent: The :class:`SmolAgent` to register.
            parent_name: Name of the parent agent already registered in this
                tree.  Pass *None* for the root / orphan nodes.

        Raises:
            ValueError: If ``agent.name`` is already registered.
            ValueError: If ``parent_name`` is specified but not yet registered.
        """
        if agent.name in self._agents:
            raise ValueError(f"Agent '{agent.name}' is already registered in this tree")

        if parent_name is not None and parent_name not in self._agents:
            raise ValueError(f"Parent agent '{parent_name}' not found in tree; register the parent first")

        self._agents[agent.name] = agent
        self._children.setdefault(agent.name, [])

        if parent_name is not None:
            self._parent[agent.name] = parent_name
            self._children[parent_name].append(agent.name)
            # Sync the agent's own parent reference using the public setter
            agent.set_parent(self._agents[parent_name])

    # ------------------------------------------------------------------
    # Lookup
    # ------------------------------------------------------------------

    def get(self, name: str) -> SmolAgent | None:
        """Return the agent registered under *name*, or *None*."""
        return self._agents.get(name)

    def get_parent(self, name: str) -> SmolAgent | None:
        """Return the direct parent of the agent named *name*, or *None*."""
        parent_name = self._parent.get(name)
        if parent_name is None:
            return None
        return self._agents.get(parent_name)

    def get_children(self, name: str) -> list[SmolAgent]:
        """Return direct children of the agent named *name*."""
        child_names = self._children.get(name, [])
        return [self._agents[n] for n in child_names if n in self._agents]

    def list_agents(self) -> list[SmolAgent]:
        """Return all registered agents in insertion order."""
        return list(self._agents.values())

    # ------------------------------------------------------------------
    # Traversal helpers
    # ------------------------------------------------------------------

    def get_ancestors(self, name: str) -> list[SmolAgent]:
        """Return ancestors from direct parent up to the root (inclusive)."""
        ancestors: list[SmolAgent] = []
        current = name
        while current in self._parent:
            current = self._parent[current]
            agent = self._agents.get(current)
            if agent:
                ancestors.append(agent)
        return ancestors

    def get_descendants(self, name: str) -> list[SmolAgent]:
        """Return all descendants (recursive) of the named agent."""
        result: list[SmolAgent] = []

        def _collect(n: str) -> None:
            for child_name in self._children.get(n, []):
                child = self._agents.get(child_name)
                if child:
                    result.append(child)
                    _collect(child_name)

        _collect(name)
        return result

    def to_dict(self) -> dict[str, Any]:
        """Return a serialisable representation of the tree."""
        return {
            "agents": list(self._agents.keys()),
            "edges": [{"child": child, "parent": parent} for child, parent in self._parent.items()],
        }

    def __len__(self) -> int:
        return len(self._agents)

    def __repr__(self) -> str:
        return f"AgentTree(agents={list(self._agents.keys())})"
