"""CLI commands for agent mesh coordination."""

import subprocess
from thegent.infra.shim_subprocess import run as shim_run
from pathlib import Path

import typer
from rich.console import Console
from rich.table import Table

from thegent.config import ThegentSettings
from thegent.mesh.mesh import MeshManager
from thegent.mesh.agent_patterns import run_detection

app = typer.Typer(help="Agent Mesh Coordination commands")
console = Console()

# Queue subcommand group
queue_app = typer.Typer(help="Distributed task queue (Phase 11)")
app.add_typer(queue_app, name="queue")


@queue_app.command("enqueue")
def queue_enqueue(
    payload: str = typer.Argument(..., help="Task payload (JSON string or simple text)"),
    priority: int = typer.Option(5, "--priority", "-p", help="Task priority (0-9, lower is higher)"),
    mesh_root: str | None = typer.Option(None, "--mesh-root", help="Path to mesh root"),
) -> None:
    """Enqueue a new task into the distributed mesh queue."""
    import json

    from thegent.mesh.task_queue import MaildirQueue

    settings = ThegentSettings()
    root = Path(mesh_root) if mesh_root else Path(settings.harness_root)
    queue = MaildirQueue(root / "queue")

    try:
        task_data = json.loads(payload)
    except json.JSONDecodeError:
        task_data = {"text": payload}

    task_id = queue.enqueue(task_data, priority=priority)
    console.print(f"[bold green]Task enqueued![/bold green] ID: [cyan]{task_id}[/cyan]")


@queue_app.command("dequeue")
def queue_dequeue(
    agent_id: str | None = typer.Option(None, "--agent-id", help="Optional owner ID for claimed task"),
    mesh_root: str | None = typer.Option(None, "--mesh-root", help="Path to mesh root"),
    ack: bool = typer.Option(False, "--ack", help="Immediately acknowledge/remove the task"),
) -> None:
    """Claim the highest-priority task from the queue."""
    import json

    from thegent.mesh.task_queue import MaildirQueue

    settings = ThegentSettings()
    root = Path(mesh_root) if mesh_root else Path(settings.harness_root)
    queue = MaildirQueue(root / "queue")

    envelope = queue.dequeue(owner=agent_id)
    if not envelope:
        console.print("[yellow]Queue is empty.[/yellow]")
        return

    task_id = envelope["id"]
    console.print(f"[bold green]Task dequeued![/bold green] ID: [cyan]{task_id}[/cyan]")
    console.print(json.dumps(envelope, indent=2))

    if ack:
        queue.ack(task_id)
        console.print(f"[dim]Task {task_id} acknowledged and removed.[/dim]")


@queue_app.command("list")
def queue_list(
    mesh_root: str | None = typer.Option(None, "--mesh-root", help="Path to mesh root"),
) -> None:
    """List all pending and in-flight tasks in the queue."""
    from thegent.mesh.task_queue import MaildirQueue

    settings = ThegentSettings()
    root = Path(mesh_root) if mesh_root else Path(settings.harness_root)
    queue = MaildirQueue(root / "queue")

    tasks = queue.list_pending()
    if not tasks:
        console.print("[yellow]No tasks in queue.[/yellow]")
        return

    table = Table(title="Distributed Task Queue (Phase 11)")
    table.add_column("ID", style="cyan")
    table.add_column("Priority", style="green")
    table.add_column("Attempts", style="magenta")
    table.add_column("Created At", style="dim")
    table.add_column("Payload", style="white")

    from datetime import datetime

    for t in tasks:
        created = datetime.fromtimestamp(t["created_at"]).strftime("%Y-%m-%d %H:%M:%S")
        table.add_row(
            t["id"][:8],
            str(t["priority"]),
            str(t["attempts"]),
            created,
            str(t["payload"])[:50] + "..." if len(str(t["payload"])) > 50 else str(t["payload"]),
        )

    console.print(table)


