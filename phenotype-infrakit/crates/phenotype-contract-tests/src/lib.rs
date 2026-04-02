pub mod contract;
pub mod consumer;
pub mod error;
pub mod mock_server;
pub mod pact;
pub mod provider;

pub use contract::{Contract, Interaction, HttpInteraction, ProviderState};
pub use consumer::{ConsumerTest, InteractionBuilder};
pub use error::{ContractError, Result};
pub use mock_server::MockServerAdapter;
pub use pact::PactFile;
pub use provider::ProviderVerifier;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
