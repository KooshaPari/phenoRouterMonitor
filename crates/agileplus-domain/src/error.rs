#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("not implemented")]
    NotImplemented,
}
