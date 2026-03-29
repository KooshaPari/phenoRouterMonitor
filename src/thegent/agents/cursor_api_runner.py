"""Cursor via cursor-api (wisdgod) - OpenAI-compatible HTTP backend."""

import hashlib
import os
import shutil
import subprocess
from thegent.infra.shim_subprocess import run as shim_run
from collections.abc import Callable
from pathlib import Path

from cachetools import TTLCache

from thegent.agents.base import AgentRunner, RunResult
from thegent.agents.resilience import TransientAgentError, is_retryable, with_retry
from thegent.config import ThegentSettings
from thegent.governance.post_agent_run_hook import dispatch_post_agent_run_hook
from thegent.infra.power import wrap_with_caffeinate
from thegent.utils import strip_ansi

_PROXY_MODEL = "claude-4.5-opus-high-thinking"
_MALLOC_STACK_NOISE = "MallocStackLogging: can't turn off malloc stack logging because it was not enabled."

# TTLCache for reachability result: keyed by (base_url, token_hash), TTL 30 seconds.
# maxsize=4 covers typical multi-instance use.
_reachability_cache: TTLCache = TTLCache(maxsize=4, ttl=30)
_reachability_status_cache: TTLCache = TTLCache(maxsize=4, ttl=30)


def _cursor_api_cache_key(base_url: str, token: str) -> tuple[str, str]:
    """Build cache key for cursor-api checks without storing full token."""
    token_hash = hashlib.sha256(token.encode()).hexdigest()[:16] if token else ""
    return (base_url, token_hash)


def _resolve_codex() -> str:
    """Resolve codex CLI path."""
    found = shutil.which("codex")
    if found:
        return found
    local = Path.home() / ".local" / "bin" / "codex"
    if local.exists():
        return str(local)
    return "codex"


def _check_cursor_api_reachable(
    base_url: str,
    token: str,
    timeout: float = 3.0,
) -> tuple[bool, bool, int | None]:
    """Perform the actual HTTP reachability check for cursor-api (GET /v1/models).

    Returns:
        tuple: (is_reachable: bool, is_connection_error: bool, status_code: int | None)
    """
    import httpx

    url = f"{base_url.rstrip('/')}/v1/models"
    headers = {"Authorization": f"Bearer {token}"} if token else {}
    try:
        resp = httpx.get(url, headers=headers, timeout=timeout)
        return (resp.status_code == 200, False, resp.status_code)
    except Exception:
        return (False, True, None)


def _is_cursor_api_reachable(base_url: str, token: str, timeout: float = 3.0) -> bool:
    """Check if cursor-api is reachable, with result cached for 30 seconds.

    The underlying HTTP request is made at most once per TTL window per
    (base_url, token_hash) pair, avoiding per-invocation latency.
    On connection failure, the cache entry is reset to allow retry.
    """
    cache_key = _cursor_api_cache_key(base_url, token)
    cached = _reachability_cache.get(cache_key)
    if cached is not None:
        return cached
    result, is_connection_error, status_code = _check_cursor_api_reachable(base_url, token, timeout)
    # Reset cache entry on connection failure to allow retry
    if is_connection_error:
        _reachability_cache.pop(cache_key, None)
        _reachability_status_cache.pop(cache_key, None)
    else:
        _reachability_cache[cache_key] = result
        _reachability_status_cache[cache_key] = status_code
    return result


def _sanitize_stderr(stderr: str) -> str:
    lines = [line for line in stderr.splitlines() if _MALLOC_STACK_NOISE not in line]
    if not lines:
        return ""
    return "\n".join(lines) + ("\n" if stderr.endswith("\n") else "")


