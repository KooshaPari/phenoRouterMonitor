//! gRPC service implementation for `AgentDispatchService` (T049b).

use agileplus_agent_dispatch::{
    AgentConfig, AgentDispatchAdapter, AgentKind, AgentPort, AgentTask, JobState,
};
use std::path::PathBuf;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{info, warn};
use uuid::Uuid;

// Include generated protobuf types.
pub mod proto {
    tonic::include_proto!("agileplus.v1");
}

use proto::{
    agent_dispatch_service_server::AgentDispatchService, CancelAgentRequest, CancelAgentResponse,
    GetAgentStatusRequest, GetAgentStatusResponse, SpawnAgentRequest, SpawnAgentResponse,
    StartReviewLoopRequest, StartReviewLoopResponse, GetReviewStatusRequest,
    GetReviewStatusResponse, AgentEventsRequest, AgentEventsResponse,
};

// ─── Service implementation ───────────────────────────────────────────────────

pub struct AgentDispatchServiceImpl {
    adapter: Arc<AgentDispatchAdapter>,
}

impl AgentDispatchServiceImpl {
    pub fn new(adapter: Arc<AgentDispatchAdapter>) -> Self {
        Self { adapter }
    }
}

#[tonic::async_trait]
impl AgentDispatchService for AgentDispatchServiceImpl {
    /// Spawn a new agent job asynchronously, return a job ID.
    async fn spawn_agent(
        &self,
        request: Request<SpawnAgentRequest>,
    ) -> Result<Response<SpawnAgentResponse>, Status> {
        let req = request.into_inner();

        let kind = match req.harness.to_lowercase().as_str() {
            "codex" => AgentKind::Codex,
            _ => AgentKind::ClaudeCode,
        };

        let task = AgentTask {
            job_id: String::new(), // assigned by adapter
            feature_slug: req.feature_slug,
            wp_sequence: req.wp_sequence as u32,
            wp_id: format!("WP{:02}", req.wp_sequence),
            prompt_path: PathBuf::from(&req.prompt_path),
            context_paths: req.context_paths.into_iter().map(PathBuf::from).collect(),
            worktree_path: PathBuf::from(&req.worktree_path),
        };

        let config = AgentConfig {
            kind,
            num_agents: req.max_agents.clamp(1, 3) as usize,
            ..Default::default()
        };

        match self.adapter.dispatch_async(task, config).await {
            Ok(job_id) => {
                info!(%job_id, "agent spawned via gRPC");
                Ok(Response::new(SpawnAgentResponse {
                    success: true,
                    agent_id: job_id,
                    message: "Agent dispatched".to_owned(),
                }))
            }
            Err(e) => {
                warn!(error = %e, "dispatch_async failed");
                Err(Status::internal(e.to_string()))
            }
        }
    }

    /// Query the current status of a job.
    async fn get_agent_status(
        &self,
        request: Request<GetAgentStatusRequest>,
    ) -> Result<Response<GetAgentStatusResponse>, Status> {
        let job_id = request.into_inner().agent_id;

        match self.adapter.query_status(&job_id).await {
            Ok(state) => {
                let state_str = match state {
                    JobState::Pending => "pending",
                    JobState::Running => "running",
                    JobState::Completed => "completed",
                    JobState::Failed => "failed",
                    JobState::Cancelled => "cancelled",
                };

                Ok(Response::new(GetAgentStatusResponse {
                    status: Some(proto::AgentStatus {
                        agent_id: job_id,
                        state: state_str.to_owned(),
                        feature_slug: String::new(),
                        wp_sequence: 0,
                        pr_url: String::new(),
                        review_cycles: 0,
                        last_activity: String::new(),
                    }),
                }))
            }
            Err(e) => Err(Status::not_found(e.to_string())),
        }
    }

    /// Cancel a running agent job.
    async fn cancel_agent(
        &self,
        request: Request<CancelAgentRequest>,
    ) -> Result<Response<CancelAgentResponse>, Status> {
        let req = request.into_inner();

        match self.adapter.cancel(&req.agent_id, &req.reason).await {
            Ok(()) => Ok(Response::new(CancelAgentResponse {
                success: true,
                message: format!("Agent {} cancelled", req.agent_id),
            })),
            Err(e) => Err(Status::not_found(e.to_string())),
        }
    }

    /// Bidirectional streaming for agent events (stub — not yet implemented).
    type AgentEventsStream = tokio_stream::wrappers::ReceiverStream<Result<AgentEventsResponse, Status>>;

    async fn agent_events(
        &self,
        _request: Request<tonic::Streaming<AgentEventsRequest>>,
    ) -> Result<Response<Self::AgentEventsStream>, Status> {
        let (_tx, rx) = tokio::sync::mpsc::channel(1);
        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(rx)))
    }

    /// Trigger the review-fix loop for a PR (stub — delegates to agent-review).
    async fn start_review_loop(
        &self,
        request: Request<StartReviewLoopRequest>,
    ) -> Result<Response<StartReviewLoopResponse>, Status> {
        let req = request.into_inner();
        let review_loop_id = Uuid::new_v4().to_string();
        info!(
            %review_loop_id,
            pr_url = %req.pr_url,
            "review loop started"
        );
        Ok(Response::new(StartReviewLoopResponse {
            success: true,
            review_loop_id,
            message: "Review loop started".to_owned(),
        }))
    }

    /// Get the status of an active review loop (stub).
    async fn get_review_status(
        &self,
        request: Request<GetReviewStatusRequest>,
    ) -> Result<Response<GetReviewStatusResponse>, Status> {
        let id = request.into_inner().review_loop_id;
        Ok(Response::new(GetReviewStatusResponse {
            status: Some(proto::ReviewStatus {
                review_loop_id: id,
                state: "running".to_owned(),
                cycle: 0,
                max_cycles: 5,
                pending_comments: vec![],
                ci_passing: false,
            }),
        }))
    }
}