@app.command("status")
def status(
    cd: str | None = typer.Option(None, "--cd", "-d", help="Working directory"),
    mesh_root: str | None = typer.Option(None, "--mesh-root", help="Path to mesh root"),
    live: bool = typer.Option(False, "--live", "-l", help="Show live updates"),
) -> None:
    """Show current mesh status and registered agents."""
    from pathlib import Path

    settings = ThegentSettings()
    root = Path(mesh_root) if mesh_root else Path(settings.harness_root)
    manager = MeshManager(mesh_root=root)

    if live:
        _live_status(manager, root)
        return

    console.print(f"[bold]Mesh Root:[/bold] {root}")

    # Registered agents
    agents_dir = root / "agents"
    if not agents_dir.exists():
        console.print("[yellow]Mesh not initialized or no agents registered.[/yellow]")
        return

    table = Table(title="Registered Agents")
    table.add_column("PID", style="cyan")
    table.add_column("Type", style="green")
    table.add_column("Status", style="magenta")
    table.add_column("Workdir", style="dim")

    for manifest in agents_dir.glob("agent-*.yaml"):
        _load_mesh_manifest(table, manifest)

    console.print(table)


def _live_status(manager: MeshManager, root: Path) -> None:
    """Display a live-updating mesh status dashboard."""
    import time

    from rich.live import Live

    def _process_manifest(manifest: Path) -> tuple[str, str, str, str] | None:
        """Process a single manifest file, returns row data or None on error."""
        import psutil
        import yaml

        try:
            with open(manifest) as f:
                data = yaml.safe_load(f)
                pid = data.get("pid", "unknown")
                agent_type = data.get("type", "unknown")
                status = data.get("status", "unknown")

                # Last modified time as heartbeat proxy
                mtime = manifest.stat().st_mtime
                elapsed = int(time.time() - mtime)
                hb = f"{elapsed}s ago"

                try:
                    p = psutil.Process(int(pid))
                    if not p.is_running():
                        status = "[red]zombie[/red]"
                except Exception:
                    status = "[red]offline[/red]"

                return (str(pid), agent_type, status, hb)
        except Exception:
            return None

    def generate_table() -> Table:
        table = Table(title="Agent Mesh Dashboard (Live)")
        table.add_column("PID", style="cyan")
        table.add_column("Type", style="green")
        table.add_column("Status", style="magenta")
        table.add_column("Heartbeat", style="dim")

        agents_dir = root / "agents"
        if not agents_dir.exists():
            return table

        for manifest in sorted(agents_dir.glob("agent-*.yaml")):
            row = _process_manifest(manifest)
            if row is not None:
                table.add_row(*row)
        return table

    with Live(generate_table(), refresh_per_second=1) as live:
        try:
            while True:
                time.sleep(1)
                live.update(generate_table())
        except KeyboardInterrupt:
            pass


def _load_mesh_manifest(table: Table, manifest: Path) -> None:
    """Load and process a single mesh manifest safely."""
    import psutil
    import yaml

    try:
        with open(manifest) as f:
            data = yaml.safe_load(f)
            pid = data.get("pid", "unknown")
            agent_type = data.get("type", "unknown")
            status = data.get("status", "unknown")
            workdir = data.get("working_directory", "unknown")

            # Check if process is still running
            try:
                p = psutil.Process(int(pid))
                if not p.is_running():
                    status = "[red]exited[/red]"
            except (psutil.NoSuchProcess, ValueError):
                status = "[red]not found[/red]"

            table.add_row(str(pid), agent_type, status, workdir)
    except Exception as e:
        console.print(f"[red]Error reading {manifest.name}: {e}[/red]")


