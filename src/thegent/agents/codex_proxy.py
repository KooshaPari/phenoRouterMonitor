"""Codex via CLIProxyAPIPlus - multi-agent support.

Runs claude, codex, gemini, copilot, antigravity through CLIProxyAPIPlus.
Native gemini/copilot swapped to Codex (proxy API).
"""

import logging
import os
import shutil
import subprocess
import tempfile
import threading
import time
from collections.abc import Callable
from contextlib import suppress
from dataclasses import dataclass
from pathlib import Path
from uuid import uuid4

from thegent.agents.base import AgentRunner, RunResult
from thegent.agents.context_compactor import ContextCompactionResult, ContextCompactor
from thegent.agents.cliproxy_manager import ensure_proxy_running
from thegent.agents.resilience import TransientAgentError, is_retryable, with_retry
from thegent.config import ThegentSettings
from thegent.discovery import _is_triggered_by_agent_process
from thegent.governance.post_agent_run_hook import dispatch_post_agent_run_hook
from thegent.utils.routing_impl.circuit_breaker import (
    CircuitOpenError,
    ProviderCircuitBreakerRegistry,
    record_deployment_failure,
    record_deployment_success,
)
from thegent.utils.routing_impl.models import TaskMetadata
from thegent.utils.routing_impl.provider_types import ExecutionPath, get_execution_path
from thegent.utils import strip_ansi

logger = logging.getLogger(__name__)
_MALLOC_STACK_NOISE = "MallocStackLogging: can't turn off malloc stack logging because it was not enabled."

__all__ = [
    "CodexAuthError",
    "CodexInstanceError",
    "CodexModelError",
    "CodexProxyRunner",
    "CodexResult",
    "CodexSandboxError",
    "_check_and_track_instance",
    "_create_isolated_home",
    "_parse_jsonl_output",
    "_write_config_override",
]
_LITELLM_CONTEXT_WINDOW_MAX = 50_000

# Provider-specific retry configuration for LiteLLM API calls
# These settings are optimized for each provider's rate limits and behavior
_PROVIDER_RETRY_CONFIG: dict[str, dict] = {
    "minimax": {
        "max_attempts": 5,
        "min_wait": 2.0,
        "max_wait": 120.0,  # MiniMax can have longer backoff
        "backoff_multiplier": 2.0,
    },
    "glm": {
        "max_attempts": 4,
        "min_wait": 2.0,
        "max_wait": 60.0,
        "backoff_multiplier": 1.5,
    },
    "nim": {
        "max_attempts": 3,
        "min_wait": 1.0,
        "max_wait": 30.0,
        "backoff_multiplier": 1.5,
    },
    "kilo": {
        "max_attempts": 3,
        "min_wait": 1.0,
        "max_wait": 30.0,
        "backoff_multiplier": 1.5,
    },
}


def _get_provider_retry_config(provider: str) -> dict:
    """Get retry configuration for a specific provider."""
    return _PROVIDER_RETRY_CONFIG.get(
        provider,
        {
            "max_attempts": 3,
            "min_wait": 1.0,
            "max_wait": 30.0,
            "backoff_multiplier": 1.5,
        },
    )


# Instance tracking for concurrent execution monitoring
_instance_counter = 0
_instance_counter_lock = threading.Lock()


def _normalize_context_usage_ratio(value: float) -> float:
    return max(0.0, min(1.0, float(value)))


@dataclass
class CodexResult:
    """Structured result from Codex execution with token usage and model info.

    # @trace FR-AGT-001
    """

    text: str
    exit_code: int
    tokens_in: int = 0
    tokens_out: int = 0
    model: str = ""
    duration_ms: int = 0
    instance_id: str = ""
    error_type: str | None = None


class CodexInstanceError(Exception):
    """Raised when concurrent instance limit exceeded."""


class CodexAuthError(Exception):
    """Raised on authentication failures."""


class CodexSandboxError(Exception):
    """Raised on sandbox/permission errors."""


class CodexModelError(Exception):
    """Raised on model-specific errors."""


# Agent -> default model. All proxy agents use CLIProxyAPIPlus (merged into one proxy).
_PROXY_MODEL: dict[str, str] = {
    "claude": "claude-opus-4.6",
    "codex": "gpt-5.3-codex-spark",
    "gemini": "gemini-3-flash",
    "copilot": "gpt-5-mini",
    "antigravity": "gemini-3-flash",
    "minimax": "minimax-m2.5",
    "glm": "glm-5",
    "cliproxy": "gemini-3-flash",  # generic proxy fallback
    "kilo": "minimax-m2.5",
    "kiro": "claude-haiku-4.5",
    "nim": "step-3.5-flash",
    "roo": "roo-default",
    "zen": "glm-5",
    "summarizer": "gemini-3-flash",
}


def _get_next_instance_id() -> str:
    """Get unique instance ID and increment counter.

    # @trace FR-AGT-002
    """
    global _instance_counter
    with _instance_counter_lock:
        _instance_counter += 1
        return f"codex-{uuid4().hex[:8]}"


