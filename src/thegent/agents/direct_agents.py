"""Direct agent invocation - cursor, claude, copilot, codex, gemini, opencode via CLIs."""

import os
import re
import shutil
import subprocess
import threading
from collections.abc import Callable
from pathlib import Path
from typing import TYPE_CHECKING, Any, cast

from thegent.agents.base import AgentRunner, RunResult
from thegent.agents.resilience import TransientAgentError, is_retryable, with_retry
from thegent.infra.fast_subprocess import run_subprocess_optimized
from thegent.infra.power import wrap_with_caffeinate
from thegent.utils import strip_ansi

if TYPE_CHECKING:
    from thegent.config import ThegentSettings

PROCESS_TIMEOUT_SECS = 3600

_NOISY_STDERR_PATTERNS = (
    r"\(node:\d+\) \[DEP0040\].*punycode",
    r"Session cleanup disabled:",
    r"Hook registry initialized with \d+ hook entries",
    r'Error executing tool run_shell_command: Tool "run_shell_command" not found',
    r"Use `node --trace-deprecation",
    r"^Loaded cached credentials\.$",
    r"^\[OK\] ",
    r"^\[INFO\] ",
    r"^Total usage est:",
    r"^Total duration ",
    r"^Total code changes:",
    r"^Usage by model:",
    r"^Copilot CLI available",
    r"^Commit:",
)


def _filter_noisy_stderr(text: str) -> str:
    if not text:
        return text
    lines = []
    for line in text.splitlines():
        if any(re.search(p, line) for p in _NOISY_STDERR_PATTERNS):
            continue
        lines.append(line)
    return "\n".join(lines).rstrip()


def _resolve_cli(cmd: str, name: str, settings: "ThegentSettings | None" = None) -> str:
    """Resolve CLI path: settings override, env override, absolute path, which, or ~/.local/bin."""
    if settings is None:
        from thegent.config import ThegentSettings

        settings = ThegentSettings()

    # Check settings first (cursor_agent_cmd for cursor-agent)
    if name == "cursor-agent" and settings.cursor_agent_cmd != "cursor":
        cmd_from_settings = settings.cursor_agent_cmd
        expanded = str(Path(cmd_from_settings).expanduser())
        if Path(expanded).exists():
            return expanded
        return cmd_from_settings

    # Fallback to environment variable override
    env_key = "THGENT_CURSOR_AGENT_CMD" if name == "cursor-agent" else f"THGENT_{name.upper().replace('-', '_')}_CMD"
    env_val = os.environ.get(env_key)
    if env_val:
        expanded = str(Path(env_val).expanduser())
        if Path(expanded).exists():
            return expanded
        return env_val
    if "/" in cmd or "~" in cmd:
        expanded = str(Path(cmd).expanduser())
        if Path(expanded).exists():
            return expanded
    found = shutil.which(cmd)
    if found:
        return found
    # cursor-agent: fallback to cursor if cursor-agent not on PATH (Cursor IDE CLI)
    if name == "cursor-agent":
        fallback = shutil.which("cursor")
        if fallback:
            return fallback
    local = Path.home() / ".local" / "bin" / name
    if local.exists():
        return str(local)
    if name == "cursor-agent":
        local_cursor = Path.home() / ".local" / "bin" / "cursor"
        if local_cursor.exists():
            return str(local_cursor)
    return cmd


# Agent name -> (cli_cmd, uses_stdin, stream_arg)
_AGENT_CLI: dict[str, tuple[str, bool, str]] = {
    "cursor-agent": ("cursor-agent", False, "--print"),
    "claude": ("claude", True, "--output-format stream-json"),
    "copilot": ("copilot", False, "--stream on"),
    "codex": ("codex", True, "--json"),
    "gemini": ("gemini", False, "--output-format stream-json"),
    "opencode": ("opencode", False, ""),
}


