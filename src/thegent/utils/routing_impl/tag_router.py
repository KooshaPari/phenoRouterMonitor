"""GW-58: Tag-based routing — route free_tier vs paid_tier to different deployments.

# @trace FR-AROUTE-058
"""

from __future__ import annotations

import logging
from dataclasses import dataclass

_log = logging.getLogger(__name__)


@dataclass
class TagRoute:
    """A tag-based routing rule.

    A route matches when ALL of its tags are present in the request's tags.
    Higher priority wins when multiple routes match.
    """

    tags: list[str]  # request must have ALL these tags
    target: str  # model/deployment to route to
    priority: int = 0  # higher priority wins on tie


class TagRouter:
    """Routes requests based on tag matching.

    Maintains a list of TagRoute instances and resolves a target model/deployment
    based on which routes' tags are all present in the request's tags.
    """

    def __init__(self) -> None:
        self._routes: list[TagRoute] = []

    def register(self, route: TagRoute) -> None:
        """Register a TagRoute with this router.

        Args:
            route: The TagRoute to register.
        """
        self._routes.append(route)
        _log.debug(
            "Registered tag route: tags=%r target=%r priority=%d",
            route.tags,
            route.target,
            route.priority,
        )

    def resolve(self, request_tags: list[str]) -> str | None:
        """Return the target of the highest-priority matching route.

        A route matches if ALL of its tags are present in request_tags.
        When multiple routes match, the one with the highest priority wins.
        Ties are broken by registration order (first registered wins).

        Args:
            request_tags: List of string tags present on the request.

        Returns:
            The target model/deployment string, or None if no route matches.
        """
        tag_set = set(request_tags)
        matching: list[TagRoute] = [route for route in self._routes if all(tag in tag_set for tag in route.tags)]

        if not matching:
            return None

        best = max(matching, key=lambda r: r.priority)
        _log.debug(
            "Tag route resolved: target=%r priority=%d from %d candidates",
            best.target,
            best.priority,
            len(matching),
        )
        return best.target


def extract_request_tags(body: dict) -> list[str]:
    """Extract routing tags from a request body.

    Reads the "tg_tags" field which must be a list of strings.

    Args:
        body: The raw request body dict.

    Returns:
        List of string tags, or empty list if not present.
    """
    return body.get("tg_tags", [])
