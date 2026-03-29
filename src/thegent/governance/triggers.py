"""AgilePlus trigger modes: watchdog, timer, and manual.

Provides three ways to trigger governance cycles:
- Watchdog: File system watcher with debounce (uses watchfiles for 5-10x performance)
- Timer: Periodic interval-based triggering
- Manual: One-shot CLI invocation
"""

from __future__ import annotations

import logging
import signal
import sys
import threading
import time
from pathlib import Path
from typing import Any, Protocol

import typer
from pydantic import BaseModel

# Use watchfiles (fast) with watchdog fallback
try:
    from watchfiles import Change, watch

    WATCHFILES_AVAILABLE = True
except ImportError:
    WATCHFILES_AVAILABLE = False
    try:
        from watchdog.events import FileSystemEventHandler
        from watchdog.observers import Observer

        WATCHDOG_AVAILABLE = True
    except ImportError:
        WATCHDOG_AVAILABLE = False

_log = logging.getLogger(__name__)
app = typer.Typer(
    name="agileplus-triggers",
    help="AgilePlus governance trigger",
    add_completion=False,
    no_args_is_help=False,
)

# Default debounce time for watchdog mode
DEFAULT_DEBOUNCE_SECONDS = 30


class TriggerConfig(BaseModel):
    """Configuration for trigger modes."""

    mode: str = "manual"  # watchdog, timer, manual
    interval_seconds: int = 300
    debounce_seconds: int = DEFAULT_DEBOUNCE_SECONDS
    watch_paths: list[str] = ["src/", "tests/", "hooks/"]
    max_cycles: int | None = None
    health_threshold: float = 90.0
    max_tasks_per_cycle: int = 10


class TriggerProtocol(Protocol):
    """Protocol for trigger implementations."""

    def start(self) -> None: ...
    def stop(self) -> None: ...
    def run(self, force: bool = False) -> Any: ...


