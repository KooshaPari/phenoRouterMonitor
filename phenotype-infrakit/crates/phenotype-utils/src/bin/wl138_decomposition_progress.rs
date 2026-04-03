//! Emit WL-138 decomposition progress as machine-readable JSON.

use std::{path::PathBuf, process::Command, time::Instant};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "wl138-decomposition-progress")]
struct Args {
    #[arg(
        long,
        default_value = "docs/reports/artifacts/wl138_decomposition_progress.json"
    )]
    output: PathBuf,

    #[arg(long)]
    skip_execution_gates: bool,

    #[arg(long)]
    checkpoint: Option<PathBuf>,

    #[arg(long, default_value = ".")]
    repo_root: PathBuf,

    #[arg(long, default_value = "python3")]
    python_bin: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Checkpoint {
    checkpoint_id: String,
    description: String,
    paths: Vec<String>,
    #[serde(default)]
    execution_gates: Vec<ExecutionGate>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ExecutionGate {
    gate_id: String,
    description: String,
    command: Vec<String>,
}

#[derive(Serialize)]
struct CheckRecord {
    path: String,
    exists: bool,
}

#[derive(Serialize)]
struct ExecutionGateResult {
    gate_id: String,
    description: String,
    command: String,
    status: String,
    exit_code: Option<i32>,
    duration_ms: f64,
    stdout_tail: Vec<String>,
    stderr_tail: Vec<String>,
}

#[derive(Serialize)]
struct CheckpointResult {
    checkpoint_id: String,
    description: String,
    complete: bool,
    checks: Vec<CheckRecord>,
    execution_gates: Vec<ExecutionGateResult>,
    evaluation: CheckpointEvaluation,
}

#[derive(Serialize)]
struct CheckpointEvaluation {
    paths_complete: bool,
    execution_gates_complete: bool,
    passed_execution_gates: usize,
    total_execution_gates: usize,
    execution_gates_skipped: bool,
}

#[derive(Serialize)]
struct Payload {
    workstream_id: &'static str,
    artifact_id: &'static str,
    generated_at_utc: DateTime<Utc>,
    complete_checkpoints: usize,
    total_checkpoints: usize,
    completion_pct: f64,
    execution_gates: ExecutionGatesSummary,
    checkpoints: Vec<CheckpointResult>,
}

#[derive(Serialize)]
struct ExecutionGatesSummary {
    complete: usize,
    total: usize,
    completion_pct: f64,
    skipped: bool,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    let repo_root = args.repo_root;

    if let Some(checkpoint_path) = args.checkpoint {
        let single = load_single_checkpoint(&checkpoint_path)?;
        let result = checkpoint_result(
            &repo_root,
            &single,
            args.skip_execution_gates,
            &args.python_bin,
        );
        write_json(&args.output, &result)?;
        println!("wrote WL-138 progress artifact: {}", args.output.display());
        println!(
            "completion: {}/{} ({}%)",
            usize::from(result.complete),
            1,
            if result.complete { 100.0 } else { 0.0 }
        );
        println!(
            "execution gates: {}/{} ({}%)",
            result.evaluation.passed_execution_gates,
            result.evaluation.total_execution_gates,
            percent(
                result.evaluation.passed_execution_gates,
                result.evaluation.total_execution_gates,
            )
        );
        return Ok(());
    }

    let checkpoints = build_checkpoints();
    let results = checkpoints
        .into_iter()
        .map(|item| {
            checkpoint_result(
                &repo_root,
                &item,
                args.skip_execution_gates,
                &args.python_bin,
            )
        })
        .collect::<Vec<_>>();
    let payload = Payload::for_all(results);
    write_json(&args.output, &payload)?;
    println!("wrote WL-138 progress artifact: {}", args.output.display());
    println!(
        "completion: {}/{} ({}%)",
        payload.complete_checkpoints, payload.total_checkpoints, payload.completion_pct
    );
    println!(
        "execution gates: {}/{} ({}%)",
        payload.execution_gates.complete,
        payload.execution_gates.total,
        payload.execution_gates.completion_pct
    );

    Ok(())
}

impl Payload {
    fn for_all(checkpoints: Vec<CheckpointResult>) -> Self {
        let complete_count = checkpoints.iter().filter(|cp| cp.complete).count();
        let total = checkpoints.len();
        let complete_execution_gates = checkpoints
            .iter()
            .map(|cp| cp.evaluation.passed_execution_gates)
            .sum();
        let total_execution_gates = checkpoints
            .iter()
            .map(|cp| cp.evaluation.total_execution_gates)
            .sum();
        let skipped = if total == 0 {
            false
        } else {
            checkpoints[0].evaluation.execution_gates_skipped
        };

        Payload {
            workstream_id: "WL-138",
            artifact_id: "wl138.decomposition_progress.v1",
            generated_at_utc: Utc::now(),
            complete_checkpoints: complete_count,
            total_checkpoints: total,
            completion_pct: percent(complete_count, total),
            execution_gates: ExecutionGatesSummary {
                complete: complete_execution_gates,
                total: total_execution_gates,
                completion_pct: percent(complete_execution_gates, total_execution_gates),
                skipped,
            },
            checkpoints,
        }
    }
}

fn percent(n: usize, d: usize) -> f64 {
    if d == 0 {
        0.0
    } else {
        ((n as f64) / (d as f64) * 100.0 * 100.0).round() / 100.0
    }
}

fn write_json<T: Serialize>(path: &PathBuf, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }
    let content = serde_json::to_string_pretty(value)?;
    std::fs::write(path, format!("{content}\n"))
        .with_context(|| format!("failed to write output artifact to {}", path.display()))?;
    Ok(())
}

