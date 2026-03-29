"""Integration with thegent codex/cc/droid harness."""

import re
import time
from collections.abc import Callable
from pathlib import Path
from typing import Any

from thegent.agents.crew.executor import ExecutionResult
from thegent.agents.direct_agents import DirectAgentRunner


def _parse_tokens_from_output(output: str) -> tuple[int, int]:
    """Parse token counts from agent output.

    Looks for common patterns like:
    - "Tokens: 1234 input, 567 output"
    - "Usage: 1234 prompt + 567 completion = 1801 total"
    - "input_tokens=1234, output_tokens=567"
    - "Prompt tokens: 1234, Completion tokens: 567"

    Args:
        output: Agent stdout/stderr text

    Returns:
        Tuple of (prompt_tokens, completion_tokens)
    """
    prompt_tokens = 0
    completion_tokens = 0

    # Pattern 1: "Tokens: X input, Y output" or "X prompt tokens, Y completion"
    match = re.search(
        r"(?:tokens?|usage)[\s:]*?(\d+)[\s,]*(?:input|prompt)[^\d]*(\d+)[\s,]*(?:output|completion)",
        output,
        re.IGNORECASE,
    )
    if match:
        prompt_tokens = int(match.group(1))
        completion_tokens = int(match.group(2))
        return prompt_tokens, completion_tokens

    # Pattern 2: "input_tokens=X, output_tokens=Y" or "prompt_tokens=X, completion_tokens=Y"
    match = re.search(
        r"(?:input|prompt)_tokens?=(\d+).*?(?:output|completion)_tokens?=(\d+)",
        output,
        re.IGNORECASE,
    )
    if match:
        prompt_tokens = int(match.group(1))
        completion_tokens = int(match.group(2))
        return prompt_tokens, completion_tokens

    # Pattern 3: "Prompt tokens: X, Completion tokens: Y"
    match = re.search(
        r"prompt[\s_-]?tokens?[\s:]*(\d+).*?completion[\s_-]?tokens?[\s:]*(\d+)",
        output,
        re.IGNORECASE,
    )
    if match:
        prompt_tokens = int(match.group(1))
        completion_tokens = int(match.group(2))
        return prompt_tokens, completion_tokens

    # Pattern 4: "X + Y = Z total" usage format
    match = re.search(r"(\d+)\s*\+\s*(\d+)\s*=\s*\d+\s*total", output, re.IGNORECASE)
    if match:
        prompt_tokens = int(match.group(1))
        completion_tokens = int(match.group(2))
        return prompt_tokens, completion_tokens

    return prompt_tokens, completion_tokens


def _parse_cost_from_output(output: str) -> float:
    """Parse cost information from agent output.

    Looks for patterns like:
    - "Cost: $0.0123"
    - "Total cost: $0.05"
    - "cost_usd=0.0123"

    Args:
        output: Agent stdout/stderr text

    Returns:
        Cost in USD
    """
    # Pattern 1: "cost_usd=X.XX" (with equals sign)
    match = re.search(r"cost_usd=(\d+\.?\d*)", output, re.IGNORECASE)
    if match:
        return float(match.group(1))

    # Pattern 2: "Cost: $X.XX" or "cost: $X.XX" or "Total cost: $X.XX"
    match = re.search(r"(?:total\s+)?cost[\s:]*(?:\$)?(\d+\.?\d*)", output, re.IGNORECASE)
    if match:
        return float(match.group(1))

    return 0.0


def create_agent_executor(
    cwd: Path | None = None,
    mode: str = "write",
    timeout: int = 300,
    model: str | None = None,
) -> Callable[[str, str, dict[str, Any]], ExecutionResult]:
    """
    Create agent_executor callback that uses thegent's codex/cc/droid harness.

    Args:
        cwd: Working directory for agent execution
        mode: Execution mode (read-only, write, full)
        timeout: Timeout in seconds
        model: Optional model override

    Returns:
        Callable (agent_id, prompt, context) -> ExecutionResult
    """

    def agent_executor(agent_id: str, prompt: str, context: dict[str, Any]) -> ExecutionResult:
        """
        Execute agent via thegent harness.

        Args:
            agent_id: Agent identifier (e.g., "codex", "cursor-agent", "claude", "copilot", "gemini", "droid")
            prompt: Task prompt
            context: Execution context

        Returns:
            ExecutionResult
        """
        # Map agent_id to agent name
        agent_name_map: dict[str, str] = {
            "codex": "codex",
            "cursor": "cursor-agent",
            "cursor-agent": "cursor-agent",
            "claude": "claude",
            "copilot": "copilot",
            "gemini": "gemini",
            "droid": "opencode",  # droid uses opencode CLI
        }

        agent_name = agent_name_map.get(agent_id.lower(), agent_id.lower())

        # Use model from context or default
        use_model = context.get("model") or model

        # Track actual execution time
        start_time = time.time()

        try:
            # Create agent runner
            runner = DirectAgentRunner(agent_name)

            # Execute agent
            result = runner.run(
                prompt=prompt,
                cwd=cwd,
                mode=mode,
                timeout=timeout,
                use_stream=True,
                live_output=False,
                agent_model=use_model,
            )

            # Calculate actual duration
            duration_seconds = time.time() - start_time
            if result.timed_out:
                duration_seconds = timeout  # Cap at timeout for timed out executions

            # Convert RunResult to ExecutionResult
            success = result.exit_code == 0 and not result.timed_out

            # Extract tokens/cost from result if available
            # Combine stdout and stderr for parsing
            combined_output = f"{result.stdout}\n{result.stderr}"
            prompt_tokens, completion_tokens = _parse_tokens_from_output(combined_output)
            tokens_used = prompt_tokens + completion_tokens
            cost_usd = _parse_cost_from_output(combined_output)

            return ExecutionResult(
                task_id=context.get("task_id", ""),
                success=success,
                result=result.stdout if success else None,
                error=result.stderr if not success else None,
                duration_seconds=duration_seconds,
                tokens_used=tokens_used,
                cost_usd=cost_usd,
            )

        except Exception as e:
            duration_seconds = time.time() - start_time
            return ExecutionResult(
                task_id=context.get("task_id", ""),
                success=False,
                error=str(e),
                duration_seconds=duration_seconds,
                tokens_used=0,
                cost_usd=0.0,
            )

    return agent_executor
