"""Use case: Run a harness (Claude/Codex) with model/provider routing."""

import os
from pathlib import Path
from typing import Optional

from thegent.adapters.claude_harness import ClaudeHarness
from thegent.adapters.codex_harness import CodexHarness
from thegent.adapters.harness_base import HarnessBase


class RunHarness:
    """Orchestrate harness execution (run_interactive, run_exec, etc.)."""

    def __init__(self, harness_type: str = "claude"):
        """Initialize with harness type ('claude' or 'codex')."""
        if harness_type.lower() in ("claude", "clode"):
            self.harness: HarnessBase = ClaudeHarness()
        elif harness_type.lower() in ("codex", "dex"):
            self.harness = CodexHarness()
        else:
            raise ValueError(f"Unknown harness type: {harness_type}")
        self.console = self.harness.console

    def run_interactive(
        self,
        model_alias: str,
        provider: Optional[str] = None,
        resume: Optional[str] = None,
        prompt: Optional[str] = None,
        *,
        cd: Optional[Path] = None,
        print_mode: bool = False,
        debug: bool = False,
        add_dir: Optional[list[str]] = None,
        sandbox: Optional[str] = None,
        full_auto: bool = False,
        search: bool = True,
        no_alt_screen: bool = False,
        continue_session: bool = False,
    ) -> None:
        """Start interactive session with model/provider routing."""
        model_alias_map = self.harness.get_model_alias_map()
        canonical = model_alias_map.get(model_alias.lower(), model_alias)

        if provider:
            provider = provider.strip().lower()
        else:
            provider = self.harness.resolve_provider_for_model(model_alias)

        if print_mode:
            if not prompt:
                self.console.print("[red]Error: --print requires a prompt.[/red]")
                raise SystemExit(1)
            self._run_exec_impl(canonical, prompt, cd=cd, add_dir=add_dir, sandbox=sandbox)
            return

        extra: list[str] = self._build_passthrough_args(
            cd=cd,
            debug=debug,
            add_dir=add_dir,
            sandbox=sandbox,
            full_auto=full_auto,
            search=search,
            no_alt_screen=no_alt_screen,
        )

        if resume:
            extra.extend(["--resume", resume])
        if prompt:
            extra.append(prompt)
        if cd:
            os.chdir(cd)

        model_override = self.harness.get_model_alias_map().get(canonical, canonical)
        self.harness.run_interactive(
            provider,
            extra_args=extra or None,
            model_override=model_override,
        )

    def _run_exec_impl(
        self,
        model: str,
        prompt: str,
        cd: Optional[Path] = None,
        add_dir: Optional[list[str]] = None,
        sandbox: Optional[str] = None,
    ) -> None:
        """Headless execution (subclass override for Codex-specific logic)."""
        raise NotImplementedError("Use case must handle exec mode per harness")

    def _build_passthrough_args(
        self,
        *,
        cd: Optional[Path] = None,
        debug: bool = False,
        add_dir: Optional[list[str]] = None,
        sandbox: Optional[str] = None,
        full_auto: bool = False,
        search: bool = True,
        no_alt_screen: bool = False,
    ) -> list[str]:
        """Build passthrough args for harness CLI."""
        args: list[str] = []
        if cd:
            args.extend(["-C", str(cd.resolve())])
        if debug:
            args.append("--debug")
        if add_dir:
            for d in add_dir:
                args.extend(["--add-dir", d])
        if sandbox:
            args.extend(["--sandbox", sandbox])
        if full_auto:
            args.append("--full-auto")
        if search:
            args.append("--search")
        if no_alt_screen:
            args.append("--no-alt-screen")
        return args

    def run_native(self, args: Optional[list[str]] = None) -> None:
        """Bypass proxy and run native binary directly."""
        binary_path = self.harness.find_binary(require_native=True)
        if not binary_path:
            self.console.print(
                f"[red]Error: native '{self.harness.get_binary_name()}' CLI not found.[/red]"
            )
            raise SystemExit(1)

        cmd = [binary_path]
        if args:
            cmd.extend(args)

        self.console.print(f"[bold green]Starting native {self.harness.get_binary_name()} (proxy bypass)...[/bold green]")
        os.execvpe(cmd[0], cmd, os.environ.copy())

    def ensure_harness_installed(self) -> str:
        """Ensure harness binary is installed. Returns path."""
        return self.harness.ensure_binary_installed()