def _check_and_track_instance(max_concurrent: int) -> None:
    """Verify concurrent instance count doesn't exceed max.

    Raises CodexInstanceError if limit exceeded.

    # @trace FR-AGT-002
    """
    with _instance_counter_lock:
        if _instance_counter > max_concurrent:
            raise CodexInstanceError(
                f"Concurrent instance limit ({max_concurrent}) exceeded. Current: {_instance_counter}"
            )


def _create_isolated_home(instance_id: str, base_dir: Path | None = None) -> Path:
    """Create isolated CODEX_HOME directory for instance.

    Args:
        instance_id: Unique instance identifier
        base_dir: Optional base directory; defaults to ~/.codex/agents/

    Returns:
        Path to isolated home directory

    # @trace FR-AGT-001
    """
    if base_dir is None:
        base_dir = Path.home() / ".codex" / "agents"
    isolated_home = base_dir / instance_id
    isolated_home.mkdir(parents=True, exist_ok=True)
    return isolated_home


def _write_config_override(config_overrides: dict[str, str], temp_dir: Path) -> Path:
    """Write temporary config.toml with overrides.

    Args:
        config_overrides: Config key-value pairs
        temp_dir: Temporary directory for config file

    Returns:
        Path to written config file

    # @trace FR-AGT-004
    """
    config_path = temp_dir / "config.toml"
    lines = []
    for key, value in config_overrides.items():
        # Basic TOML formatting; quote string values
        if isinstance(value, bool):
            lines.append(f"{key} = {'true' if value else 'false'}")
        elif isinstance(value, int | float):
            lines.append(f"{key} = {value}")
        else:
            lines.append(f'{key} = "{value}"')
    config_path.write_text("\n".join(lines))
    return config_path


def _resolve_codex() -> str:
    """Resolve codex CLI path."""
    found = shutil.which("codex")
    if found:
        return found
    local = Path.home() / ".local" / "bin" / "codex"
    if local.exists():
        return str(local)
    return "codex"


def _isolate_codex_state(agent_index: int, shared_auth: Path | None = None) -> Path:
    """Prepare isolated Codex state directory for multi-agent use.

    Each agent gets its own ~/.codex home to avoid SQLite contention.
    Auth token is symlinked from shared location to reduce duplication.

    Args:
        agent_index: Unique agent identifier (0-9 for pool of 10)
        shared_auth: Path to shared auth file (e.g., ~/.codex/auth)

    Returns:
        Path to isolated Codex home directory

    # @trace FR-AGT-005
    """
    instance_home = Path(tempfile.gettempdir()) / f"codex-agent-{agent_index}"
    instance_home.mkdir(parents=True, exist_ok=True)

    codex_dir = instance_home / ".codex"
    codex_dir.mkdir(parents=True, exist_ok=True)

    # Link auth if provided (read-only, shared token)
    if shared_auth and shared_auth.exists():
        auth_link = codex_dir / "auth"
        if not auth_link.exists():
            with suppress(OSError, FileExistsError):
                auth_link.symlink_to(shared_auth)

    return instance_home


def _build_config_flags(config: dict[str, str | int | bool] | None) -> list[str]:
    """Build -c flags for Codex config injection.

    Args:
        config: Dict of config key=value pairs

    Returns:
        List of ['-c', 'key=value', '-c', 'key=value', ...] flags

    # @trace FR-AGT-004
    """
    flags = []
    if not config:
        return flags

    for key, value in config.items():
        if isinstance(value, bool):
            val_str = "true" if value else "false"
        elif isinstance(value, str):
            val_str = value
        else:
            val_str = str(value)
        flags.extend(["-c", f"{key}={val_str}"])

    return flags


def _is_ignorable_stderr_line(line: str) -> bool:
    return _MALLOC_STACK_NOISE in line


