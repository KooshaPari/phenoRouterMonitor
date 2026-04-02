# agileplus-agents Plan

## Overview
Rust workspace for agent orchestration with dispatch, review, and gRPC service.

## Phases

### Phase 1: Core Dispatch (2 weeks)
- Claude Code/Codex subprocess spawning
- Worktree creation automation
- GitHub PR creation via gh CLI

### Phase 2: Review System (2 weeks)
- GitHub/Coderabbit polling
- Review comment feedback loop
- Agent retry orchestration

### Phase 3: gRPC Service (2 weeks)
- AgentDispatchService implementation
- Async gRPC server with tonic
- Client SDK for external integration

## Deliverables
- Dispatch crate with 3+ provider support
- Review crate with polling and feedback
- gRPC service with 99% uptime target
- Integration tests for all components

## Resources
- 1 Rust developer
- Dependencies: tokio, tonic, prost, dashmap