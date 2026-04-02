use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("provider verification failed: {0}")]
    ProviderVerificationFailed(String),
    #[error("consumer test failed: {0}")]
    ConsumerTestFailed(String),
    #[error("mock server error: {0}")]
    MockServerError(String),
    #[error("pact file error: {0}")]
    PactError(String),
    #[error("interaction mismatch: {expected} vs {actual}")]
    InteractionMismatch { expected: String, actual: String },
}

pub type Result<T> = std::result::Result<T, ContractError>;
