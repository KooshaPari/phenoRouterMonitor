"""WP-33001: Universal External Proxy (Donut Bridge).
Provides a generic wrapper to intercept and control I/O for black-box agents.
Supports stdio, HTTP, and LSP interception to enforce thegent's policies externally.
"""

import asyncio
import logging
from asyncio import subprocess
from collections.abc import Callable

_log = logging.getLogger(__name__)


class BlackBoxProxy:
    """Universal proxy for external agents."""

    def __init__(self, agent_cmd: list[str], policy_enforcer: Callable | None = None) -> None:
        self.agent_cmd = agent_cmd
        self.policy_enforcer = policy_enforcer
        self.process: subprocess.Process | None = None

    async def start(self):
        """Launch the black-box agent as a subprocess with intercepted I/O."""
        _log.info("Launching black-box agent: %s", " ".join(self.agent_cmd))
        self.process = await asyncio.create_subprocess_exec(
            *self.agent_cmd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )

    async def send_input(self, input_data: str) -> str:
        """Intercept input, apply policy, and forward to the agent."""
        if not self.process or not self.process.stdin:
            raise RuntimeError("Agent process not started.")

        # 1. External Policy Check before sending input
        if self.policy_enforcer:
            allowed, reason = self.policy_enforcer(input_data, "input")
            if not allowed:
                _log.warning("Input blocked by proxy policy: %s", reason)
                return f"ERROR: Input blocked by governance proxy: {reason}"

        # 2. Forward to agent
        self.process.stdin.write(input_data.encode())
        await self.process.stdin.drain()

        # 3. Read output (simplified for mock)
        # In a real impl, this would handle streams or protocol-specific framing (LSP/JSON-RPC)
        output = await self.process.stdout.read(4096)
        output_str = output.decode()

        # 4. External Policy Check on output
        if self.policy_enforcer:
            allowed, reason = self.policy_enforcer(output_str, "output")
            if not allowed:
                _log.warning("Output blocked by proxy policy: %s", reason)
                return f"ERROR: Output blocked by governance proxy: {reason}"

        return output_str

    async def stop(self):
        """Terminate the black-box agent."""
        if self.process:
            self.process.terminate()
            await self.process.wait()
            _log.info("Black-box agent terminated.")