class WatchdogTrigger:
    """File system watcher trigger with debounce.

    Uses watchfiles (5-10x faster) or watchdog fallback to watch specified paths
    for changes and triggers cycles after a debounce period without new changes.
    """

    # Directories to exclude from watching
    EXCLUDE_DIRS = frozenset(
        {
            ".git",
            "node_modules",
            "__pycache__",
            ".venv",
            "venv",
            "dist",
            "build",
            ".claude",
            ".thegent",
            ".ruff_cache",
            ".pytest_cache",
            "docs-dist",
        }
    )

    # File extensions to watch
    WATCH_EXTENSIONS = frozenset({".py", ".sh", ".yaml", ".json", ".md"})

    def __init__(
        self,
        loop: Any,
        config: TriggerConfig,
    ) -> None:
        self.loop = loop
        self.config = config
        self._running = False
        self._stop_event = threading.Event()
        self._debounce_timer: threading.Timer | None = None
        self._last_trigger_time = 0.0
        self._watch_thread: threading.Thread | None = None
        self._use_watchfiles = WATCHFILES_AVAILABLE
        # Fallback to watchdog if watchfiles not available
        if not self._use_watchfiles and WATCHDOG_AVAILABLE:
            self._observer: "Observer | None" = None  # type: ignore[valid-type]
            self._handler: FileSystemEventHandler | None = None  # type: ignore[valid-type]
        else:
            self._observer = None
            self._handler = None

    def start(self) -> None:
        """Start the watchdog trigger."""
        self._running = True
        backend = "watchfiles" if self._use_watchfiles else "watchdog"
        _log.info(
            "Starting watchdog trigger (%s): paths=%s, debounce=%ds",
            backend,
            self.config.watch_paths,
            self.config.debounce_seconds,
        )

        if self._use_watchfiles:
            # Use watchfiles (fast, Rust-based)
            self._watch_thread = threading.Thread(target=self._watchfiles_loop, daemon=True)
            self._watch_thread.start()
        elif WATCHDOG_AVAILABLE:
            # Fallback to watchdog
            self._handler = _WatchdogEventHandler(
                on_change=self._on_file_change,
                exclude_dirs=self.EXCLUDE_DIRS,
                watch_extensions=self.WATCH_EXTENSIONS,
            )
            self._observer = Observer()
            for watch_path in self.config.watch_paths:
                path = Path(watch_path)
                if path.exists():
                    self._observer.schedule(self._handler, str(path), recursive=True)
                    _log.debug("Watching: %s", path)
            self._observer.start()
        else:
            raise ImportError("No file watcher available. Install watchfiles or watchdog")

    def stop(self) -> None:
        """Stop the watchdog trigger."""
        _log.info("Stopping watchdog trigger")
        self._running = False
        self._stop_event.set()

        if self._debounce_timer:
            self._debounce_timer.cancel()

        if self._use_watchfiles:
            # watchfiles loop will exit when _running is False
            if self._watch_thread:
                self._watch_thread.join(timeout=2.0)
        elif self._observer is not None:
            self._observer.stop()
            self._observer.join()

    def run(self, force: bool = False) -> Any:
        """Run a single governance cycle (for watchdog, triggers immediately)."""
        _log.info("Watchdog trigger: running one cycle (force=%s)", force)
        return self.loop.run_once(force=force)

    def _watchfiles_loop(self) -> None:
        """Watch files using watchfiles (fast Rust-based backend)."""
        try:
            # Collect all watch paths
            watch_paths = [Path(p) for p in self.config.watch_paths if Path(p).exists()]
            if not watch_paths:
                _log.warning("No valid watch paths found")
                return

            # Create filter function for watchfiles
            def watch_filter(change: Change, path_str: str) -> bool:
                """Filter changes to only process relevant files."""
                return self._should_process(path_str)

            # watchfiles can watch multiple paths (pass as separate args)
            # Use stop_event to allow graceful shutdown
            for changes in watch(
                *watch_paths,
                recursive=True,
                stop_event=self._stop_event,
                watch_filter=watch_filter,
            ):
                if not self._running:
                    break

                # Process changes (already filtered by watch_filter)
                if changes:
                    change_count = len(changes)
                    _log.debug("Detected %d file change(s)", change_count)
                    self._on_file_change()
        except Exception:
            _log.exception("Error in watchfiles loop")

    def _should_process(self, path_str: str) -> bool:
        """Check if the file change should be processed."""
        path = Path(path_str)

        # Skip directories
        if path.is_dir():
            return False

        # Check extension
        ext = path.suffix.lower()
        if ext not in self.WATCH_EXTENSIONS:
            return False

        # Check if any parent directory is excluded
        parts = path.parts
        if any(part in self.EXCLUDE_DIRS for part in parts):
            return False

        return True

    def _on_file_change(self) -> None:
        """Called when a watched file changes."""
        if not self._running:
            return

        current_time = time.time()
        time_since_last = current_time - self._last_trigger_time

        # Cancel any pending debounce timer
        if self._debounce_timer:
            self._debounce_timer.cancel()

        # If enough time has passed since last trigger, trigger immediately
        if time_since_last >= self.config.debounce_seconds:
            self._trigger_cycle()
        else:
            # Schedule debounced trigger
            remaining = self.config.debounce_seconds - time_since_last
            _log.debug(
                "Changes detected, debouncing for %.1fs",
                remaining,
            )
            self._debounce_timer = threading.Timer(
                remaining,
                self._trigger_cycle,
            )
            self._debounce_timer.start()

    def _trigger_cycle(self) -> None:
        """Trigger a governance cycle."""
        self._last_trigger_time = time.time()
        _log.info("Watchdog triggering governance cycle")

        try:
            result = self.loop.run_once()
            _log.info(
                "Cycle completed: score=%.2f, tasks=%d",
                result.health_score,
                result.tasks_executed,
            )
        except Exception:
            _log.exception("Error running governance cycle")


# Watchdog fallback event handler (only used if watchfiles not available)
if not WATCHFILES_AVAILABLE:

    class _WatchdogEventHandler(FileSystemEventHandler):
        """Event handler for watchdog file system events (fallback).

        Filters events to only process relevant file changes.
        Only used when watchfiles is not available.
        """

        def __init__(
            self,
            on_change: Any,
            exclude_dirs: frozenset[str],
            watch_extensions: frozenset[str],
        ) -> None:
            self._on_change = on_change
            self._exclude_dirs = exclude_dirs
            self._watch_extensions = watch_extensions

        def _should_process(self, path: str) -> bool:
            """Check if the event should be processed."""
            # Skip directories
            if Path(path).is_dir():
                return False

            # Check extension
            ext = Path(path).suffix.lower()
            if ext not in self._watch_extensions:
                return False

            # Check if any parent directory is excluded
            parts = Path(path).parts
            if any(part in self._exclude_dirs for part in parts):
                return False

            return True

        def on_modified(self, event: Any) -> None:
            """Called when a file is modified."""
            if event.is_directory:
                return
            if self._should_process(event.src_path):
                _log.debug("File modified: %s", event.src_path)
                self._on_change()

        def on_created(self, event: Any) -> None:
            """Called when a file is created."""
            if event.is_directory:
                return
            if self._should_process(event.src_path):
                _log.debug("File created: %s", event.src_path)
                self._on_change()

        def on_deleted(self, event: Any) -> None:
            """Called when a file is deleted."""
            if event.is_directory:
                return
            if self._should_process(event.src_path):
                _log.debug("File deleted: %s", event.src_path)
                self._on_change()


