//! # Phenotype MCP Server
//!
//! FastMCP v3.0 integration for Phenotype agent tools.
//!
//! ## Overview
//!
//! This crate provides Model Context Protocol (MCP) server implementation
//! for Phenotype's agent tooling. Built on [FastMCP v3.0 GA](https://github.com/PrefectHQ/fastmcp)
//! which is used by 70%+ of all MCP servers.
//!
//! ## Features
//!
//! - Tool definitions with typed schemas
//! - Resource management
//! - Prompt templates
//! - OpenTelemetry instrumentation
//! - Component versioning
//! - Granular authorization
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use phenotype_mcp::{PhenotypeMcpServer, tools};
//! use fastmcp::Server;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let server = PhenotypeMcpServer::new();
//!     server.register_tools().await?;
//!     server.run().await
//! }
//! ```

pub mod tools;
pub mod resources;
pub mod prompts;

use anyhow::Result;
use fastmcp::{Server, Tool};
use std::sync::Arc;
use tracing::{info, instrument};

/// Phenotype MCP Server
///
/// Built on FastMCP v3.0 GA with component versioning,
/// granular authorization, and OpenTelemetry support.
pub struct PhenotypeMcpServer {
    server: Server,
    context: Arc<ServerContext>,
}

/// Server context shared across all tool handlers
#[derive(Clone)]
pub struct ServerContext {
    pub version: String,
    pub environment: String,
}

impl PhenotypeMcpServer {
    /// Create a new Phenotype MCP server
    pub fn new() -> Self {
        let server = Server::new("phenotype");

        let context = Arc::new(ServerContext {
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: std::env::var("PHENOTYPE_ENV")
                .unwrap_or_else(|_| "development".to_string()),
        });

        Self { server, context }
    }

    /// Register all Phenotype tools with the MCP server
    #[instrument(skip(self))]
    pub async fn register_tools(&self) -> Result<()> {
        info!("Registering Phenotype MCP tools");

        // AgilePlus tools
        self.register_agileplus_tools().await?;

        // Phenotype tools
        self.register_phenotype_tools().await?;

        // Agent dispatch tools
        self.register_dispatch_tools().await?;

        info!("All tools registered successfully");
        Ok(())
    }

    async fn register_agileplus_tools(&self) -> Result<()> {
        // Feature specification tool
        self.server.add_tool(Tool::new(
            "agileplus_create_feature",
            "Create a new feature specification in AgilePlus",
            tools::agileplus::CreateFeatureInput::schema(),
            |ctx: Arc<ServerContext>, args| {
                let ctx = ctx.clone();
                async move {
                    tools::agileplus::create_feature(ctx, args).await
                }
            },
        )).await?;

        // Feature validation tool
        self.server.add_tool(Tool::new(
            "agileplus_validate",
            "Validate a feature against governance rules",
            tools::agileplus::ValidateFeatureInput::schema(),
            |ctx: Arc<ServerContext>, args| {
                let ctx = ctx.clone();
                async move {
                    tools::agileplus::validate_feature(ctx, args).await
                }
            },
        )).await?;

        // Work package status tool
        self.server.add_tool(Tool::new(
            "agileplus_status",
            "Update work package status",
            tools::agileplus::UpdateStatusInput::schema(),
            |ctx: Arc<ServerContext>, args| {
                let ctx = ctx.clone();
                async move {
                    tools::agileplus::update_status(ctx, args).await
                }
            },
        )).await?;

        Ok(())
    }

    async fn register_phenotype_tools(&self) -> Result<()> {
        // Spec parsing tool
        self.server.add_tool(Tool::new(
            "phenotype_parse_spec",
            "Parse and validate a feature specification",
            tools::phenotype::ParseSpecInput::schema(),
            |ctx: Arc<ServerContext>, args| {
                let ctx = ctx.clone();
                async move {
                    tools::phenotype::parse_spec(ctx, args).await
                }
            },
        )).await?;

        // Merge analysis tool
        self.server.add_tool(Tool::new(
            "phenotype_merge_analysis",
            "Analyze specification merge conflicts",
            tools::phenotype::MergeAnalysisInput::schema(),
            |ctx: Arc<ServerContext>, args| {
                let ctx = ctx.clone();
                async move {
                    tools::phenotype::analyze_merge(ctx, args).await
                }
            },
        )).await?;

        Ok(())
    }

    async fn register_dispatch_tools(&self) -> Result<()> {
        // Agent dispatch tool
        self.server.add_tool(Tool::new(
            "agent_dispatch",
            "Dispatch work to an AI agent",
            tools::dispatch::DispatchInput::schema(),
            |ctx: Arc<ServerContext>, args| {
                let ctx = ctx.clone();
                async move {
                    tools::dispatch::dispatch(ctx, args).await
                }
            },
        )).await?;

        // Agent status tool
        self.server.add_tool(Tool::new(
            "agent_status",
            "Check agent task status",
            tools::dispatch::StatusInput::schema(),
            |ctx: Arc<ServerContext>, args| {
                let ctx = ctx.clone();
                async move {
                    tools::dispatch::status(ctx, args).await
                }
            },
        )).await?;

        Ok(())
    }

    /// Run the MCP server
    #[instrument(skip(self))]
    pub async fn run(&self) -> Result<()> {
        info!(
            version = %self.context.version,
            env = %self.context.environment,
            "Starting Phenotype MCP server"
        );

        self.server.run().await?;

        Ok(())
    }

    /// Get the server transport URL
    pub fn transport_url(&self) -> String {
        std::env::var("MCP_TRANSPORT_URL")
            .unwrap_or_else(|_| "stdio".to_string())
    }
}

impl Default for PhenotypeMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export tool input/output types for convenience
pub use tools::agileplus::{CreateFeatureInput, CreateFeatureOutput};
pub use tools::agileplus::{ValidateFeatureInput, ValidateFeatureOutput};
pub use tools::agileplus::{UpdateStatusInput, UpdateStatusOutput};
pub use tools::phenotype::{ParseSpecInput, ParseSpecOutput};
pub use tools::phenotype::{MergeAnalysisInput, MergeAnalysisOutput};
pub use tools::dispatch::{DispatchInput, DispatchOutput};
pub use tools::dispatch::{StatusInput, StatusOutput};
