//! thegent-monitor: Process monitoring utility
//! Replaces scripts/monitor-process-count.sh

use anyhow::Result;
use clap::Parser;
use colored::*;
use std::process::Command;

#[derive(Parser)]
#[command(name = "thegent-monitor")]
#[command(about = "Monitor process count and detect fork failures")]
struct Args {
    /// Watch mode: continuously monitor
    #[arg(short, long)]
    watch: bool,

    /// Interval in seconds for watch mode
    #[arg(short, long, default_value = "1")]
    interval: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.watch {
        loop {
            print!("\x1B[2J\x1B[1;1H"); // Clear screen
            monitor_once()?;
            std::thread::sleep(std::time::Duration::from_secs(args.interval));
        }
    } else {
        monitor_once()?;
    }

    Ok(())
}

fn monitor_once() -> Result<()> {
    println!(
        "{}",
        "🔍 Monitoring process count and fork failures...".blue()
    );
    println!();

    // Current process count
    let proc_count = get_process_count()?;
    println!("{} {}", "Current process count:".blue(), proc_count);

    // System limits
    let max_procs = get_max_processes().unwrap_or_else(|| "unknown".to_string());
    println!("{} {}", "Max user processes:".blue(), max_procs);

    // Check for fork failures
    let fork_failures = get_fork_failures().unwrap_or(0);
    if fork_failures > 0 {
        println!(
            "{} {}",
            "⚠️  Fork failures detected in system logs:".yellow(),
            fork_failures
        );
    }

    // Check thegent-related processes
    let thegent_procs = get_thegent_processes()?;
    println!("{} {}", "thegent-related processes:".blue(), thegent_procs);

    // Analysis and recommendations
    println!();
    println!("{}", "📊 Analysis:".blue());

    if proc_count > 500 {
        println!(
            "  {} ({})",
            "🔴 CRITICAL: Process count is very high".red(),
            proc_count
        );
        println!("     Recommendation: Restart shell, apply fast-path fixes");
    } else if proc_count > 200 {
        println!(
            "  {} ({})",
            "🟡 WARNING: Process count is elevated".yellow(),
            proc_count
        );
        println!("     Recommendation: Monitor and consider applying fixes");
    } else {
        println!(
            "  {} ({})",
            "✅ Process count is normal".green(),
            proc_count
        );
    }

    if thegent_procs > 50 {
        println!(
            "  {} ({})",
            "🟡 WARNING: Many thegent processes".yellow(),
            thegent_procs
        );
        println!("     Recommendation: Check for process leaks");
    }

    if fork_failures > 0 {
        println!(
            "  {} ({})",
            "🔴 CRITICAL: Fork failures detected".red(),
            fork_failures
        );
        println!("     Recommendation: Increase process limit, restart shell");
    }

    println!();
    println!("{}", "💡 To fix:".blue());
    println!("   1. Run: bash thegent/scripts/fix-which-timeout.sh");
    println!("   2. Restart your shell");
    println!("   3. Monitor: watch -n 1 'ps aux | wc -l'");
    println!();

    Ok(())
}

fn get_process_count() -> Result<usize> {
    let output = Command::new("ps").arg("aux").output()?;

    if output.status.success() {
        let count = String::from_utf8_lossy(&output.stdout).lines().count();
        Ok(count.saturating_sub(1)) // Subtract header line
    } else {
        Ok(0)
    }
}

fn get_max_processes() -> Option<String> {
    use std::process::Command;

    // Try ulimit -u
    let output = Command::new("sh")
        .arg("-c")
        .arg("ulimit -u")
        .output()
        .ok()?;

    if output.status.success() {
        let limit = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !limit.is_empty() && limit != "unlimited" {
            return Some(limit);
        }
    }

    None
}

fn get_fork_failures() -> Option<usize> {
    // Try dmesg if available
    let output = Command::new("dmesg").output().ok()?;

    if output.status.success() {
        let dmesg_output = String::from_utf8_lossy(&output.stdout);
        let count = dmesg_output
            .lines()
            .filter(|line| line.contains("fork: retry"))
            .count();
        Some(count)
    } else {
        None
    }
}

fn get_thegent_processes() -> Result<usize> {
    let output = Command::new("ps").arg("aux").output()?;

    if output.status.success() {
        let ps_output = String::from_utf8_lossy(&output.stdout);
        let count = ps_output
            .lines()
            .filter(|line| {
                line.contains("thegent") || line.contains("common.sh") || line.contains("hook")
            })
            .count();
        Ok(count)
    } else {
        Ok(0)
    }
}
