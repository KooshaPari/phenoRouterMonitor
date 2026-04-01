//! Phenotype Capital — organizational resource ledger and secret management.
//!
//! Tracks LLM accounts, cloud credits, free-tier resources, and browser profiles
//! for agent-aware resource allocation and consumption monitoring.
//!
//! # Example
//!
//! ```rust
//! use phenotype_capital::{CapitalLedger, Registry};
//!
//! let registry = Registry::from_capital_toml("capital.toml")?;
//! let ledger = CapitalLedger::new(".agileplus/capital.db")?;
//! let accounts = registry.accounts_by_tag("free-tier");
//! ```

pub mod ledger;
pub mod registry;
pub mod secrets;

pub use ledger::CapitalLedger;
pub use registry::{Account, AccountType, Registry, RateLimit, Rotation};
pub use secrets::{SecretEntry, SecretStatus, SecretManager};
