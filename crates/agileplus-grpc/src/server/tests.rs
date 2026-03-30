use super::domain_error_to_status;

#[test]
fn domain_error_mapping() {
    use agileplus_domain::error::DomainError;

    let status = domain_error_to_status(DomainError::NotFound("feat".into()));
    assert_eq!(status.code(), tonic::Code::NotFound);

    let status = domain_error_to_status(DomainError::InvalidTransition {
        from: "a".into(),
        to: "b".into(),
        reason: "test".into(),
    });
    assert_eq!(status.code(), tonic::Code::FailedPrecondition);

    let status = domain_error_to_status(DomainError::Conflict("x".into()));
    assert_eq!(status.code(), tonic::Code::AlreadyExists);
}
