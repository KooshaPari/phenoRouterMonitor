"""Smart merge and conflict resolution for the agent mesh."""

import subprocess
from thegent.infra.shim_subprocess import run as shim_run
from pathlib import Path


class SmartMerge:
    """Smart merge and conflict resolution (SCLI-P5.1–P5.4)."""

    def __init__(self, mesh_root: Path) -> None:
        self.mesh_root = mesh_root

    def merge_ast_aware(self, base: Path, ours: Path, theirs: Path, output: Path) -> bool:
        """Mergiraf integration (AST-aware merge) (SCLI-P5.1)."""
        # Mergiraf command line: mergiraf merge --base <base> --ours <ours> --theirs <theirs> --output <output>
        try:
            shim_run(
                [
                    "mergiraf",
                    "merge",
                    "--base",
                    str(base),
                    "--ours",
                    str(ours),
                    "--theirs",
                    str(theirs),
                    "--output",
                    str(output),
                ],
                check=True,
                capture_output=True,
            )
            return True
        except (subprocess.CalledProcessError, FileNotFoundError):
            # Fallback to standard 3-way merge if mergiraf fails or missing
            try:
                with output.open("wb") as out_file:
                    shim_run(
                        ["git", "merge-file", "-p", str(ours), str(base), str(theirs)],
                        check=True,
                        stdout=out_file,
                    )
                return True
            except subprocess.CalledProcessError:
                return False

    def predict_conflicts(self, agent_intents: list[dict]) -> list[tuple[str, str, str]]:
        """Conflict prediction before commit (trial merge from intents) (SCLI-P5.2)."""
        # Intent format: {"agent_id": "...", "files": ["...", "..."], "type": "write"}
        conflicts = []
        files_to_agents = {}

        for intent in agent_intents:
            agent_id = intent["agent_id"]
            for file in intent["files"]:
                if file in files_to_agents:
                    # Potential conflict
                    other_agent = files_to_agents[file]
                    conflicts.append((file, agent_id, other_agent))
                files_to_agents[file] = agent_id

        return conflicts

    def resolve_imports(self, content_a: str, content_b: str, language: str = "python") -> str:
        """Import union auto-resolution (SCLI-P5.3)."""
        # Simple regex-based import union for Python/JS
        import_lines = set()
        other_lines = []

        for content in [content_a, content_b]:
            for line in content.splitlines():
                line = line.strip()
                if language == "python":
                    if line.startswith(("import ", "from ")):
                        import_lines.add(line)
                    else:
                        other_lines.append(line)
                elif language in ("javascript", "typescript"):
                    if line.startswith(("import ", "from ", "require(")):
                        import_lines.add(line)
                    else:
                        other_lines.append(line)

        sorted_imports = sorted(import_lines)
        return "\n".join([*sorted_imports, "", *other_lines])

    def merge_structural(self, path_a: Path, path_b: Path, output: Path) -> bool:
        """JSON/YAML structural merge (SCLI-P5.4)."""
        # Use jq for JSON merge, custom logic for YAML
        ext = path_a.suffix.lower()
        if ext == ".json":
            try:
                with output.open("wb") as out_file:
                    shim_run(
                        ["jq", "-s", ".[0] * .[1]", str(path_a), str(path_b)],
                        check=True,
                        stdout=out_file,
                    )
                return True
            except (subprocess.CalledProcessError, FileNotFoundError):
                return False
        elif ext in (".yaml", ".yml"):
            # Deep merge YAML manually using ruamel.yaml or similar
            try:
                import ruamel.yaml

                yaml = ruamel.yaml.YAML()
                with open(path_a) as f:
                    data_a = yaml.load(f)
                with open(path_b) as f:
                    data_b = yaml.load(f)

                # Simple deep merge logic
                merged = self._deep_merge(data_a, data_b)
                with open(output, "w") as f:
                    yaml.dump(merged, f)
                return True
            except ImportError:
                return False
        return False

    def _deep_merge(self, a: dict, b: dict) -> dict:
        """Deep merge two dictionaries."""
        for key, value in b.items():
            if key in a:
                if isinstance(a[key], dict) and isinstance(value, dict):
                    self._deep_merge(a[key], value)
                else:
                    a[key] = value
            else:
                a[key] = value
        return a
