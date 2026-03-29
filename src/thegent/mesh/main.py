"""heliosShield: High-performance CLI for agent mesh orchestration."""

from pathlib import Path

import typer

from thegent.mesh.mesh import MeshManager
from thegent.mesh.agent_patterns import run_detection
from thegent.mesh.observability import mesh_status_cmd

app = typer.Typer(help="Mesh: Local agent mesh coordination (init, status, discover).")


@app.command("status")
def status(mesh_root: Path | None = typer.Option(None, "--mesh-root", help="Path to mesh root")):
    """Show current mesh status."""
    from thegent.config import ThegentSettings

    root = mesh_root or Path(ThegentSettings().harness_root)
    mesh_status_cmd(root)


@app.command("agents")
def agents(mesh_root: Path | None = typer.Option(None, "--mesh-root", help="Path to mesh root")):
    """List registered mesh agents."""
    from thegent.config import ThegentSettings

    root = mesh_root or Path(ThegentSettings().harness_root)
    mesh_status_cmd(root)


@app.command("init")
def init(mesh_root: Path | None = typer.Option(None, "--mesh-root", help="Path to mesh root")):
    """Initialize agent mesh."""
    from thegent.config import ThegentSettings

    root = mesh_root or Path(ThegentSettings().harness_root)
    MeshManager(root)


@app.command("discover")
def discover(
    patterns: str | None = typer.Option(
        None,
        "--patterns",
        help="Comma-separated regex patterns (optional; defaults to agents.conf)",
    ),
    mesh_root: Path | None = typer.Option(None, "--mesh-root", help="Path to mesh root"),
):
    """Discover active agents and register them."""
    from thegent.config import ThegentSettings

    root = mesh_root or Path(ThegentSettings().harness_root)
    mesh = MeshManager(root)
    if patterns is None:
        discovered = run_detection()
        agents = [{"pid": a["pid"], "name": a["agent"]} for a in discovered]
    else:
        pattern_list = [p.strip() for p in patterns.split(",")]
        agents = mesh.discover_agents(pattern_list)

    registered_ids: list[str] = []
    for agent in agents:
        pid = agent.get("pid", "unknown")
        name = str(agent.get("name", "agent"))
        agent_id = f"{name}-{pid}"
        mesh.register_agent(
            agent_id,
            {
                "pid": pid,
                "name": name,
                "source": "auto-detect" if patterns is None else "pattern-filter",
            },
        )
        registered_ids.append(agent_id)

    typer.echo(f"Discovered {len(registered_ids)} agents.")
    for agent_id in registered_ids:
        typer.echo(f"- {agent_id}")


if __name__ == "__main__":
    app()