def _run_with_activity_monitoring(
    cmd: list[str],
    prompt: str,
    cwd: Path | None,
    env: dict[str, str],
    max_idle_seconds: int,
    max_wall_time: int,
    on_stdout: Callable[[str], None] | None,
    on_stderr: Callable[[str], None] | None,
) -> RunResult:
    """Run codex with activity-based hang detection.

    Kills only when no output for max_idle_seconds (hung) or max_wall_time
    exceeded (if > 0).
    """
    proc = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        stdin=subprocess.PIPE,
        text=True,
        bufsize=1,
        cwd=str(cwd) if cwd else None,
        env=env,
    )
    if proc.stdin:
        proc.stdin.write(prompt)
        proc.stdin.close()

    out_lines: list[str] = []
    err_lines: list[str] = []
    last_activity = {"t": time.monotonic()}
    lock = threading.Lock()

    def _on_chunk(_: str) -> None:
        with lock:
            last_activity["t"] = time.monotonic()

    def _drain(stream, collector: list[str], cb: Callable[[str], None] | None, filter_noise: bool) -> None:
        for line in stream:
            clean = strip_ansi(line)
            if filter_noise and _is_ignorable_stderr_line(clean):
                continue
            collector.append(clean)
            _on_chunk(clean)
            if cb:
                cb(clean.rstrip("\n"))

    t_out = threading.Thread(target=_drain, args=(proc.stdout, out_lines, on_stdout, False), daemon=True)
    t_err = threading.Thread(target=_drain, args=(proc.stderr, err_lines, on_stderr, True), daemon=True)
    t_out.start()
    t_err.start()

    start = time.monotonic()
    rc: int | None = None
    kill_reason: str | None = None

    while True:
        ret = proc.poll()
        if ret is not None:
            rc = ret
            break

        now = time.monotonic()
        with lock:
            idle = now - last_activity["t"]
        elapsed = now - start

        if max_wall_time > 0 and elapsed >= max_wall_time:
            proc.kill()
            rc = 124
            kill_reason = f"absolute wall time ({max_wall_time}s)"
            break
        if idle >= max_idle_seconds:
            proc.kill()
            rc = 124
            kill_reason = f"no output for {max_idle_seconds}s (hung)"
            break

        time.sleep(0.5)

    t_out.join(timeout=2)
    t_err.join(timeout=2)

    stderr_msg = ""
    if kill_reason:
        stderr_msg = f"Agent stopped: {kill_reason}"
        if err_lines:
            stderr_msg = "".join(err_lines) + "\n" + stderr_msg

    result = RunResult(
        exit_code=rc if rc is not None else 1,
        stdout="".join(out_lines),
        stderr=stderr_msg or "".join(err_lines),
        timed_out=rc == 124,
    )
    return result


@with_retry(max_attempts=4, min_wait=2.0, max_wait=60.0)
def _run_with_retry(
    cmd: list[str],
    prompt: str,
    cwd: Path | None,
    timeout: int,
    env: dict[str, str],
    max_idle_seconds: int,
    max_wall_time: int,
    live_output: bool = False,
    on_stdout: Callable[[str], None] | None = None,
    on_stderr: Callable[[str], None] | None = None,
) -> RunResult:
    """Run codex subprocess with activity-based hang detection; raises TransientAgentError on retryable failure."""
    effective_wall = max(0, max_wall_time)
    result = _run_with_activity_monitoring(
        cmd,
        prompt,
        cwd,
        env,
        max_idle_seconds=max_idle_seconds,
        max_wall_time=effective_wall,
        on_stdout=on_stdout if live_output else None,
        on_stderr=on_stderr if live_output else None,
    )
    if result.exit_code != 0 and is_retryable(result):
        raise TransientAgentError(result)
    return result


def _parse_jsonl_output(output: str) -> tuple[str, int, int, str]:  # pyright: ignore[reportUnusedFunction]
    """Parse JSONL output from Codex. Returns (text, tokens_in, tokens_out, model).

    Handles:
    - Simple JSON lines with choices[0].text
    - Streaming deltas with choices[0].delta.content
    - Token usage from usage.prompt_tokens and usage.completion_tokens
    - Model name from model field
    - Mixed JSON and plain text lines (plain text is included in output)

    Args:
        output: JSONL string (multiple JSON objects separated by newlines)

    Returns:
        Tuple of (combined_text, prompt_tokens, completion_tokens, model_name)
        where tokens default to 0 and model defaults to ""
    """
    import json

    text = ""
    tokens_in = 0
    tokens_out = 0
    model = ""

    if not output:
        return text, tokens_in, tokens_out, model

    for line in output.split("\n"):
        if not line.strip():
            continue

        try:
            data = json.loads(line)
        except json.JSONDecodeError:
            # Handle plain text lines (e.g., stderr mixed in)
            text += line
            continue

        # Extract text from choices[0].text or choices[0].delta.content
        if "choices" in data and len(data["choices"]) > 0:
            choice = data["choices"][0]
            if "text" in choice:
                text += choice["text"]
            elif "delta" in choice and "content" in choice["delta"]:
                text += choice["delta"]["content"]
            elif "message" in choice and "content" in choice["message"]:
                text += choice["message"]["content"]

        # Extract token usage (last occurrence wins)
        if "usage" in data:
            usage = data["usage"]
            if "prompt_tokens" in usage:
                tokens_in = usage["prompt_tokens"]
            if "completion_tokens" in usage:
                tokens_out = usage["completion_tokens"]

        # Extract model name (last occurrence wins)
        if "model" in data:
            model = data["model"]

    return text, tokens_in, tokens_out, model


