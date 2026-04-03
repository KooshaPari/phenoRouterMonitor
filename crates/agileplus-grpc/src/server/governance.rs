use std::pin::Pin;

use tonic::{Response, Status};

use agileplus_domain::domain::audit::AuditChain;
use agileplus_domain::ports::{AgentPort, ObservabilityPort, ReviewPort, StoragePort, VcsPort};
use agileplus_proto::agileplus::v1::{
    CheckGovernanceGateRequest, CheckGovernanceGateResponse, GateViolation as ProtoGateViolation,
    GetAuditTrailRequest, GetAuditTrailResponse, VerifyAuditChainRequest, VerifyAuditChainResponse,
};

use super::{AgilePlusCoreServer, domain_error_to_status};
use crate::conversions::audit_entry_to_proto;

pub(super) type GetAuditTrailStream =
    Pin<Box<dyn tokio_stream::Stream<Item = Result<GetAuditTrailResponse, Status>> + Send>>;

impl<S, V, A, R, O> AgilePlusCoreServer<S, V, A, R, O>
where
    S: StoragePort + 'static,
    V: VcsPort + 'static,
    A: AgentPort + 'static,
    R: ReviewPort + 'static,
    O: ObservabilityPort + 'static,
{
    pub(super) async fn handle_check_governance_gate(
        &self,
        request: CheckGovernanceGateRequest,
    ) -> Result<Response<CheckGovernanceGateResponse>, Status> {
        let feature = self
            .storage
            .get_feature_by_slug(&request.feature_slug)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| {
                Status::not_found(format!("feature '{}' not found", request.feature_slug))
            })?;

        let contract = self
            .storage
            .get_latest_governance_contract(feature.id)
            .await
            .map_err(domain_error_to_status)?;

        let Some(contract) = contract else {
            return Ok(Response::new(CheckGovernanceGateResponse {
                passed: true,
                violations: Vec::new(),
            }));
        };

        let relevant_rules: Vec<_> = contract
            .rules
            .iter()
            .filter(|rule| rule.transition.is_empty() || rule.transition == request.transition)
            .collect();

        let evidence = self
            .storage
            .get_evidence_by_fr("")
            .await
            .unwrap_or_default();
        let mut violations = Vec::new();

        for rule in &relevant_rules {
            for required_evidence in &rule.required_evidence {
                let satisfied = evidence.iter().any(|candidate| {
                    candidate.fr_id == required_evidence.fr_id
                        && format!("{:?}", candidate.evidence_type).to_lowercase()
                            == format!("{:?}", required_evidence.evidence_type).to_lowercase()
                });

                if !satisfied {
                    violations.push(ProtoGateViolation {
                        fr_id: required_evidence.fr_id.clone(),
                        rule_id: rule.transition.clone(),
                        message: format!(
                            "Missing required evidence '{:?}' for FR {}",
                            required_evidence.evidence_type, required_evidence.fr_id
                        ),
                        remediation: format!(
                            "Provide evidence of type '{:?}' for FR {}",
                            required_evidence.evidence_type, required_evidence.fr_id
                        ),
                    });
                }
            }
        }

        Ok(Response::new(CheckGovernanceGateResponse {
            passed: violations.is_empty(),
            violations,
        }))
    }

    pub(super) async fn handle_get_audit_trail(
        &self,
        request: GetAuditTrailRequest,
    ) -> Result<Response<GetAuditTrailStream>, Status> {
        let feature = self
            .storage
            .get_feature_by_slug(&request.feature_slug)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| {
                Status::not_found(format!("feature '{}' not found", request.feature_slug))
            })?;

        let entries = self
            .storage
            .get_audit_trail(feature.id)
            .await
            .map_err(domain_error_to_status)?;

        let slug = request.feature_slug;
        let after_id = request.after_id;
        let filtered = entries.into_iter().filter(|entry| entry.id > after_id);

        let stream = tokio_stream::iter(filtered.map(move |entry| {
            let mut proto = audit_entry_to_proto(entry);
            proto.feature_slug = slug.clone();
            Ok(GetAuditTrailResponse {
                audit_entry: Some(proto),
            })
        }));

        Ok(Response::new(Box::pin(stream)))
    }

    pub(super) async fn handle_verify_audit_chain(
        &self,
        request: VerifyAuditChainRequest,
    ) -> Result<Response<VerifyAuditChainResponse>, Status> {
        let slug = request.feature_slug;
        let feature = self
            .storage
            .get_feature_by_slug(&slug)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| Status::not_found(format!("feature '{slug}' not found")))?;

        let entries = self
            .storage
            .get_audit_trail(feature.id)
            .await
            .map_err(domain_error_to_status)?;

        let entries_verified = entries.len() as i64;
        let chain = AuditChain { entries };

        match chain.verify_chain() {
            Ok(()) => Ok(Response::new(VerifyAuditChainResponse {
                valid: true,
                entries_verified,
                first_invalid_id: String::new(),
                error_message: String::new(),
            })),
            Err(e) => Ok(Response::new(VerifyAuditChainResponse {
                valid: false,
                entries_verified: 0,
                first_invalid_id: String::new(),
                error_message: e.to_string(),
            })),
        }
    }
}