def _wrap_with_harness(cmd: list[str], agent_name: str | None = None) -> list[str]:
    """WP-10010: Wrap command with unified harness (thegent-hooks or heliosShield)."""
    from thegent.config import ThegentSettings

    settings = ThegentSettings()
    if not settings.helios_shield_enabled:
        return cmd

    # Phase 2/3: Prefer thegent-hooks Rust harness
    harness_bin = None

    # Check for thegent-hooks first
    if os.environ.get("THEGENT_HOOKS_RUST", "1") == "1":
        harness_bin = shutil.which("thegent-hooks")
        if harness_bin:
            if agent_name:
                # thegent-hooks agent <agent_name> -- <cmd> is the unified mesh entry point
                return [harness_bin, "agent", agent_name, "--", *cmd]
            # Fallback for generic git commands if no agent name provided
            return [harness_bin, "git", *cmd]

    # Fallback to legacy heliosShield harness
    harness_bin = shutil.which("harness")
    if not harness_bin:
        # Check relative path to heliosShield
        potential = Path.cwd().parent / "heliosShield" / "bin" / "harness"
        if potential.exists():
            harness_bin = str(potential)
        else:
            # Check workspace root
            settings = ThegentSettings()
            potential = settings.factory_skills_dir.parent.parent / "heliosShield" / "bin" / "harness"
            if potential.exists():
                harness_bin = str(potential)

    if harness_bin:
        # heliosShield harness currently does not support opencode invocation shape.
        if cmd and cmd[0] == "opencode":
            return cmd
        return [harness_bin, *cmd]
    return cmd