class TimerTrigger:
    """Periodic timer-based trigger.

    Triggers governance cycles at fixed intervals.
    """

    def __init__(
        self,
        loop: Any,
        config: TriggerConfig,
    ) -> None:
        self.loop = loop
        self.config = config
        self._running = False
        self._stop_event = threading.Event()

    def start(self) -> None:
        """Start the timer trigger."""
        self._running = True
        _log.info(
            "Starting timer trigger: interval=%ds",
            self.config.interval_seconds,
        )

        self._timer_thread = threading.Thread(target=self._timer_loop, daemon=True)
        self._timer_thread.start()

    def stop(self) -> None:
        """Stop the timer trigger."""
        _log.info("Stopping timer trigger")
        self._running = False
        self._stop_event.set()

    def run(self, force: bool = False) -> Any:
        """Run a single governance cycle (for timer, triggers immediately)."""
        _log.info("Timer trigger: running one cycle (force=%s)", force)
        return self.loop.run_once(force=force)

    def _timer_loop(self) -> None:
        """Run timer loop."""
        cycles_run = 0

        while self._running and not self._stop_event.is_set():
            try:
                # Check if we've hit max cycles
                if self.config.max_cycles and cycles_run >= self.config.max_cycles:
                    _log.info("Reached max_cycles=%d, stopping", self.config.max_cycles)
                    break

                _log.debug("Timer triggering governance cycle")

                result = self.loop.run_once()
                cycles_run += 1

                _log.info(
                    "Cycle %d completed: score=%.2f, tasks=%d",
                    cycles_run,
                    result.health_score,
                    result.tasks_executed,
                )

            except Exception:
                _log.exception("Error running governance cycle")

            # Sleep in small increments to allow quick shutdown
            for _ in range(self.config.interval_seconds):
                if not self._running:
                    break
                time.sleep(1)


class ManualTrigger:
    """One-shot manual trigger.

    Runs a single governance cycle and exits.
    """

    def __init__(
        self,
        loop: Any,
    ) -> None:
        self.loop = loop

    def start(self) -> None:
        """Start the trigger (no-op for manual mode)."""

    def stop(self) -> None:
        """Stop the trigger (no-op for manual mode)."""

    def run(self, force: bool = False) -> Any:
        """Run a single governance cycle."""
        _log.info("Manual trigger: running one cycle (force=%s)", force)
        return self.loop.run_once(force=force)


class HealthThresholdTrigger:
    """Trigger governance cycle when health drops below threshold."""

    def __init__(
        self,
        loop: Any,
        threshold: float = 90.0,
        check_interval: int = 60,
    ) -> None:
        self.loop = loop
        self.threshold = threshold
        self.check_interval = check_interval
        self._shutdown = False
        self._thread: threading.Thread | None = None

    def start(self) -> None:
        """Start health monitoring in background."""
        self._shutdown = False

        def monitor() -> None:
            while not self._shutdown:
                try:
                    result = self.loop.run_once()
                    if result.health_score < self.threshold:
                        _log.info(
                            "Health %.2f < threshold %.2f, triggering full cycle",
                            result.health_score,
                            self.threshold,
                        )
                        self.loop.run_once(force=True)
                except Exception:
                    _log.exception("Health monitor error")
                for _ in range(self.check_interval):
                    if self._shutdown:
                        return
                    time.sleep(1)

        self._thread = threading.Thread(target=monitor, daemon=True)
        self._thread.start()
        _log.info(
            "Health threshold trigger started: threshold=%.1f, interval=%ds",
            self.threshold,
            self.check_interval,
        )

    def stop(self) -> None:
        """Stop health monitoring."""
        self._shutdown = True

    def run(self, force: bool = False) -> Any:
        """Run a single governance cycle (for health threshold, triggers immediately)."""
        _log.info("Health threshold trigger: running one cycle (force=%s)", force)
        return self.loop.run_once(force=force)


def create_trigger(
    mode: str,
    loop: Any,
    config: TriggerConfig,
    *,
    watch_health_threshold: float = 90.0,
    watch_health_interval: int = 60,
) -> TriggerProtocol:
    """Factory function to create the appropriate trigger."""

    if mode == "watchdog":
        return WatchdogTrigger(loop, config)
    if mode == "timer":
        return TimerTrigger(loop, config)
    if mode == "manual":
        return ManualTrigger(loop)
    if mode == "watch-health":
        return HealthThresholdTrigger(
            loop,
            threshold=watch_health_threshold,
            check_interval=watch_health_interval,
        )
    raise ValueError(f"Unknown trigger mode: {mode}")


