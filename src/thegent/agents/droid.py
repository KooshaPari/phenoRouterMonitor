"""Droid runner - invokes Factory droid exec, OpenAI Codex CLI, or custom CLI backends."""

import os
import subprocess
import tempfile
from collections.abc import Callable
from pathlib import Path

from thegent.agents.base import AgentRunner, RunResult
from thegent.config import ThegentSettings
from thegent.infra import run_subprocess_optimized
from thegent.infra.power import wrap_with_caffeinate
from thegent.utils import strip_ansi


def _resolve_cmd(cmd: str, candidates: list[Path] | None = None) -> str:
    """Resolve command: use as-is if absolute path exists, else try common locations."""
    expanded = str(Path(cmd).expanduser()) if "~" in cmd or "/" in cmd else cmd
    if os.path.isabs(expanded) and Path(expanded).exists():
        return expanded
    if cmd != expanded and Path(expanded).exists():
        return str(Path(expanded).resolve())
    if candidates:
        for c in candidates:
            if c.exists():
                return str(c)
    return cmd


def _resolve_droid_cmd(cmd: str) -> str:
    """Resolve droid command."""
    return _resolve_cmd(
        cmd,
        candidates=[
            Path.home() / ".local" / "bin" / "droid",
            Path.home() / ".factory" / "bin" / "droid",
        ],
    )


def _resolve_codex_cmd(cmd: str) -> str:
    """Resolve Codex CLI command."""
    return _resolve_cmd(
        cmd,
        candidates=[
            Path.home() / ".local" / "bin" / "codex",
            Path.home() / ".codex" / "bin" / "codex",
        ],
    )


def _model_requires_streaming(model: str) -> bool:
    """Return True when the target model/provider requires streamed responses."""
    lowered = model.strip().lower()
    return "minimax" in lowered


class DroidRunner(AgentRunner):
    """Runs droids via Factory droid exec."""

    def __init__(
        self,
        droid_name: str,
        droids_dir: Path,
        droid_cmd: str = "droid",
        model: str = "custom:minimax-m2.5",
        use_litellm_router: bool | None = None,
    ) -> None:
        self.droid_name = droid_name
        self.droids_dir = droids_dir.expanduser().resolve()
        self._droid_cmd = _resolve_droid_cmd(droid_cmd)
        self._model = model
        settings = ThegentSettings()
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
        env: dict[str, str] | None = None,
        image_paths: list[str] | None = None,
    ) -> RunResult:
        """Run droid via droid exec."""
        # Route via LiteLLM Router if enabled
        if self._use_litellm_router:
            return self._run_via_litellm_router(
                prompt, cwd, mode, timeout, self._model, use_stream, live_output, on_stdout, on_stderr, env=env
            )

        droid_path = self.droids_dir / f"{self.droid_name}.md"
        if not droid_path.exists():
            return RunResult(
                exit_code=1,
                stdout="",
                stderr=f"Droid not found: {droid_path}",
                timed_out=False,
            )

        droid_content = droid_path.read_text()
        combined = f"{droid_content.rstrip()}\n\n---\nUser request: {prompt}"
        tmp_path: str | None = None
        try:
            with tempfile.NamedTemporaryFile(mode="w", suffix=".md", delete=False) as f:
                f.write(combined)
                tmp_path = f.name

            cmd = [self._droid_cmd, "exec", "-f", tmp_path, "--model", self._model]
            if cwd:
                cmd.extend(["--cwd", str(cwd)])
            if use_stream or _model_requires_streaming(self._model):
                cmd.extend(["--output-format", "stream-json"])
            if mode == "write":
                cmd.extend(["--auto", "low"])
            elif mode == "full":
                cmd.extend(["--auto", "high"])

            cmd = wrap_with_caffeinate(cmd, "droid")

            # Merge provided env with current environment
            process_env = os.environ.copy()
            if env:
                process_env.update(env)

            proc = run_subprocess_optimized(
                cmd,
                check=False,
                capture_output=True,
                timeout=timeout + 5,
                cwd=str(cwd) if cwd else None,
                env=process_env,
            )
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
            return RunResult(
                exit_code=proc.returncode,
                stdout=strip_ansi(stdout_text),
                stderr=strip_ansi(stderr_text),
                timed_out=proc.returncode == 124,
            )
        except FileNotFoundError:
            return RunResult(
                exit_code=1,
                stdout="",
                stderr=(
                    "droid CLI not found. Install: curl -fsSL https://app.factory.ai/cli | sh\n"
                    "Then add ~/.local/bin to PATH, or set THGENT_DROID_CMD=/path/to/droid"
                ),
                timed_out=False,
            )
        except subprocess.TimeoutExpired:
            return RunResult(
                exit_code=124,
                stdout="",
                stderr=f"Droid timed out after {timeout}s",
                timed_out=True,
            )
        finally:
            if tmp_path is not None:
                Path(tmp_path).unlink(missing_ok=True)

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
        """Run via LiteLLM Router for Droid compatibility."""
        try:
            from thegent.utils.routing_impl.litellm_router import get_enhanced_router

            router = get_enhanced_router()

            # Extract provider/model from Factory format
            model_to_use = model.split(":", 1)[1] if ":" in model else model

            # Check if droid file exists
            droid_path = self.droids_dir / f"{self.droid_name}.md"
            if droid_path.exists():
                droid_content = droid_path.read_text()
                combined_prompt = f"{droid_content.rstrip()}\n\n---\nUser request: {prompt}"
            else:
                combined_prompt = prompt

            result = router.route(combined_prompt, model=model_to_use, stream=use_stream, timeout=timeout)

            if not result.success:
                return RunResult(
                    exit_code=1,
                    stdout="",
                    stderr=result.error or "Routing failed",
                    timed_out=False,
                )

            # Handle response
            if use_stream:
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
            return RunResult(
                exit_code=1,
                stdout="",
                stderr=f"LiteLLM Router execution failed: {e}",
                timed_out=False,
            )