fn load_single_checkpoint(path: &PathBuf) -> Result<Checkpoint> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read checkpoint fixture {}", path.display()))?;
    let checkpoint = serde_json::from_str::<Checkpoint>(&raw)
        .with_context(|| format!("failed to parse checkpoint JSON {}", path.display()))?;
    Ok(checkpoint)
}

fn checkpoint_result(
    root: &PathBuf,
    item: &Checkpoint,
    skip_execution_gates: bool,
    python_bin: &str,
) -> CheckpointResult {
    let mut checks = Vec::with_capacity(item.paths.len());
    let mut paths_complete = true;
    for rel_path in &item.paths {
        let path = root.join(rel_path);
        let exists = path.exists();
        if !exists {
            paths_complete = false;
        }
        checks.push(CheckRecord {
            path: rel_path.clone(),
            exists,
        });
    }

    let mut gate_results = Vec::with_capacity(item.execution_gates.len());
    let mut passed = 0usize;

    for gate in item.execution_gates.iter().cloned() {
        let command = normalize_command(gate.clone(), python_bin);
        let result = if skip_execution_gates {
            ExecutionGateResult {
                gate_id: gate.gate_id,
                description: gate.description,
                command: command.join(" "),
                status: "skipped".to_string(),
                exit_code: None,
                duration_ms: 0.0,
                stdout_tail: Vec::new(),
                stderr_tail: Vec::new(),
            }
        } else {
            let (status, exit_code, duration_ms, stdout_tail, stderr_tail) =
                execute_command(root, &command);
            if status == "pass" {
                passed += 1;
            }
            ExecutionGateResult {
                gate_id: gate.gate_id,
                description: gate.description,
                command: command.join(" "),
                status,
                exit_code,
                duration_ms,
                stdout_tail,
                stderr_tail,
            }
        };
        gate_results.push(result);
    }

    let total = gate_results.len();
    let gates_complete = if total == 0 {
        true
    } else {
        passed == total && !skip_execution_gates
    };

    CheckpointResult {
        checkpoint_id: item.checkpoint_id.clone(),
        description: item.description.clone(),
        complete: paths_complete && gates_complete,
        checks,
        execution_gates: gate_results,
        evaluation: CheckpointEvaluation {
            paths_complete,
            execution_gates_complete: gates_complete,
            passed_execution_gates: passed,
            total_execution_gates: total,
            execution_gates_skipped: skip_execution_gates,
        },
    }
}

fn normalize_command(gate: ExecutionGate, python_bin: &str) -> Vec<String> {
    let mut command = gate.command;
    if command.first().is_some_and(|first| first == "${PYTHON}") {
        command[0] = python_bin.to_string();
    }
    command
}

fn execute_command(
    root: &PathBuf,
    command: &[String],
) -> (String, Option<i32>, f64, Vec<String>, Vec<String>) {
    if command.is_empty() {
        return (
            "fail".to_string(),
            Some(127),
            0.0,
            Vec::new(),
            vec!["empty command".to_string()],
        );
    }

    let start = Instant::now();
    let output = Command::new(&command[0])
        .args(&command[1..])
        .current_dir(root)
        .output();
    let elapsed_ms = (Instant::now() - start).as_secs_f64() * 1000.0;

    match output {
        Ok(completed) => {
            let status = if completed.status.success() {
                "pass"
            } else {
                "fail"
            }
            .to_string();

            let exit_code = completed.status.code();
            let stdout_tail = tail_lines(String::from_utf8_lossy(&completed.stdout).to_string());
            let stderr_tail = tail_lines(String::from_utf8_lossy(&completed.stderr).to_string());
            (status, exit_code, elapsed_ms, stdout_tail, stderr_tail)
        }
        Err(err) => (
            "fail".to_string(),
            Some(1),
            elapsed_ms,
            Vec::new(),
            vec![err.to_string()],
        ),
    }
}

