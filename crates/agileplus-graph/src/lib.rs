pub mod config;
pub mod health;
pub mod nodes;
pub mod queries;
pub mod relationships;
pub mod store;

pub use config::GraphConfig;
pub use health::GraphHealth;
pub use nodes::NodeStore;
pub use queries::GraphQueries;
pub use relationships::RelationshipStore;
pub use store::{GraphError, GraphStore};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Graph error: {0}")]
    Graph(#[from] GraphError),
    #[error("Config error: {0}")]
    Config(String),
}