def main(
    *,
    mode: str = "manual",
    interval: int = 300,
    debounce: int = DEFAULT_DEBOUNCE_SECONDS,
    max_cycles: int | None = None,
    force: bool = False,
    watch: list[str] | None = None,
    project_dir: Path = Path.cwd(),
    health_targets: Path | None = None,
    threshold: float = 90.0,
    lifecycle_mode: str = "soft",
    watch_health: float = 90.0,
    watch_health_interval: int = 60,
) -> int:
    """CLI entry point for triggers."""

    # Setup logging
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s %(message)s",
    )

    # Find health-targets.json
    if health_targets:
        health_targets_path = health_targets
    else:
        # Look in standard locations
        candidates = [
            project_dir / "contracts" / "health-targets.json",
            project_dir.parent / "contracts" / "health-targets.json",
        ]
        health_targets_path = None
        for c in candidates:
            if c.exists():
                health_targets_path = c
                break

        if health_targets_path is None:
            _log.error("health-targets.json not found")
            return 1

    # Create the loop
    from thegent.governance.agileplus import AgilePlusLoop

    loop = AgilePlusLoop(
        project_dir=project_dir,
        health_targets_path=health_targets_path,
        health_threshold=threshold,
        lifecycle_mode=lifecycle_mode,
    )

    # Create config
    config = TriggerConfig(
        mode=mode,
        interval_seconds=interval,
        debounce_seconds=debounce,
        watch_paths=watch or ["src/", "tests/", "hooks/"],
        max_cycles=max_cycles,
        health_threshold=threshold,
    )

    # Create and run trigger
    trigger = create_trigger(
        mode,
        loop,
        config,
        watch_health_threshold=watch_health,
        watch_health_interval=watch_health_interval,
    )

    # Handle signals
    def shutdown(signum: int, frame: Any) -> None:
        _log.info("Shutdown signal received")
        if hasattr(trigger, "stop"):
            trigger.stop()
        sys.exit(0)

    signal.signal(signal.SIGINT, shutdown)
    signal.signal(signal.SIGTERM, shutdown)

    if mode == "manual":
        result = trigger.run(force=force)
        _log.info("\nCycle Result:")
        _log.info("  State: %s", result.state)
        _log.info("  Health Score: %.2f", result.health_score)
        _log.info("  Health Band: %s", result.health_band)
        _log.info("  Tasks Executed: %d", result.tasks_executed)
        _log.info("  Tasks Verified: %d", result.tasks_verified)
        _log.info("  Error: %s", result.error or "none")

        if result.error:
            return 1
    else:
        trigger.start()
        while True:
            time.sleep(60)

    return 0


@app.callback(invoke_without_command=True)
def cli(
    mode: str = typer.Option("manual", "--mode", help="Trigger mode"),
    interval: int = typer.Option(300, "--interval", help="Interval in seconds for timer mode"),
    debounce: int = typer.Option(DEFAULT_DEBOUNCE_SECONDS, "--debounce", help="Debounce seconds for watchdog mode"),
    max_cycles: int | None = typer.Option(None, "--max-cycles", help="Maximum cycles to run (timer/watchdog only)"),
    force: bool = typer.Option(False, "--force", help="Run even if health >= threshold"),
    watch: list[str] | None = typer.Option(None, "--watch", help="Paths to watch (repeat --watch for multiple paths)"),
    project_dir: Path = typer.Option(Path.cwd(), "--project-dir", help="Project directory"),
    health_targets: Path | None = typer.Option(None, "--health-targets", help="Path to health-targets.json"),
    threshold: float = typer.Option(90.0, "--threshold", help="Health threshold"),
    lifecycle_mode: str = typer.Option("soft", "--lifecycle-mode", help="Lifecycle execution mode: soft or hard"),
    watch_health: float = typer.Option(
        90.0, "--watch-health", help="Trigger cycle when health drops below this threshold"
    ),
    watch_health_interval: int = typer.Option(60, "--watch-health-interval", help="Health check interval in seconds"),
) -> None:
    """Run AgilePlus trigger modes."""
    if mode not in {"watchdog", "timer", "manual", "watch-health"}:
        raise typer.BadParameter("mode must be one of: watchdog, timer, manual, watch-health")
    if lifecycle_mode not in {"soft", "hard"}:
        raise typer.BadParameter("lifecycle-mode must be one of: soft, hard")

    code = main(
        mode=mode,
        interval=interval,
        debounce=debounce,
        max_cycles=max_cycles,
        force=force,
        watch=watch,
        project_dir=project_dir,
        health_targets=health_targets,
        threshold=threshold,
        lifecycle_mode=lifecycle_mode,
        watch_health=watch_health,
        watch_health_interval=watch_health_interval,
    )
    raise typer.Exit(code=code)


if __name__ == "__main__":
    app()
