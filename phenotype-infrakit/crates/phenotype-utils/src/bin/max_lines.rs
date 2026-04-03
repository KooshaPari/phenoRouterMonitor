//! thegent-max-lines: fast max-lines gate across polyglot source files.

use anyhow::{Context, Result};
use clap::Parser;
use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "thegent-max-lines")]
#[command(about = "Fail when tracked source files exceed max line budget")]
struct Args {
    /// Hard failure limit per file.
    #[arg(long, default_value_t = 500)]
    max_lines: usize,

    /// Warning threshold per file.
    #[arg(long, default_value_t = 350)]
    warn_lines: usize,

    /// Comma-separated extensions to check (without dot).
    #[arg(long, default_value = "py,rs,go,zig,mojo,ts,tsx,js,jsx,sh,zsh,bash")]
    exts: String,

    /// Comma-separated path prefixes to exclude.
    #[arg(
        long,
        default_value = ".git,node_modules,dist,build,target,.venv,__pycache__,docs/.vitepress/dist,docs-dist,.shadow-"
    )]
    exclude_prefixes: String,

    /// File scope to scan: changed (default) or all tracked files.
    #[arg(long, default_value = "changed")]
    scope: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("MAX_LINES_GATE FAIL: {err}");
        std::process::exit(2);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    if args.warn_lines > args.max_lines {
        anyhow::bail!("warn-lines cannot exceed max-lines");
    }

    let exts = parse_csv_set(&args.exts);
    let excludes = parse_csv_vec(&args.exclude_prefixes);
    let repo_root = current_repo_root()?;
    let candidates = match args.scope.trim().to_ascii_lowercase().as_str() {
        "changed" => changed_files(&repo_root)?,
        "all" => tracked_files(&repo_root)?,
        other => anyhow::bail!("invalid scope '{other}' (use changed|all)"),
    };

    let mut warnings: Vec<(String, usize)> = Vec::new();
    let mut failures: Vec<(String, usize)> = Vec::new();
    let mut checked = 0usize;

    for rel in candidates {
        if should_exclude(&rel, &excludes) {
            continue;
        }
        if !has_allowed_ext(&rel, &exts) {
            continue;
        }

        let full = repo_root.join(&rel);
        // Skip tracked entries that were removed in the working tree.
        if !full.is_file() {
            continue;
        }
        let lines =
            count_lines(&full).with_context(|| format!("counting lines for {}", rel.display()))?;
        checked += 1;

        let rel_s = rel.to_string_lossy().to_string();
        if lines > args.max_lines {
            failures.push((rel_s, lines));
        } else if lines > args.warn_lines {
            warnings.push((rel_s, lines));
        }
    }

    warnings.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    failures.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    for (path, lines) in &warnings {
        println!("[WARN] {path}: {lines} lines (>{})", args.warn_lines);
    }
    for (path, lines) in &failures {
        eprintln!("[FAIL] {path}: {lines} lines (max {})", args.max_lines);
    }

    println!(
        "MAX_LINES_GATE summary: checked={} warn={} fail={} max={} warn_at={}",
        checked,
        warnings.len(),
        failures.len(),
        args.max_lines,
        args.warn_lines
    );

    if failures.is_empty() {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

fn current_repo_root() -> Result<PathBuf> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("running git rev-parse --show-toplevel")?;
    if !out.status.success() {
        anyhow::bail!("not in git repository");
    }
    let root = String::from_utf8(out.stdout).context("repo root output not utf-8")?;
    Ok(PathBuf::from(root.trim()))
}

fn tracked_files(repo_root: &Path) -> Result<Vec<PathBuf>> {
    let out = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(repo_root)
        .output()
        .context("running git ls-files -z")?;
    if !out.status.success() {
        anyhow::bail!("git ls-files failed");
    }

    let files = out
        .stdout
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| PathBuf::from(String::from_utf8_lossy(s).to_string()))
        .collect();
    Ok(files)
}

fn changed_files(repo_root: &Path) -> Result<Vec<PathBuf>> {
    if let Ok(base_ref) = env::var("GITHUB_BASE_REF") {
        let base_ref = base_ref.trim();
        if !base_ref.is_empty() {
            let range = format!("origin/{base_ref}...HEAD");
            let from_base = git_list(
                repo_root,
                &[
                    "diff",
                    "--name-only",
                    "--diff-filter=ACMRTUXB",
                    &range,
                    "-z",
                ],
            )?;
            if !from_base.is_empty() {
                return Ok(from_base);
            }
        }
    }

    // Prefer staged changes (pre-commit), then fallback to working tree changes.
    let staged = git_list(
        repo_root,
        &[
            "diff",
            "--cached",
            "--name-only",
            "--diff-filter=ACMRTUXB",
            "-z",
        ],
    )?;
    if !staged.is_empty() {
        return Ok(staged);
    }

    let mut changed = git_list(
        repo_root,
        &["diff", "--name-only", "--diff-filter=ACMRTUXB", "-z"],
    )?;
    let mut untracked = git_list(
        repo_root,
        &["ls-files", "--others", "--exclude-standard", "-z"],
    )?;
    changed.append(&mut untracked);
    dedup_paths(changed)
}

fn git_list(repo_root: &Path, args: &[&str]) -> Result<Vec<PathBuf>> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .with_context(|| format!("running git {}", args.join(" ")))?;
    if !out.status.success() {
        anyhow::bail!("git {} failed", args.join(" "));
    }
    Ok(out
        .stdout
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .map(|s| PathBuf::from(String::from_utf8_lossy(s).to_string()))
        .collect())
}

fn dedup_paths(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for p in paths {
        let k = p.to_string_lossy().to_string();
        if seen.insert(k) {
            out.push(p);
        }
    }
    Ok(out)
}

fn parse_csv_set(raw: &str) -> HashSet<String> {
    raw.split(',')
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

fn parse_csv_vec(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn should_exclude(path: &Path, excludes: &[String]) -> bool {
    let path_s = path.to_string_lossy();
    excludes.iter().any(|prefix| {
        path_s == *prefix || path_s.starts_with(&format!("{prefix}/")) || path_s.starts_with(prefix)
    })
}

fn has_allowed_ext(path: &Path, exts: &HashSet<String>) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => exts.contains(&ext.to_ascii_lowercase()),
        None => false,
    }
}

fn count_lines(path: &Path) -> Result<usize> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    if bytes.is_empty() {
        return Ok(0);
    }
    let nl = bytes.iter().filter(|b| **b == b'\n').count();
    let ends_with_nl = bytes.last().is_some_and(|b| *b == b'\n');
    Ok(if ends_with_nl { nl } else { nl + 1 })
}
