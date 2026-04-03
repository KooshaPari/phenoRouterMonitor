//! thegent-grep: Safe grep wrapper that avoids alias/config issues
//! Replaces scripts/safe-grep.sh

use anyhow::Result;
use clap::Parser;
use std::env;
use std::process::Command;

#[derive(Parser)]
#[command(name = "thegent-grep")]
#[command(about = "Safe grep wrapper that avoids alias/config issues")]
struct Args {
    /// Use ripgrep (rg) instead of grep
    #[arg(long)]
    rg: bool,

    /// Arguments to pass to grep/rg
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Determine which tool to use
    let use_rg = args.rg || args.args.is_empty() || !args.args[0].starts_with('-');

    if use_rg {
        exec_rg(&args.args)
    } else {
        exec_grep(&args.args)
    }
}

fn exec_rg(args: &[String]) -> Result<()> {
    // Find rg binary
    let rg_bin = find_binary("rg").ok_or_else(|| anyhow::anyhow!("rg not found in PATH"))?;

    // Clean environment to avoid config errors
    let mut cmd = Command::new(rg_bin);
    cmd.args(args);
    cmd.arg("--no-config"); // Disable rg config

    // Remove problematic environment variables
    cmd.env_remove("GREP_OPTIONS");
    cmd.env_remove("GREP_COLOR");
    cmd.env_remove("GREP_COLORS");
    cmd.env_remove("_proxy");
    cmd.env_remove("CURSOR_SANDBOX");
    cmd.env_remove("SUDO_ASKPASS");
    cmd.env_remove("CURSOR_ASKPASS");

    // Execute
    let status = cmd.status()?;
    std::process::exit(status.code().unwrap_or(1));
}

fn exec_grep(args: &[String]) -> Result<()> {
    // Find grep binary (use system grep, not aliased version)
    let grep_bin = find_binary("grep").ok_or_else(|| anyhow::anyhow!("grep not found in PATH"))?;

    let mut cmd = Command::new(grep_bin);
    cmd.args(args);

    // Remove problematic environment variables
    cmd.env_remove("GREP_OPTIONS");
    cmd.env_remove("GREP_COLOR");
    cmd.env_remove("GREP_COLORS");
    cmd.env_remove("_proxy");
    cmd.env_remove("CURSOR_SANDBOX");
    cmd.env_remove("SUDO_ASKPASS");
    cmd.env_remove("CURSOR_ASKPASS");

    // Execute
    let status = cmd.status()?;
    std::process::exit(status.code().unwrap_or(1));
}

fn find_binary(name: &str) -> Option<String> {
    // Try safe PATH first
    let safe_path = env::var("THEGENT_TOOL_BIN_PATH")
        .unwrap_or_else(|_| "/usr/bin:/opt/homebrew/bin:/bin:/usr/sbin:/sbin".to_string());

    // Use 'command -v' to find binary
    let output = Command::new("command")
        .arg("-v")
        .arg(name)
        .env("PATH", &safe_path)
        .output()
        .ok()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(path);
        }
    }

    // Fallback to regular PATH
    let output = Command::new("command").arg("-v").arg(name).output().ok()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() {
            return Some(path);
        }
    }

    None
}