class DirectAgentRunner(AgentRunner):
    """Invokes cursor, claude, copilot, codex, gemini directly via their CLIs."""

    def __init__(
        self,
        agent_name: str,
        cli_cmd: str | None = None,
        default_model: str = "",
        use_litellm_router: bool | None = None,
    ) -> None:
        self.agent_name = agent_name
        spec = _AGENT_CLI.get(agent_name)
        if not spec:
            raise ValueError(f"Unknown direct agent: {agent_name}")
        self._cli_name, self._uses_stdin, self._stream_arg = spec
        from thegent.config import ThegentSettings

        settings = ThegentSettings()
        self._cli_cmd = _resolve_cli(cli_cmd or self._cli_name, self._cli_name, settings)
        self._default_model = default_model
        self._use_litellm_router = use_litellm_router if use_litellm_router is not None else settings.use_litellm_router

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
        image_paths: list[str] | None = None,
        audio_paths: list[str] | None = None,
        env: dict[str, str] | None = None,
    ) -> RunResult:
        model = agent_model or self._default_model

        # WL-116: Handle audio transcript inputs
        audio_transcript: str | None = None
        if audio_paths:
            from thegent.agents.audio_inputs import inject_transcript_into_prompt, load_transcripts

            audio_transcript, _audio_sources = load_transcripts(audio_paths)
            if audio_transcript:
                # Inject transcript into prompt for direct agents
                prompt = inject_transcript_into_prompt(prompt, audio_transcript)

        # Route via LiteLLM Router if enabled and not opencode
        if self._use_litellm_router and self.agent_name != "opencode":
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

        # WP-Y6: OTel GenAI Instrumentation
        from thegent.observability.otel_instrumentation import instrument_genai_call

        system_map = {
            "claude": "anthropic",
            "gemini": "google",
            "codex": "openai",
            "copilot": "github",
            "cursor-agent": "cursor",
        }

        with instrument_genai_call(
            agent_name=self.agent_name,
            model=model,
            system=system_map.get(self.agent_name),
        ) as span:
            cmd = self._build_cmd(cwd, use_stream, model, mode, image_paths=image_paths)
            # WL-114: Claude Code CLI receives images via --input-format stream-json JSONL stdin.
            if self._uses_stdin and self.agent_name == "claude" and image_paths:
                from thegent.agents.image_inputs import build_claude_stdin_with_images

                stdin_input: str | None = build_claude_stdin_with_images(prompt, image_paths)
            else:
                stdin_input = prompt if self._uses_stdin else None
            if not self._uses_stdin:
                if self.agent_name == "gemini":
                    cmd.extend(["-p", prompt])
                elif self.agent_name == "copilot":
                    cmd.extend(["-p", prompt])  # copilot requires -p for non-interactive
                else:
                    cmd.append(prompt)

            # heliosShield harness currently does not support opencode invocation shape.
            if self.agent_name != "opencode":
                cmd = _wrap_with_harness(cmd, agent_name=self.agent_name)
            cmd = wrap_with_caffeinate(cmd, self.agent_name)

            # @trace WL-038 — capture result so $defer directives can be post-processed
            _dr: RunResult
            try:
                if live_output:
                    _dr = self._run_live(cmd, cwd, timeout, stdin_input, on_stdout, on_stderr, env=env)
                else:
                    _dr = self._run_capture(cmd, cwd, timeout, stdin_input, env=env)

                span.set_attribute("exit_code", _dr.exit_code)
            except FileNotFoundError:
                env_hint = (
                    "THGENT_CURSOR_AGENT_CMD"
                    if self._cli_name == "cursor-agent"
                    else f"THGENT_{self._cli_name.upper().replace('-', '_')}_CMD"
                )
                _dr = RunResult(
                    exit_code=1,
                    stdout="",
                    stderr=(
                        f"{self._cli_name} not found. Install and add to PATH, or set {env_hint}=/path/to/{self._cli_name}"
                    ),
                    timed_out=False,
                )
                span.set_attribute("exit_code", 1)
                cast("Any", span).record_exception(FileNotFoundError(_dr.stderr))
            except subprocess.TimeoutExpired:
                _dr = RunResult(
                    exit_code=124,
                    stdout="",
                    stderr=f"Agent timed out after {timeout}s",
                    timed_out=True,
                )
                span.set_attribute("exit_code", 124)
                span.set_attribute("timed_out", True)

            # WL-116: Add audio_transcript to result if audio was processed
            if audio_transcript:
                _dr = RunResult(
                    exit_code=_dr.exit_code,
                    stdout=_dr.stdout,
                    stderr=_dr.stderr,
                    timed_out=_dr.timed_out,
                    context_tokens_used=_dr.context_tokens_used,
                    context_window_max=_dr.context_window_max,
                    context_usage_ratio=_dr.context_usage_ratio,
                    audio_transcript=audio_transcript,
                    grounding_sources=_dr.grounding_sources,
                )

            return self._process_output_deferrals(_dr, cwd=cwd)

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
        """Run via LiteLLM Router for direct CLI compatibility."""
        try:
            from thegent.utils.routing_impl.litellm_router import get_enhanced_router

            router = get_enhanced_router()
            # Map provider if possible
            provider = self.agent_name
            # If model already has provider/ prefix, use it
            model_to_use = model if "/" in model else f"{provider}/{model}"

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
                # For direct CLIs, we need to collect the stream into a single response
                # or handle it as expected. Since we're emulating direct CLIs,
                # we'll collect it for now unless we implement full SSE.
                stdout_collector = []
                response_iter = result.response
                if response_iter is None:
                    return RunResult(
                        exit_code=1,
                        stdout="",
                        stderr="No response from router",
                        timed_out=False,
                    )
                for chunk in response_iter:
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
            if result.response is not None:
                if hasattr(result.response, "choices") and result.response.choices:
                    content = result.response.choices[0].message.content
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
            # Fallback to direct CLI execution if LiteLLM Router fails
            # This ensures robustness
            return RunResult(
                exit_code=1,
                stdout="",
                stderr=f"LiteLLM Router execution failed: {e}",
                timed_out=False,
            )

    def _build_cmd(
        self,
        cwd: Path | None,
        use_stream: bool,
        model: str,
        mode: str,
        *,
        image_paths: list[str] | None = None,
    ) -> list[str]:
        cmd = [self._cli_cmd]

        if self.agent_name == "codex":
            cmd.extend(["exec", "-", "--skip-git-repo-check"])
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
            return cmd

        if self.agent_name == "cursor-agent":
            cmd.extend(["--print"])
            if mode != "read-only":
                cmd.append("--trust")
            if cwd:
                cmd.extend(["--workspace", str(cwd)])
            if model:
                cmd.extend(["--model", model])
            return cmd

        if self.agent_name == "claude":
            cmd.extend(["--print"])
            if mode != "read-only":
                cmd.append("--dangerously-skip-permissions")
            if cwd:
                cmd.extend(["--add-dir", str(cwd)])
            if use_stream:
                cmd.extend(self._stream_arg.split())
                cmd.append("--verbose")  # required with --print + stream-json
            if model:
                cmd.extend(["--model", model])
            # WL-114: image content blocks require stream-json input format.
            if image_paths:
                cmd.extend(["--input-format", "stream-json"])
            return cmd

        if self.agent_name == "copilot":
            if cwd:
                cmd.extend(["--add-dir", str(cwd)])
            if mode != "read-only":
                cmd.append("--allow-all-tools")
            if use_stream:
                cmd.extend(self._stream_arg.split())
            if model:
                cmd.extend(["--model", model])
            return cmd

        if self.agent_name == "gemini":
            if cwd:
                cmd.extend(["--include-directories", str(cwd)])
            if use_stream:
                cmd.extend(["-o", "stream-json"])
            if model:
                cmd.extend(["-m", model])
            return cmd

        if self.agent_name == "opencode":
            # OpenCode non-interactive execution path.
            cmd.append("run")
            if model:
                cmd.extend(["-m", model])
            return cmd

        return cmd

    def _run_capture(
        self,
        cmd: list[str],
        cwd: Path | None,
        timeout: int,
        stdin_input: str | None,
        env: dict[str, str] | None = None,
    ) -> RunResult:
        try:
            return self._run_capture_attempt(cmd, cwd, timeout, stdin_input, env=env)
        except TransientAgentError as e:
            return e.result

    @with_retry(max_attempts=4, min_wait=2.0, max_wait=60.0)
    def _run_capture_attempt(
        self,
        cmd: list[str],
        cwd: Path | None,
        timeout: int,
        stdin_input: str | None,
        env: dict[str, str] | None = None,
    ) -> RunResult:
        kwargs: dict = {
            "capture_output": True,
            "timeout": min(timeout + 10, PROCESS_TIMEOUT_SECS + 10),
            "cwd": str(cwd) if cwd else None,
            "env": env,
        }
        if stdin_input is not None:
            kwargs["input"] = stdin_input.encode() if isinstance(stdin_input, str) else stdin_input
        else:
            kwargs["stdin"] = subprocess.DEVNULL
        proc = run_subprocess_optimized(cmd, check=False, **cast("Any", kwargs))
        stdout_text = (
            proc.stdout
            if isinstance(proc.stdout, str)
            else (proc.stdout.decode("utf-8", errors="replace") if proc.stdout else "")
        )
        stderr_text = (
            proc.stderr
            if isinstance(proc.stderr, str)
            else (proc.stderr.decode("utf-8", errors="replace") if proc.stderr else "")
        )
        result = RunResult(
            exit_code=proc.returncode,
            stdout=strip_ansi(stdout_text),
            stderr=_filter_noisy_stderr(strip_ansi(stderr_text)),
            timed_out=proc.returncode == 124,
        )
        if result.exit_code != 0 and is_retryable(result):
            raise TransientAgentError(result)
        return result

    def _run_live(
        self,
        cmd: list[str],
        cwd: Path | None,
        timeout: int,
        stdin_input: str | None,
        on_stdout: Callable[[str], None] | None,
        on_stderr: Callable[[str], None] | None,
        env: dict[str, str] | None = None,
    ) -> RunResult:
        proc = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            stdin=subprocess.PIPE if stdin_input else subprocess.DEVNULL,
            text=True,
            bufsize=1,
            cwd=str(cwd) if cwd else None,
            env=env,
        )
        if stdin_input and proc.stdin:
            proc.stdin.write(stdin_input)
            proc.stdin.close()
        out_lines: list[str] = []
        err_lines: list[str] = []

        def _drain(stream, collector: list[str], cb: Callable[[str], None] | None) -> None:
            for line in stream:
                clean = strip_ansi(line)
                collector.append(clean)
                if cb:
                    cb(clean.rstrip("\n"))

        t_out = threading.Thread(target=_drain, args=(proc.stdout, out_lines, on_stdout), daemon=True)
        t_err = threading.Thread(target=_drain, args=(proc.stderr, err_lines, on_stderr), daemon=True)
        t_out.start()
        t_err.start()
        try:
            rc = proc.wait(timeout=PROCESS_TIMEOUT_SECS + 10)
        except subprocess.TimeoutExpired:
            proc.kill()
            rc = 124
        t_out.join(timeout=1)
        t_err.join(timeout=1)
        return RunResult(
            exit_code=rc,
            stdout="".join(out_lines),
            stderr=_filter_noisy_stderr("".join(err_lines)),
            timed_out=rc == 124,
        )