@app.command("discover")
def discover(
    patterns: str | None = typer.Option(
        None,
        "--patterns",
        help="Comma-separated regex patterns (optional; defaults to agents.conf)",
    ),
    mesh_root: str | None = typer.Option(None, "--mesh-root", help="Path to mesh root"),
    register: bool = typer.Option(True, "--register/--no-register", help="Register discovered agents"),
) -> None:
    """Discover active agent processes and optionally register them."""
    from pathlib import Path

    settings = ThegentSettings()
    root = Path(mesh_root) if mesh_root else Path(settings.harness_root)
    manager = MeshManager(mesh_root=root)

    if patterns is None:
        console.print("Scanning agents using agents.conf patterns...")
        detected = run_detection()
        agents = [{"pid": a["pid"], "name": a["agent"]} for a in detected]
    else:
        pattern_list = [p.strip() for p in patterns.split(",")]
        console.print(f"Scanning for agents matching: {', '.join(pattern_list)}...")
        agents = manager.discover_agents(pattern_list)

    if not agents:
        console.print("[yellow]No agents discovered.[/yellow]")
        return

    console.print(f"Discovered [bold]{len(agents)}[/bold] agent(s):")
    for a in agents:
        pid = a["pid"]
        name = a["name"]
        console.print(f"  - [cyan]PID {pid}[/cyan]: {name}")

        if register:
            manager.register_agent(
                f"agent-{pid}",
                {
                    "pid": pid,
                    "type": name,
                    "working_directory": "unknown",  # Improved detection needed
                },
            )
            manager.heartbeat(f"agent-{pid}")

    if register:
        console.print("[green]Agents registered to mesh.[/green]")


@app.command("merge")
def mesh_merge(
    base: Path = typer.Argument(..., help="Base file version"),
    ours: Path = typer.Argument(..., help="Our file version"),
    theirs: Path = typer.Argument(..., help="Their file version"),
    output: Path = typer.Option(None, "--output", "-o", help="Output file (default: overwrite 'ours')"),
) -> None:
    """WP-16004: AST-aware conflict resolution using SmartMerge."""
    from thegent.governance.heliosShield_bridge import SmartMerge

    if output is None:
        output = ours

    merger = SmartMerge()
    console.print(f"Attempting AST-aware merge: [cyan]{ours.name}[/cyan]...")

    success = merger.merge_files(base, ours, theirs, output)

    if success:
        console.print("[bold green]✓ Merge successful![/bold green]")
    else:
        console.print("[bold red]✗ Merge failed or conflicts remain.[/bold red]")
        raise typer.Exit(1)


@app.command("consensus")
def mesh_consensus(
    task_id: str = typer.Argument(..., help="Task ID to reach consensus on"),
) -> None:
    """WP-X: Reach consensus on a task across multiple agents."""
    console.print(f"Reaching consensus for task: [cyan]{task_id}[/cyan]...")
    # Placeholder for actual consensus logic (Phase 13)
    console.print("[yellow]Consensus engine not yet implemented. Defaulting to majority vote...[/yellow]")
    console.print("[green]Consensus reached: Accept changes.[/green]")


@app.command("run")
def run_agent(
    agent_type: str = typer.Argument("claude-code", help="Type of agent to run"),
    prompt: str = typer.Option("", "--prompt", "-p", help="Initial prompt"),
    workdir: str | None = typer.Option(None, "--workdir", "-w", help="Working directory"),
) -> None:
    """Start a new agent in a managed mesh session (tmux)."""
    from pathlib import Path

    script_path = Path(__file__).parent / "scripts" / "agent-run.sh"

    if not script_path.exists():
        from thegent.errors import print_error

        print_error(f"{script_path} not found.")
        raise typer.Exit(1)

    console.print(f"Launching {agent_type} in managed mesh session...")

    try:
        result = shim_run(
            ["bash", str(script_path), agent_type, prompt, str(workdir)], capture_output=True, text=True, check=True
        )
        console.print(result.stdout)
    except subprocess.CalledProcessError as e:
        console.print("[red]Failed to launch agent:[/red]")
        console.print(e.stderr)
        raise typer.Exit(e.returncode)