fn tail_lines(data: String) -> Vec<String> {
    let mut lines = data.lines().map(ToOwned::to_owned).collect::<Vec<_>>();
    if lines.len() > 5 {
        lines.drain(0..(lines.len() - 5));
    }
    lines
}

fn build_checkpoints() -> Vec<Checkpoint> {
    vec![
        Checkpoint {
            checkpoint_id: "python-monolith-cuts".to_string(),
            description: "Python command/server decomposition scaffolding exists".to_string(),
            paths: vec![
                "src/thegent/cli/commands/helpers.py".to_string(),
                "src/thegent/mcp/server_runtime_helpers.py".to_string(),
            ],
            execution_gates: Vec::new(),
        },
        Checkpoint {
            checkpoint_id: "rust-hook-splits".to_string(),
            description: "Rust hook dispatcher decomposition folders exist and execute via Rust tests".to_string(),
            paths: vec![
                "hooks/hook-dispatcher/src/dispatch".to_string(),
                "hooks/hook-dispatcher/src/contract".to_string(),
                "hooks/hook-dispatcher/src/io".to_string(),
            ],
            execution_gates: vec![
                ExecutionGate {
                    gate_id: "rust-hook-dispatcher-tests".to_string(),
                    description: "hook-dispatcher decomposition modules compile + tests execute".to_string(),
                    command: vec![
                        "cargo".to_string(),
                        "test".to_string(),
                        "-q".to_string(),
                        "--manifest-path".to_string(),
                        "hooks/hook-dispatcher/Cargo.toml".to_string(),
                    ],
                },
            ],
        },
        Checkpoint {
            checkpoint_id: "zig-abi-gate".to_string(),
            description: "Zig ABI contract + promotion checks execute".to_string(),
            paths: vec![
                "contracts/runtime/zig_abi_contract_v1.json".to_string(),
                "scripts/validate_zig_abi_contract.py".to_string(),
                "scripts/check_zig_abi_artifact.py".to_string(),
                "tests/fixtures/runtime/zig_abi_symbols_fixture.txt".to_string(),
                "tests/fixtures/runtime/zig_abi_error_envelope_fixture.json".to_string(),
            ],
            execution_gates: vec![
                ExecutionGate {
                    gate_id: "zig-contract-validation".to_string(),
                    description: "contract schema and readiness gates validate".to_string(),
                    command: vec![
                        "${PYTHON}".to_string(),
                        "scripts/validate_zig_abi_contract.py".to_string(),
                        "--contract".to_string(),
                        "contracts/runtime/zig_abi_contract_v1.json".to_string(),
                    ],
                },
                ExecutionGate {
                    gate_id: "zig-abi-artifact-check".to_string(),
                    description: "required symbols + error envelope pass artifact check".to_string(),
                    command: vec![
                        "${PYTHON}".to_string(),
                        "scripts/check_zig_abi_artifact.py".to_string(),
                        "--contract".to_string(),
                        "contracts/runtime/zig_abi_contract_v1.json".to_string(),
                        "--symbols-file".to_string(),
                        "tests/fixtures/runtime/zig_abi_symbols_fixture.txt".to_string(),
                        "--error-envelope-json".to_string(),
                        "tests/fixtures/runtime/zig_abi_error_envelope_fixture.json".to_string(),
                    ],
                },
            ],
        },
        Checkpoint {
            checkpoint_id: "mojo-gate".to_string(),
            description: "Mojo contract + promotion gate outcome tests execute".to_string(),
            paths: vec![
                "contracts/runtime/mojo_kernel_contract_v1.json".to_string(),
                "scripts/mojo_score_rank_harness.py".to_string(),
                "tests/test_mojo_score_rank_harness.py".to_string(),
            ],
            execution_gates: vec![
                ExecutionGate {
                    gate_id: "mojo-promotion-gate-outcomes".to_string(),
                    description:
                        "harness smoke + enforced promotion-gate behavior are verified".to_string(),
                    command: vec![
                        "${PYTHON}".to_string(),
                        "-m".to_string(),
                        "pytest".to_string(),
                        "-q".to_string(),
                        "tests/test_mojo_score_rank_harness.py::test_run_smoke_with_fake_mojo".to_string(),
                        "tests/test_mojo_score_rank_harness.py::test_run_enforces_promotion_gate_by_default".to_string(),
                    ],
                },
            ],
        },
        Checkpoint {
            checkpoint_id: "runtime-matrix-artifacts".to_string(),
            description: "Wave-2 runtime and migration artifacts are present".to_string(),
            paths: vec![
                "contracts/runtime/runtime-modularization-matrix.json".to_string(),
                "contracts/runtime/wl131_batch_a_rust_migration_v1.json".to_string(),
            ],
            execution_gates: Vec::new(),
        },
    ]
}
