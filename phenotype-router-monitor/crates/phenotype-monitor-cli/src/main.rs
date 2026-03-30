//! phenotype-monitor CLI
//!
//! Real-time monitoring tool for router, metrics, and metering systems.

use clap::Parser;

#[derive(Parser)]
#[command(name = "phenotype-monitor")]
#[command(about = "Real-time monitoring for phenotype-router-monitor", long_about = None)]
struct Args {
    #[arg(long, default_value = "127.0.0.1:9090")]
    api_addr: String,

    #[arg(long, default_value = "5")]
    refresh_interval: u64,
}

fn main() {
    let _args = Args::parse();
    println!("phenotype-monitor: CLI tool for router/metrics monitoring");
    println!("This is a placeholder. Implementation coming with Phase 1 extraction.");
}
