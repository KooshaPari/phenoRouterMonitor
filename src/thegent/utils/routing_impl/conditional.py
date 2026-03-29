"""GW-56: Conditional routing — route by metadata, params, or url with operators.

Supports $eq, $in, $regex, $and, $or operators (Portkey-compatible).

# @trace FR-AROUTE-056
"""

from __future__ import annotations

import logging
import re
from dataclasses import dataclass

_log = logging.getLogger(__name__)


@dataclass
class ConditionalRoute:
    """A conditional routing rule mapping a condition to a target model/provider."""

    condition: dict
    target: str  # model or provider to route to
    name: str = ""


def evaluate_condition(condition: dict, context: dict) -> bool:
    """Evaluate a routing condition against a context dict.

    Args:
        condition: Condition dict using MongoDB-style operators.
        context: Flat dict of key→value (e.g., {"metadata.user_id": "u123"}).

    Returns:
        True if the condition matches the context, False otherwise.

    Raises:
        ValueError: If an unknown operator is encountered.
    """
    # Handle logical operators at the top level
    if "$and" in condition:
        operands = condition["$and"]
        return all(evaluate_condition(sub, context) for sub in operands)

    if "$or" in condition:
        operands = condition["$or"]
        return any(evaluate_condition(sub, context) for sub in operands)

    # Field-level operators: {"field": {"$op": value}}
    for field_name, ops in condition.items():
        if not isinstance(ops, dict):
            # Plain equality shorthand: {"field": value}
            return context.get(field_name) == ops

        for op, operand in ops.items():
            field_value = context.get(field_name)

            if op == "$eq":
                if field_value != operand:
                    return False

            elif op == "$ne":
                if field_value == operand:
                    return False

            elif op == "$in":
                if field_value not in operand:
                    return False

            elif op == "$nin":
                if field_value in operand:
                    return False

            elif op == "$regex":
                if field_value is None:
                    return False
                if not re.search(operand, str(field_value), re.IGNORECASE):
                    return False

            elif op == "$exists":
                key_present = field_name in context
                if bool(operand) != key_present:
                    return False

            else:
                raise ValueError(f"Unknown operator: {op!r}")

    return True


def match_conditional_route(
    routes: list[ConditionalRoute],
    context: dict,
) -> ConditionalRoute | None:
    """Return the first route whose condition matches the context, or None.

    Args:
        routes: Ordered list of ConditionalRoute instances to evaluate.
        context: Flat routing context dict.

    Returns:
        The first matching ConditionalRoute, or None if none match.
    """
    for route in routes:
        try:
            if evaluate_condition(route.condition, context):
                _log.debug(
                    "Conditional route matched: name=%r target=%r",
                    route.name,
                    route.target,
                )
                return route
        except ValueError:
            _log.exception("Error evaluating condition for route name=%r", route.name)
            raise
    return None


def build_routing_context(
    request_body: dict,
    metadata: dict | None = None,
) -> dict:
    """Build a flat routing context dict from a request body and optional metadata.

    Body fields are prefixed with "params."; metadata fields with "metadata.".

    Args:
        request_body: The raw request body dict (e.g., LiteLLM/OpenAI payload).
        metadata: Optional metadata dict to include under "metadata." prefix.

    Returns:
        Flat dict usable as context for evaluate_condition.
    """
    ctx: dict = {}

    for key, value in request_body.items():
        ctx[f"params.{key}"] = value

    if metadata:
        for key, value in metadata.items():
            ctx[f"metadata.{key}"] = value

    return ctx