class CodexRunner(AgentRunner):
    """Runs droids via OpenAI Codex CLI (codex exec)."""

    def __init__(
        self,
        droid_name: str,
        droids_dir: Path,
        codex_cmd: str = "codex",
        model: str = "gpt-5.3-codex-spark-xhigh",
        use_litellm_router: bool | None = None,
    ) -> None:
        self.droid_name = droid_name
        self.droids_dir = droids_dir.expanduser().resolve()
        self._codex_cmd = _resolve_codex_cmd(codex_cmd)
        self._model = model
        settings = ThegentSettings()
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
        env: dict[str, str] | None = None,
        image_paths: list[str] | None = None,
    ) -> RunResult:
        """Run droid via codex exec."""
        # Route via LiteLLM Router if enabled
        if self._use_litellm_router:
            return self._run_via_litellm_router(
                prompt, cwd, mode, timeout, self._model, use_stream, live_output, on_stdout, on_stderr, env=env
            )

        droid_path = self.droids_dir / f"{self.droid_name}.md"
        if not droid_path.exists():
            return RunResult(
                exit_code=1,
                stdout="",
                stderr=f"Droid not found: {droid_path}",
                timed_out=False,
            )

        droid_content = droid_path.read_text()
        combined = f"{droid_content.rstrip()}\n\n---\nUser request: {prompt}"

        cmd = [
            self._codex_cmd,
            "exec",
            "-",
            "--model",
            self._model,
        ]
        if cwd:
            cmd.extend(["--cd", str(cwd)])
        if use_stream:
            cmd.extend(["--json"])
        if mode == "write":
            cmd.extend(["--sandbox", "workspace-write"])
        elif mode == "full":
            cmd.extend(["--full-auto"])

        cmd = wrap_with_caffeinate(cmd, "codex")

        # Merge provided env with current environment
        process_env = os.environ.copy()
        if env:
            process_env.update(env)

        try:
            # Use run_subprocess_optimized with input support
            proc = run_subprocess_optimized(
                cmd,
                check=False,
                input=combined,
                capture_output=True,
                timeout=timeout + 5,
                cwd=str(cwd) if cwd else None,
                env=process_env,
            )
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
            return RunResult(
                exit_code=proc.returncode,
                stdout=strip_ansi(stdout_text),
                stderr=strip_ansi(stderr_text),
                timed_out=proc.returncode == 124,
            )
        except subprocess.TimeoutExpired:
            return RunResult(
                exit_code=124,
                stdout="",
                stderr=f"Codex timed out after {timeout}s",
                timed_out=True,
            )
        except FileNotFoundError:
            return RunResult(
                exit_code=1,
                stdout="",
                stderr=(
                    "Codex CLI not found. Install: npm i -g @openai/codex or brew install --cask codex\n"
                    "Then add to PATH, or set THGENT_DROID_CODEX_CMD=/path/to/codex"
                ),
                timed_out=False,
            )

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
        """Run via LiteLLM Router for Codex/Droid compatibility."""
        try:
            from thegent.utils.routing_impl.litellm_router import get_enhanced_router

            router = get_enhanced_router()

            # Check if droid file exists
            droid_path = self.droids_dir / f"{self.droid_name}.md"
            if droid_path.exists():
                droid_content = droid_path.read_text()
                combined_prompt = f"{droid_content.rstrip()}\n\n---\nUser request: {prompt}"
            else:
                combined_prompt = prompt

            result = router.route(combined_prompt, model=model, stream=use_stream, timeout=timeout)

            if not result.success:
                return RunResult(
                    exit_code=1,
                    stdout="",
                    stderr=result.error or "Routing failed",
                    timed_out=False,
                )

            # Handle response
            if use_stream:
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
            return RunResult(
                exit_code=1,
                stdout="",
                stderr=f"LiteLLM Router execution failed: {e}",
                timed_out=False,
            )