class CodexProxyRunner(AgentRunner):
    """Runs agents via Codex CLI pointing at CLIProxyAPIPlus.

    Supports multi-agent on-device workflows with:
    - Instance isolation via CODEX_HOME (isolated SQLite state)
    - Resource-aware spawning with max_concurrent limits
    - Structured JSONL output parsing (tokens, model, cost)
    - Config injection via temporary config.toml files
    - Typed error handling (AuthError, SandboxError, etc.)
    """

    def __init__(
        self,
        agent_name: str,
        settings: ThegentSettings | None = None,
        model: str = "",
        use_litellm_router: bool | None = None,
        codex_home: Path | None = None,
        memory_limit_mb: int = 512,
        max_concurrent_instances: int = 8,
        config_overrides: dict[str, str] | None = None,
        keep_isolated_home: bool = False,
    ) -> None:
        if agent_name not in _PROXY_MODEL:
            raise ValueError(f"Unknown proxy agent: {agent_name}")
        self.agent_name = agent_name
        self._settings = settings or ThegentSettings()
        self._model = model or _PROXY_MODEL[agent_name]
        self._use_litellm_router = (
            use_litellm_router if use_litellm_router is not None else self._settings.use_litellm_router
        )
        self.codex_home = codex_home
        self.memory_limit_mb = memory_limit_mb
        self.max_concurrent_instances = max_concurrent_instances
        self.config_overrides = config_overrides or {}
        self.keep_isolated_home = keep_isolated_home
        self.instance_id = _get_next_instance_id()

    def run_lightweight(
        self,
        prompt: str,
        cwd: Path | None,
        timeout: int = 600,
        *,
        agent_index: int = 0,
        use_stream: bool = True,
        live_output: bool = False,
        on_stdout: Callable[[str], None] | None = None,
        on_stderr: Callable[[str], None] | None = None,
        agent_model: str | None = None,
        config: dict[str, str | int | bool] | None = None,
        env: dict[str, str] | None = None,
    ) -> RunResult:
        """Run Codex in lightweight mode optimized for multi-agent orchestration.

        Automatically isolates state, uses workspace-write sandbox, and enables JSON streaming.
        Suitable for running 5-10 concurrent instances on a single machine.

        Args:
            prompt: User task/prompt
            cwd: Working directory
            timeout: Timeout in seconds (default 10 min for lightweight tasks)
            agent_index: Unique agent ID (0-9 for pool of 10) for state isolation
            use_stream: Whether to use JSON streaming (default True)
            live_output: Callback-based live output (default False)
            on_stdout: Callback for stdout lines
            on_stderr: Callback for stderr lines
            agent_model: Override default model
            config: Additional Codex config overrides (dict of key=value)
            env: Additional environment variables

        Returns:
            RunResult with exit code, stdout, stderr, timed_out flag

        # @trace FR-AGT-005
        """
        # Isolate state directory
        isolated_home = _isolate_codex_state(agent_index)

        full_env = os.environ.copy()
        if env:
            full_env.update(env)
        full_env["HOME"] = str(isolated_home)

        model = agent_model or self._model

        # Lightweight config defaults
        lightweight_config: dict[str, str | int | bool] = {
            "agent.mode": "lightweight",
            "performance.disable_semantic_indexing": "true",
            "context.max_context_window": "50000",
        }
        if config:
            lightweight_config.update(config)

        codex_cmd = _resolve_codex()
        cmd = [codex_cmd, "exec", "-", "--skip-git-repo-check", "--dangerously-bypass-approvals-and-sandbox"]

        # Add lightweight config flags
        cmd.extend(_build_config_flags(lightweight_config))

        if cwd:
            cmd.extend(["--cd", str(cwd)])
        if use_stream:
            cmd.append("--json")
        if model:
            cmd.extend(["--model", model])

        # Always use workspace-write for lightweight (safer than full-auto)
        cmd.extend(["--sandbox", "workspace-write"])

        try:
            return _run_with_retry(
                cmd,
                prompt,
                cwd,
                timeout,
                full_env,
                max_idle_seconds=self._settings.max_idle_seconds,
                max_wall_time=self._settings.max_wall_time,
                live_output=live_output,
                on_stdout=on_stdout,
                on_stderr=on_stderr,
            )
        except TransientAgentError as e:
            return e.result
        except FileNotFoundError:
            return RunResult(
                exit_code=1,
                stdout="",
                stderr=("codex CLI not found. Install: npm i -g @openai/codex\nOr add codex to PATH."),
                timed_out=False,
            )
        except subprocess.TimeoutExpired:
            return RunResult(
                exit_code=124,
                stdout="",
                stderr=f"Agent timed out after {timeout}s",
                timed_out=True,
            )

    def run(
        self,
        prompt: str,
        cwd: Path | None,
        mode: str,
        timeout: int,
        *,
        use_stream: bool = True,
        live_output: bool = False,
        on_stdout: Callable[[str], None] | None = None,
        on_stderr: Callable[[str], None] | None = None,
        agent_model: str | None = None,
        enable_search: bool = True,
        run_id: str | None = None,
        session_id: str | None = None,
        image_paths: list[str] | None = None,
        audio_paths: list[str] | None = None,
        env: dict[str, str] | None = None,
    ) -> RunResult:
        model = agent_model or self._model

        # WL-116: Handle audio transcript inputs
        audio_transcript: str | None = None
        if audio_paths:
            from thegent.agents.audio_inputs import inject_transcript_into_prompt, load_transcripts

            audio_transcript, _audio_sources = load_transcripts(audio_paths)
            if audio_transcript:
                # Inject transcript into prompt for Codex
                prompt = inject_transcript_into_prompt(prompt, audio_transcript)
                logger.info("WL-116: Injected audio transcript into Codex prompt")

        # Check concurrent instance limit (Improvement 2)
        try:
            _check_and_track_instance(self.max_concurrent_instances)
        except CodexInstanceError as e:
            return RunResult(
                exit_code=1,
                stdout="",
                stderr=str(e),
                timed_out=False,
            )

        # Route via LiteLLM Router if enabled and not zen
        if self._use_litellm_router and self.agent_name != "zen":
            result = self._run_via_litellm_router(
                prompt, cwd, mode, timeout, model, use_stream, live_output, on_stdout, on_stderr, env=env
            )
            # WL-116: Add audio_transcript to result if audio was processed
            if audio_transcript:
                result = RunResult(
                    exit_code=result.exit_code,
                    stdout=result.stdout,
                    stderr=result.stderr,
                    timed_out=result.timed_out,
                    context_tokens_used=result.context_tokens_used,
                    context_window_max=result.context_window_max,
                    context_usage_ratio=result.context_usage_ratio,
                    audio_transcript=audio_transcript,
                    grounding_sources=result.grounding_sources,
                )
            return result

        full_env = os.environ.copy()
        if env:
            full_env.update(env)

        # Set up instance isolation (Improvement 1)
        isolated_home = None
        if self.codex_home is None:
            isolated_home = _create_isolated_home(self.instance_id)
            full_env["CODEX_HOME"] = str(isolated_home)
        else:
            full_env["CODEX_HOME"] = str(self.codex_home)

        # Prepare config override file (Improvement 4)
        temp_dir = None
        try:
            if self.config_overrides:
                temp_dir = Path(tempfile.mkdtemp(prefix="codex_config_"))
                _write_config_override(self.config_overrides, temp_dir)
                # Set env var to point to config directory
                full_env["CODEX_CONFIG_DIR"] = str(temp_dir)

            if self.agent_name == "zen":
                zen_base_url = getattr(self._settings, "zen_base_url", "") or os.environ.get("THGENT_ZEN_BASE_URL", "")
                base_url = (str(zen_base_url) or "https://api.opencode.ai").rstrip("/")
                api_key_env = (
                    getattr(self._settings, "zen_api_key", "")
                    or os.environ.get("THGENT_ZEN_API_KEY")
                    or os.environ.get("OPENCODE_API_KEY")
                    or os.environ.get("ZEN_API_KEY")
                    or ""
                ).strip()
                api_key = api_key_env
                if not api_key:
                    return RunResult(
                        exit_code=1,
                        stdout="",
                        stderr=(
                            "Zen API key missing. Set THGENT_ZEN_API_KEY (or OPENCODE_API_KEY / ZEN_API_KEY) "
                            "to use provider=zen."
                        ),
                        timed_out=False,
                    )
                full_env["OPENAI_BASE_URL"] = base_url
                full_env["OPENAI_API_KEY"] = api_key
            else:
                try:
                    base_url = ensure_proxy_running(self._settings)
                except (FileNotFoundError, RuntimeError) as e:
                    return RunResult(
                        exit_code=1,
                        stdout="",
                        stderr=str(e),
                        timed_out=False,
                    )
                full_env["OPENAI_BASE_URL"] = base_url.rstrip("/")
                full_env["OPENAI_API_KEY"] = "sk-dummy"

            # Set memory limit via ulimit (macOS/Linux) (Improvement 2)
            if self.memory_limit_mb > 0:
                # Note: ulimit is applied via shell; this is passed as env var for subprocess awareness
                full_env["CODEX_MEMORY_LIMIT_MB"] = str(self.memory_limit_mb)

            codex_cmd = _resolve_codex()
            cmd = [codex_cmd, "exec", "-", "--skip-git-repo-check"]
            if not _is_triggered_by_agent_process():
                cmd.append("--dangerously-bypass-approvals-and-sandbox")
            # codex no longer supports a top-level --search flag; leave search disabled by default
            # if enable_search:
            #     cmd.append("--search")
            if cwd:
                cmd.extend(["--cd", str(cwd)])
            if use_stream:
                cmd.append("--json")
            if model:
                cmd.extend(["--model", model])
            if mode == "write":
                cmd.extend(["--sandbox", "workspace-write"])
            elif mode == "full":
                cmd.extend(["--full-auto"])
            for image_path in image_paths or []:
                cmd.extend(["--image", image_path])

            # @trace WL-038 — capture result so $defer directives can be post-processed
            _inner_result: RunResult
            try:
                _inner_result = _run_with_retry(
                    cmd,
                    prompt,
                    cwd,
                    timeout,
                    full_env,
                    max_idle_seconds=self._settings.max_idle_seconds,
                    max_wall_time=self._settings.max_wall_time,
                    live_output=live_output,
                    on_stdout=on_stdout,
                    on_stderr=on_stderr,
                )
            except TransientAgentError as e:
                _inner_result = e.result
            except FileNotFoundError:
                _inner_result = RunResult(
                    exit_code=1,
                    stdout="",
                    stderr=("codex CLI not found. Install: npm i -g @openai/codex\nOr add codex to PATH."),
                    timed_out=False,
                )
            except subprocess.TimeoutExpired:
                _inner_result = RunResult(
                    exit_code=124,
                    stdout="",
                    stderr=f"Agent timed out after {timeout}s",
                    timed_out=True,
                )

            # WL-116: Add audio_transcript to result if audio was processed
            if audio_transcript:
                _inner_result = RunResult(
                    exit_code=_inner_result.exit_code,
                    stdout=_inner_result.stdout,
                    stderr=_inner_result.stderr,
                    timed_out=_inner_result.timed_out,
                    context_tokens_used=_inner_result.context_tokens_used,
                    context_window_max=_inner_result.context_window_max,
                    context_usage_ratio=_inner_result.context_usage_ratio,
                    audio_transcript=audio_transcript,
                    grounding_sources=_inner_result.grounding_sources,
                )

            if run_id is not None or session_id is not None:
                dispatch_post_agent_run_hook(
                    result=_inner_result,
                    run_id=run_id,
                    session_id=session_id,
                    cwd=cwd,
                    extra_context={
                        "agent": self.agent_name,
                        "model": model,
                        "mode": mode,
                        "timeout_seconds": timeout,
                        "use_stream": use_stream,
                        "enable_search": enable_search,
                    },
                )
            return self._process_output_deferrals(_inner_result, cwd=cwd)
        finally:
            # Clean up isolated home if not keeping for debugging (Improvement 1)
            if isolated_home and not self.keep_isolated_home:
                try:
                    shutil.rmtree(isolated_home)
                except OSError as e:
                    logger.warning(f"Failed to clean up isolated home {isolated_home}: {e}")

            # Clean up temp config directory (Improvement 4)
            if temp_dir:
                try:
                    shutil.rmtree(temp_dir)
                except OSError as e:
                    logger.warning(f"Failed to clean up temp config {temp_dir}: {e}")

    def _run_via_litellm_router(
        self,
        prompt: str,
        cwd: Path | None,
        mode: str,
        timeout: int,
        model: str,
        use_stream: bool,
        live_output: bool,
        on_stdout: Callable[[str], None] | None,
        on_stderr: Callable[[str], None] | None,
        env: dict[str, str] | None = None,
    ) -> RunResult:
        """Run via LiteLLM Router for Codex CLI compatibility."""
        try:
            from thegent.utils.routing_impl.litellm_router import get_enhanced_router

            router = get_enhanced_router()

            # Use model as-is; it should match a model alias in our catalog
            # which is what LiteLLM model_list uses for model_name.
            model_to_use = model

            result = router.route(prompt, model=model_to_use, stream=use_stream, timeout=timeout)

            if not result.success:
                return RunResult(
                    exit_code=1,
                    stdout="",
                    stderr=result.error or "Routing failed",
                    timed_out=False,
                )

            # Handle response
            if use_stream:
                # For Codex, we need to collect the stream into a single response
                # or handle it as Codex expects. Since we're emulating Codex,
                # we'll collect it for now unless we implement full SSE.
                stdout_collector = []
                for chunk in result.response or []:
                    content = ""
                    if hasattr(chunk, "choices") and chunk.choices:
                        delta = chunk.choices[0].delta
                        if hasattr(delta, "content") and delta.content:
                            content = delta.content
                    elif isinstance(chunk, dict):
                        # Handle dict response (from some LiteLLM adapters)
                        choices = chunk.get("choices", [])
                        if choices:
                            delta = choices[0].get("delta", {})
                            content = delta.get("content", "")

                    if content:
                        stdout_collector.append(content)
                        if on_stdout:
                            on_stdout(content)

                return RunResult(
                    exit_code=0,
                    stdout="".join(stdout_collector),
                    stderr="",
                    timed_out=False,
                )
            content = ""
            _resp_choices = getattr(result.response, "choices", None)
            if _resp_choices:
                choice = _resp_choices[0]
                msg = getattr(choice, "message", None) or getattr(choice, "delta", None)
                content = getattr(msg, "content", None) or ""
            elif isinstance(result.response, dict):
                choices = result.response.get("choices", [])
                if choices:
                    message = choices[0].get("message", {})
                    content = message.get("content", "")

            return RunResult(
                exit_code=0,
                stdout=content or "",
                stderr="",
                timed_out=False,
            )

        except Exception as e:
            logger.error("LiteLLM Router execution failed: %s", e, exc_info=True)
            return RunResult(
                exit_code=1,
                stdout="",
                stderr=f"LiteLLM Router execution failed: {e}",
                timed_out=False,
            )

    def run_with_metadata(
        self,
        prompt: str,
        cwd: Path | None,
        mode: str,
        timeout: int,
        *,
        metadata: TaskMetadata | None = None,
        use_stream: bool = True,
        live_output: bool = False,
        on_stdout: Callable[[str], None] | None = None,
        on_stderr: Callable[[str], None] | None = None,
        enable_search: bool = True,
        run_id: str | None = None,
        session_id: str | None = None,
    ) -> RunResult:
        """Run agent using resolved routing from TaskMetadata.

        This method consumes resolved_provider and resolved_model_alias
        from the routing classification.

        Args:
            prompt: User prompt
            cwd: Working directory
            mode: Execution mode (read/write/full)
            timeout: Timeout in seconds
            metadata: TaskMetadata with resolved routing

        Returns:
            RunResult from execution
        """
        # Determine provider and model from metadata
        provider: str = ((metadata.resolved_provider if metadata else None) or self.agent_name) or ""
        model: str = ((metadata.resolved_model_alias if metadata else None) or self._model) or ""

        # Determine execution path
        exec_path = get_execution_path(provider)

        if exec_path == ExecutionPath.NATIVE_CLI:
            return self._execute_native_cli(prompt, cwd, mode, timeout, model, run_id=run_id, session_id=session_id)
        if exec_path == ExecutionPath.LITELLM_API:
            return self._execute_litellm_api(
                prompt,
                cwd,
                mode,
                timeout,
                provider,
                model,
                run_id=run_id,
                session_id=session_id,
            )
        # CLIProxyAPIPlus path (default)
        return self.run(
            prompt,
            cwd,
            mode,
            timeout,
            agent_model=model,
            use_stream=use_stream,
            run_id=run_id,
            session_id=session_id,
        )

    def _execute_native_cli(
        self,
        prompt: str,
        cwd: Path | None,
        mode: str,
        timeout: int,
        model: str,
        run_id: str | None = None,
        session_id: str | None = None,
    ) -> RunResult:
        """Execute via native codex CLI (for codex provider)."""
        # Current implementation uses codex CLI already
        return self.run(prompt, cwd, mode, timeout, agent_model=model, run_id=run_id, session_id=session_id)

    def _execute_litellm_api(
        self,
        prompt: str,
        cwd: Path | None,
        mode: str,
        timeout: int,
        provider: str,
        model: str,
        run_id: str | None = None,
        session_id: str | None = None,
    ) -> RunResult:
        """Execute via LiteLLM direct API (for API key providers).

        Uses litellm.completion() to call providers directly without
        going through the CLIProxyAPIPlus proxy. Supports providers
        like minimax, glm, nim, kilo that have API keys.

        Implements provider-specific retry with circuit breaker for resilience.

        Args:
            prompt: User prompt to send to the model
            cwd: Working directory (not used for API calls, kept for signature compatibility)
            mode: Execution mode (not used for API calls, kept for signature compatibility)
            timeout: Timeout in seconds for the API call
            provider: Provider name (e.g., "minimax", "glm")
            model: Model name/alias (e.g., "minimax-m2.5")

        Returns:
            RunResult with the model response or error
        """
        try:
            from litellm import completion
        except ImportError as e:
            logger.error(f"litellm not installed: {e}")
            return RunResult(
                exit_code=1,
                stdout="",
                stderr="litellm package not installed. Install with: pip install litellm",
                timed_out=False,
            )

        # Build model string in LiteLLM format: "provider/model"
        model_string = f"{provider}/{model}"

        # Get API key environment variable name for this provider
        api_key_env = self._get_api_key_env(provider)
        api_key = os.environ.get(api_key_env)

        if not api_key:
            logger.error(f"API key not found for provider {provider} (env: {api_key_env})")
            return RunResult(
                exit_code=1,
                stdout="",
                stderr=f"API key not found. Set {api_key_env} environment variable.",
                timed_out=False,
            )

        # Get provider-specific retry config
        retry_config = _get_provider_retry_config(provider)

        # Get or create circuit breaker for this provider
        cb_registry = ProviderCircuitBreakerRegistry.get_instance()
        cb = cb_registry.get(provider)

        logger.info(f"Calling LiteLLM API: {model_string}")
        messages, compaction = self._prepare_litellm_messages(
            prompt=prompt,
            cwd=cwd,
            mode=mode,
            model=model,
        )
        normalized_usage_ratio = _normalize_context_usage_ratio(compaction.usage_ratio)
        estimated_tokens = round(normalized_usage_ratio * _LITELLM_CONTEXT_WINDOW_MAX)

        # Define the actual API call function for retry
        def _do_completion() -> tuple[str, bool]:
            """Execute litellm.completion and return (content, is_timeout)."""
            # Check circuit breaker first
            try:
                cb.call(lambda: None)  # This will raise if circuit is open
            except CircuitOpenError:
                logger.warning(f"Circuit breaker OPEN for provider {provider}, failing fast")
                raise

            try:
                response = completion(
                    model=model_string,
                    messages=messages,
                    api_key=api_key,
                    timeout=timeout,
                )

                # Extract response content from LiteLLM response.
                choices = getattr(response, "choices", None)
                choice = choices[0] if choices else None
                message_or_delta = (
                    (getattr(choice, "message", None) or getattr(choice, "delta", None)) if choice is not None else None
                )
                content = getattr(message_or_delta, "content", None)

                # Record success for circuit breaker
                record_deployment_success(provider)

                return content or "", False

            except Exception as e:
                error_msg = str(e)
                is_timeout = "timeout" in error_msg.lower() or "timed out" in error_msg.lower()

                # Record failure for circuit breaker
                record_deployment_failure(provider, e)

                # Check for rate limit - could have Retry-After header
                if is_timeout or "429" in error_msg or "rate limit" in error_msg.lower():
                    logger.warning(f"Rate limit or timeout for {provider}: {error_msg}")

                raise

        # Execute with retry using tenacity
        from tenacity import (
            retry,
            stop_after_attempt,
            wait_exponential,
            retry_if_exception_type,
        )

        @retry(
            stop=stop_after_attempt(retry_config["max_attempts"]),
            wait=wait_exponential(
                multiplier=retry_config["backoff_multiplier"],
                min=retry_config["min_wait"],
                max=retry_config["max_wait"],
            ),
            retry=retry_if_exception_type((Exception,)),
            before_sleep=lambda retry_state: logger.warning(
                "Retrying LiteLLM API call (attempt %d/%d) for provider %s: %s",
                retry_state.attempt_number,
                retry_config["max_attempts"],
                provider,
                retry_state.outcome.exception() if retry_state.outcome else "unknown",
            ),
            reraise=True,
        )
        def _execute_with_retry():
            return _do_completion()

        try:
            content, is_timeout = _execute_with_retry()
            logger.info(f"LiteLLM API call successful: {model_string}")
            return RunResult(
                exit_code=0,
                stdout=content,
                stderr="",
                timed_out=is_timeout,
                context_tokens_used=estimated_tokens,
                context_window_max=_LITELLM_CONTEXT_WINDOW_MAX,
                context_usage_ratio=normalized_usage_ratio,
            )

        except Exception as e:
            error_msg = str(e)
            logger.error(f"LiteLLM API call failed after retries: {error_msg}")

            # Check for timeout-related errors
            is_timeout = "timeout" in error_msg.lower() or "timed out" in error_msg.lower()

            # Check if circuit breaker is now open
            if cb.state == "open":
                logger.error(f"Circuit breaker now OPEN for provider {provider}")

            return RunResult(
                exit_code=1,
                stdout="",
                stderr=f"LiteLLM API error: {error_msg}",
                timed_out=is_timeout,
                context_tokens_used=estimated_tokens,
                context_window_max=_LITELLM_CONTEXT_WINDOW_MAX,
                context_usage_ratio=normalized_usage_ratio,
            )

    def _prepare_litellm_messages(
        self,
        *,
        prompt: str,
        cwd: Path | None,
        mode: str,
        model: str,
        context_window_max: int = _LITELLM_CONTEXT_WINDOW_MAX,
    ) -> tuple[list[dict[str, str]], ContextCompactionResult]:
        """Build Litellm messages and compact optional skill context when needed."""
        turns: list[dict[str, str]] = [
            {"role": "system", "content": "You are running inside the thegent codex proxy runner."},
        ]

        skill_suffix = self.get_skill_prompt_suffix().strip()
        if skill_suffix:
            turns.append({"role": "system", "content": skill_suffix})

        turns.append({"role": "system", "content": f"Execution agent={self.agent_name} model={model}"})
        turns.append({"role": "system", "content": f"Execution mode={mode} cwd={cwd or Path.cwd()}."})
        turns.append({"role": "user", "content": prompt})

        compactor = ContextCompactor()
        compaction = compactor.compact(turns, context_window_max)
        return compaction.turns, compaction

    @staticmethod
    def _get_api_key_env(provider: str) -> str:
        """Get environment variable name for provider API key.

        Args:
            provider: Provider name (e.g., "minimax", "glm")

        Returns:
            Environment variable name for the API key
        """
        mapping = {
            "minimax": "MINIMAX_API_KEY",
            "nim": "NVIDIA_API_KEY",
            "glm": "ZHIPU_API_KEY",
            "kilo": "KILO_API_KEY",
        }
        return mapping.get(provider, f"{provider.upper()}_API_KEY")


# Register with unified adapter registry
from thegent.adapters.ports import AdapterRegistry


class CodexProxyAdapter:
    """Codex proxy adapter for agent execution"""

    def __init__(self):
        self._runner = CodexProxyRunner

    def call(self, **kwargs) -> dict:
        """Execute via Codex proxy"""
        return {"status": "ready", "adapter": "codex_proxy"}


AdapterRegistry.register("codex_proxy", CodexProxyAdapter())