@with_retry(max_attempts=4, min_wait=2.0, max_wait=60.0)
def _run_with_retry(
    cmd: list[str],
    prompt: str,
    cwd: Path | None,
    timeout: int,
    env: dict[str, str],
) -> RunResult:
    """Run codex subprocess; raises TransientAgentError on retryable failure."""
    proc = shim_run(
        cmd,
        check=False,
        input=prompt,
        capture_output=True,
        text=True,
        timeout=timeout + 5,
        cwd=str(cwd) if cwd else None,
        env=env,
    )
    result = RunResult(
        exit_code=proc.returncode,
        stdout=strip_ansi(proc.stdout),
        stderr=_sanitize_stderr(strip_ansi(proc.stderr)),
        timed_out=proc.returncode == 124,
    )
    if result.exit_code != 0 and is_retryable(result):
        raise TransientAgentError(result)
    return result


class CursorApiRunner(AgentRunner):
    """Runs Cursor models via cursor-api (wisdgod) - OpenAI-compatible HTTP backend."""

    def __init__(
        self,
        settings: ThegentSettings | None = None,
        model: str = "",
    ) -> None:
        self._settings = settings or ThegentSettings()
        self._model = model or _PROXY_MODEL

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
        env: dict[str, str] | None = None,
        run_id: str | None = None,
        session_id: str | None = None,
    ) -> RunResult:
        model = agent_model or self._model
        base_url = self._settings.cursor_api_url.rstrip("/")
        token = self._settings.cursor_api_token or ""

        cache_key = _cursor_api_cache_key(base_url, token)
        if not _is_cursor_api_reachable(base_url, token):
            status_code = _reachability_status_cache.get(cache_key)
            if status_code in (401, 403):
                message = "cursor-api auth failed (HTTP 401/403). Verify THGENT_CURSOR_API_TOKEN and token scope."
            elif status_code:
                message = (
                    f"cursor-api returned HTTP {status_code}. Check endpoint at {base_url} and THGENT_CURSOR_API_TOKEN."
                )
            else:
                message = (
                    "cursor-api not reachable. Start cursor-api (wisdgod) at "
                    f"{base_url} or set THGENT_CURSOR_API_URL. "
                    "Set THGENT_CURSOR_API_TOKEN for auth."
                )
            unreachable_result = RunResult(
                exit_code=1,
                stdout="",
                stderr=message,
                timed_out=False,
            )
            if run_id is not None or session_id is not None:
                dispatch_post_agent_run_hook(
                    result=unreachable_result,
                    run_id=run_id,
                    session_id=session_id,
                    cwd=cwd,
                    extra_context={
                        "agent": "cursor-api",
                        "model": model,
                        "mode": mode,
                        "timeout_seconds": timeout,
                    },
                )
            return unreachable_result

        process_env = os.environ.copy()
        process_env["OPENAI_BASE_URL"] = base_url
        process_env["OPENAI_API_KEY"] = token or "sk-dummy"
        if env:
            process_env.update(env)

        codex_cmd = _resolve_codex()
        cmd = [codex_cmd, "exec", "-", "--skip-git-repo-check"]
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

        cmd = wrap_with_caffeinate(cmd, "cursor-api")

        # @trace WL-038 — capture result so $defer directives can be post-processed
        _result: RunResult
        try:
            _result = _run_with_retry(cmd, prompt, cwd, timeout, process_env)
        except TransientAgentError as e:
            _result = e.result
        except FileNotFoundError:
            _result = RunResult(
                exit_code=1,
                stdout="",
                stderr=("codex CLI not found. Install: npm i -g @openai/codex\nOr add codex to PATH."),
                timed_out=False,
            )
        except subprocess.TimeoutExpired:
            _result = RunResult(
                exit_code=124,
                stdout="",
                stderr=f"Agent timed out after {timeout}s",
                timed_out=True,
            )
        if run_id is not None or session_id is not None:
            dispatch_post_agent_run_hook(
                result=_result,
                run_id=run_id,
                session_id=session_id,
                cwd=cwd,
                extra_context={
                    "agent": "cursor-api",
                    "model": model,
                    "mode": mode,
                    "timeout_seconds": timeout,
                },
            )
        return self._process_output_deferrals(_result, cwd=cwd)
