"""GW-41: Request/response transforms including middle-out context compression.

middle-out: Compresses long message histories by summarizing middle messages
while preserving system prompt, first user message, and recent messages.

# @trace FR-REQEXT-041
"""

from __future__ import annotations


def extract_transforms(body: dict) -> list[str]:
    """Extract transforms list from request body.

    Returns body.get("transforms", []).
    """
    return body.get("transforms", [])


def apply_middle_out(messages: list[dict], max_messages: int = 20) -> list[dict]:
    """Compress messages list using middle-out strategy.

    If len(messages) <= max_messages: return unchanged.
    Otherwise:
    - Keep: system messages (all), first user message, last (max_messages // 2) messages
    - Replace middle with a single synthetic assistant message summarizing count
      {"role": "assistant", "content": f"[{n} earlier messages omitted for context window]"}

    Returns new list (does not mutate).
    """
    if len(messages) <= max_messages:
        return list(messages)

    # Collect system messages (all of them, by index)
    system_messages: list[dict] = [m for m in messages if m.get("role") == "system"]

    # Find first user message (non-system)
    first_user: dict | None = None
    first_user_idx: int = -1
    for i, m in enumerate(messages):
        if m.get("role") != "system":
            first_user = m
            first_user_idx = i
            break

    # Last (max_messages // 2) messages
    tail_count = max_messages // 2
    recent = list(messages[-tail_count:])

    # Determine what indices are "kept" to figure out how many are omitted
    kept_indices: set[int] = set()
    # All system message indices
    for i, m in enumerate(messages):
        if m.get("role") == "system":
            kept_indices.add(i)
    # First user message
    if first_user_idx >= 0:
        kept_indices.add(first_user_idx)
    # Recent (tail) messages
    tail_start = len(messages) - tail_count
    for i in range(tail_start, len(messages)):
        kept_indices.add(i)

    omitted_count = len(messages) - len(kept_indices)

    # Build the result list: system messages + first user + omission marker + recent
    result: list[dict] = []
    result.extend(system_messages)
    if first_user is not None:
        # Only add first_user if it's not already in recent (avoid duplication)
        if first_user not in recent:
            result.append(first_user)

    if omitted_count > 0:
        result.append(
            {
                "role": "assistant",
                "content": f"[{omitted_count} earlier messages omitted for context window]",
            }
        )

    result.extend(recent)
    return result


def apply_transforms(body: dict, max_messages: int = 20) -> dict:
    """Apply all transforms listed in body["transforms"].

    Supported transforms: "middle-out"
    Unknown transforms are silently ignored.
    Returns modified copy of body.
    """
    transforms = extract_transforms(body)
    if not transforms:
        return dict(body)

    result = dict(body)

    for transform in transforms:
        if transform == "middle-out":
            messages = result.get("messages", [])
            if isinstance(messages, list):
                result = dict(result)
                result["messages"] = apply_middle_out(messages, max_messages=max_messages)
        # Unknown transforms are silently ignored (by design per spec)

    return result
