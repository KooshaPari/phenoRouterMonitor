[workspace]
resolver = "3"
members = [
  # Core crates
  "crates/agileplus-domain",
  "crates/agileplus-cli",
  "crates/agileplus-api",
  "crates/agileplus-grpc",
  "crates/agileplus-sqlite",
  "crates/agileplus-git",
  "crates/agileplus-plane",
  "crates/agileplus-telemetry",
  "crates/agileplus-triage",
  "crates/agileplus-events",
  "crates/agileplus-cache",
  "crates/agileplus-subcmds",
  "crates/agileplus-graph",
  "crates/agileplus-nats",
  "crates/agileplus-sync",
  "crates/agileplus-dashboard",
  "crates/agileplus-github",
  "crates/agileplus-p2p",
  "crates/agileplus-integration-tests",
  "crates/agileplus-contract-tests",
  "crates/agileplus-benchmarks",
  
  # Libraries
  "libs/nexus",
  "libs/plugin-registry",
  "libs/plugin-sample",
  "libs/plugin-cli",
  "libs/plugin-git",
  "libs/plugin-grpc",
  "libs/plugin-integration",
  "libs/intent-registry",
  "libs/health-monitor",
]

[workspace.package]
version = "0.1.1"
edition = "2021"
license = "MIT"
rust-version = "1.75"
authors = ["AgilePlus Team"]
repository = "https://github.com/AgilePlus/agileplus"
keywords = ["agile", "project-management", "software-development"]
categories = ["Software Development", "Project Management"]

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
sha2 = "0.10"
tokio = { version = "1", features = ["full"] }
thiserror = "1"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
tracing-opentelemetry = "0.24"
futures-util = "0.3"
axum = { version = "0.7", features = ["json", "macros"] }
tonic = { version = "0.10", features = ["transport"] }
tonic-build = { version = "0.10", features = ["prost"] }
prost = "0.12"
opentelemetry = { version = "0.21", features = ["trace", "metrics"] }
opentelemetry-otlp = { version = "0.14", features = [
  "trace",
  "metrics",
  "http-proto",
  "reqwest-client",
] }
opentelemetry_sdk = { version = "0.21", features = [
  "trace",
  "metrics",
  "rt-tokio",
] }
criterion = { version = "0.5", features = ["async_tokio", "html_reports"] }
time = ">=0.3.49"
trait-variant = "0.1"
async-trait = "0.1"

# Plugin crates (external from separate repos)
agileplus-plugin-core = { git = "https://github.com/KooshaPari/agileplus-plugin-core" }
agileplus-plugin-git = { git = "https://github.com/KooshaPari/agileplus-plugin-git" }
agileplus-plugin-sqlite = { git = "https://github.com/KooshaPari/agileplus-plugin-sqlite" }

# Git operations
gix = "0.67"
git2 = "0.18"

# Database
rusqlite = { version = "0.30", features = ["bundled"] }