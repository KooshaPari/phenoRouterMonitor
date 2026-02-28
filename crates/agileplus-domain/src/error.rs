#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("not implemented")]
    NotImplemented,

    #[error("invalid governance rule: {0}")]
    InvalidGovernanceRule(String),

    #[error("audit chain error: {0}")]
    AuditChain(String),
}
