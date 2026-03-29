"""Tool primitives for SmolGents.

A :class:`Tool` is a named, callable unit of capability that a
:class:`~thegent.agents.smolgents.base.SmolAgent` can invoke during task
execution.  Keeping tools as simple dataclasses enables easy serialisation,
introspection, and test doubles.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import Callable


@dataclass
class Tool:
    """A single callable capability exposed to a SmolAgent.

    Attributes:
        name: Unique identifier used by the agent to select this tool.
        description: Human-readable explanation of what the tool does.
        func: The callable invoked with arbitrary positional/keyword args.
            Must return a value that can be coerced to ``str`` for
            inclusion in the agent's task output.
        metadata: Optional bag of extra data (e.g. tags, version, source).

    Example::

        def add(a: int, b: int) -> int:
            return a + b

        tool = Tool(name="add", description="Add two integers", func=add)
        result = tool("add", 1, 2)   # returns 3 via __call__
    """

    name: str
    description: str
    func: Callable[..., Any]
    metadata: dict[str, Any] = field(default_factory=dict)

    def __call__(self, *args: Any, **kwargs: Any) -> Any:
        """Invoke the underlying callable directly."""
        return self.func(*args, **kwargs)

    def __repr__(self) -> str:
        return f"Tool(name={self.name!r}, description={self.description!r})"