class CustomCliRunner(AgentRunner):
    """Runs droids via a generic custom CLI (e.g. claudemax, claudeglm in ~/.local/bin)."""

    def __init__(
        self,
        droid_name: str,
        droids_dir: Path,
        custom_cmd: str,
        model: str = "",
    ) -> None:
        self.droid_name = droid_name
        self.droids_dir = droids_dir.expanduser().resolve()
        # Resolve path if it looks like a path; otherwise use as-is (for PATH lookup)
        if not custom_cmd:
            self._custom_cmd = ""
        elif "/" in custom_cmd or custom_cmd.startswith("~"):
            self._custom_cmd = str(Path(custom_cmd).expanduser().resolve())
        else:
            self._custom_cmd = custom_cmd
        self._model = model

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
        env: dict[str, str] | None = None,
        image_paths: list[str] | None = None,
    ) -> RunResult:
        """Run droid via custom CLI. Prompt sent via stdin; expects --model and --cd support."""
        droid_path = self.droids_dir / f"{self.droid_name}.md"
        if not droid_path.exists():
            return RunResult(
                exit_code=1,
                stdout="",
                stderr=f"Droid not found: {droid_path}",
                timed_out=False,
            )

        droid_content = droid_path.read_text()
        combined = f"{droid_content.rstrip()}\n\n---\nUser request: {prompt}"

        cmd = [self._custom_cmd]
        if self._model:
            cmd.extend(["--model", self._model])
        if cwd:
            cmd.extend(["--cd", str(cwd)])

        cmd = wrap_with_caffeinate(cmd, self.droid_name)

        # Merge provided env with current environment
        process_env = os.environ.copy()
        if env:
            process_env.update(env)

        try:
            # Use run_subprocess_optimized with input support
            proc = run_subprocess_optimized(
                cmd,
                check=False,
                input=combined,
                capture_output=True,
                timeout=timeout + 5,
                cwd=str(cwd) if cwd else None,
                env=process_env,
            )
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
            return RunResult(
                exit_code=proc.returncode,
                stdout=strip_ansi(stdout_text),
                stderr=strip_ansi(stderr_text),
                timed_out=proc.returncode == 124,
            )
        except subprocess.TimeoutExpired:
            return RunResult(
                exit_code=124,
                stdout="",
                stderr=f"Custom CLI timed out after {timeout}s",
                timed_out=True,
            )
        except FileNotFoundError:
            return RunResult(
                exit_code=1,
                stdout="",
                stderr=(
                    f"Custom CLI not found: {self._custom_cmd}\n"
                    "Set THGENT_DROID_CUSTOM_CMD to path (e.g. ~/.local/bin/claudemax)"
                ),
                timed_out=False,
            )


def get_droid_runner(
    backend: str,
    droid_name: str,
    droids_dir: Path,
    *,
    droid_cmd: str = "droid",
    droid_model: str = "custom:minimax-m2.5",
    codex_cmd: str = "codex",
    codex_model: str = "gpt-5.3-codex-spark-xhigh",
    custom_cmd: str = "",
    custom_model: str = "",
) -> AgentRunner:
    """Factory: return the appropriate droid runner for the given backend."""
    backend_lower = backend.lower().strip()
    if backend_lower == "codex":
        return CodexRunner(
            droid_name,
            droids_dir,
            codex_cmd=codex_cmd,
            model=codex_model,
        )
    if backend_lower == "custom" and custom_cmd:
        return CustomCliRunner(
            droid_name,
            droids_dir,
            custom_cmd=custom_cmd,
            model=custom_model or droid_model,
        )
    return DroidRunner(
        droid_name,
        droids_dir,
        droid_cmd=droid_cmd,
        model=droid_model,
    )
