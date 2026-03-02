---
audience: [sdk, developers]
---

# gRPC API Reference

AgilePlus exposes a gRPC API for programmatic access to all core domain operations. The API is defined in protobuf and implemented by `agileplus-grpc` (WP09). All methods are async-capable and return structured error responses.

## Service Definition: AgilePlusCoreService

The main service handles feature lifecycle, work packages, governance, audit, and command dispatch.

```protobuf
service AgilePlusCoreService {
  // Feature operations
  rpc GetFeature(GetFeatureRequest) returns (GetFeatureResponse);
  rpc ListFeatures(ListFeaturesRequest) returns (ListFeaturesResponse);
  rpc GetFeatureState(GetFeatureStateRequest) returns (GetFeatureStateResponse);

  // Work package operations
  rpc ListWorkPackages(ListWorkPackagesRequest) returns (ListWorkPackagesResponse);
  rpc GetWorkPackageStatus(GetWorkPackageStatusRequest) returns (GetWorkPackageStatusResponse);

  // Governance operations
  rpc CheckGovernanceGate(CheckGovernanceGateRequest) returns (CheckGovernanceGateResponse);
  rpc GetAuditTrail(GetAuditTrailRequest) returns (stream GetAuditTrailResponse);
  rpc VerifyAuditChain(VerifyAuditChainRequest) returns (VerifyAuditChainResponse);

  // Command dispatch (MCP -> Core)
  rpc DispatchCommand(DispatchCommandRequest) returns (DispatchCommandResponse);

  // Server-streaming real-time agent status events
  rpc StreamAgentEvents(StreamAgentEventsRequest) returns (stream AgentEvent);
}
```

## Connection & Initialization

The default gRPC server listens on port 50051:

```bash
# List available services
grpcurl -plaintext localhost:50051 list

# Query a single feature
grpcurl -plaintext \
  -d '{"slug":"001-feature-slug"}' \
  localhost:50051 agileplus.v1.AgilePlusCoreService/GetFeature
```

### Python Client Example

```python
import grpc
from agileplus.v1 import core_pb2, core_pb2_grpc

# Create a secure or insecure channel
channel = grpc.insecure_channel('localhost:50051')
stub = core_pb2_grpc.AgilePlusCoreServiceStub(channel)

# Query a feature
request = core_pb2.GetFeatureRequest(slug="001-login")
response = stub.GetFeature(request)
print(f"Feature state: {response.feature.state}")
```

### Rust Client Example

```rust
use tonic::transport::Channel;
use agileplus::v1::agileplus_core_service_client::AgilePlusCoreServiceClient;
use agileplus::v1::GetFeatureRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let channel = Channel::from_static("http://127.0.0.1:50051")
        .connect()
        .await?;
    let mut client = AgilePlusCoreServiceClient::new(channel);

    let request = GetFeatureRequest {
        slug: "001-login".to_string(),
    };
    let response = client.get_feature(request).await?;
    println!("Feature: {:?}", response.into_inner().feature);
    Ok(())
}
```

## Core Message Types

### Feature

```protobuf
message Feature {
  int64 id = 1;
  string slug = 2;
  string friendly_name = 3;
  string state = 4;
  string target_branch = 5;
  string created_at = 6;
  string updated_at = 7;
  int32 wp_count = 8;
  int32 wp_done = 9;
}
```

- **slug**: Unique identifier (e.g., `"001-user-login"`)
- **state**: One of `SPECIFY`, `PLAN`, `IMPLEMENT`, `REVIEW`, `DONE`
- **wp_count**: Total work packages for this feature
- **wp_done**: Completed work packages

### WorkPackageStatus

```protobuf
message WorkPackageStatus {
  int64 id = 1;
  string title = 2;
  string state = 3;
  int32 sequence = 4;
  string agent_id = 5;
  string pr_url = 6;
  string pr_state = 7;
  repeated int32 depends_on = 8;
  repeated string file_scope = 9;
}
```

- **state**: One of `PLANNED`, `DOING`, `FOR_REVIEW`, `DONE`
- **pr_url**: GitHub PR URL if a PR exists
- **depends_on**: Sequence numbers of blocking work packages
- **file_scope**: Files this WP is authorized to modify

### Governance & Audit

```protobuf
message GovernanceStatus {
  bool all_gates_passed = 1;
  int32 total_rules = 2;
  int32 passed_rules = 3;
  repeated GateViolation outstanding = 4;
}

message AuditEntry {
  int64 id = 1;
  string feature_slug = 2;
  int32 wp_sequence = 3;
  string timestamp = 4;
  string actor = 5;
  string transition = 6;
  repeated string evidence_refs = 7;
  bytes prev_hash = 8;
  bytes hash = 9;
}
```

Governance gates validate state transitions. Audit entries form an append-only, tamper-evident chain.

## RPC Methods

### GetFeature

```protobuf
rpc GetFeature(GetFeatureRequest) returns (GetFeatureResponse);

message GetFeatureRequest {
  string slug = 1;
}

message GetFeatureResponse {
  Feature feature = 1;
}
```

Returns the full feature record for a given slug.

### ListFeatures

```protobuf
rpc ListFeatures(ListFeaturesRequest) returns (ListFeaturesResponse);

message ListFeaturesRequest {
  string state_filter = 1;  // Optional: "SPECIFY", "PLAN", etc.
}

message ListFeaturesResponse {
  repeated Feature features = 1;
}
```

Returns all features, optionally filtered by state.

### ListWorkPackages

```protobuf
rpc ListWorkPackages(ListWorkPackagesRequest) returns (ListWorkPackagesResponse);

message ListWorkPackagesRequest {
  string feature_slug = 1;
  string state_filter = 2;  // Optional state filter
}

message ListWorkPackagesResponse {
  repeated WorkPackageStatus packages = 1;
}
```

Lists all work packages for a feature.

### CheckGovernanceGate

```protobuf
rpc CheckGovernanceGate(CheckGovernanceGateRequest)
    returns (CheckGovernanceGateResponse);

message CheckGovernanceGateRequest {
  string feature_slug = 1;
  string transition = 2;  // e.g., "IMPLEMENT"
}

message CheckGovernanceGateResponse {
  bool passed = 1;
  repeated GateViolation violations = 2;
}
```

Validates whether a feature can transition to a new state.

### GetAuditTrail (Streaming)

```protobuf
rpc GetAuditTrail(GetAuditTrailRequest) returns (stream GetAuditTrailResponse);

message GetAuditTrailRequest {
  string feature_slug = 1;
  int64 after_id = 2;  // Start from entry ID
}

message GetAuditTrailResponse {
  AuditEntry audit_entry = 1;
}
```

Streams audit entries in chronological order. Useful for replaying history.

### StreamAgentEvents (Streaming)

```protobuf
rpc StreamAgentEvents(StreamAgentEventsRequest)
    returns (stream AgentEvent);

message StreamAgentEventsRequest {
  string feature_slug = 1;
}

message AgentEvent {
  string event_type = 1;      // "started", "completed", "failed"
  string feature_slug = 2;
  int32 wp_sequence = 3;
  string agent_id = 4;        // "claude-code", "codex"
  string payload = 5;         // JSON-encoded event data
  string timestamp = 6;
}
```

Allows real-time monitoring of agent progress on a feature.

## Authentication

API access is controlled via bearer tokens:

```bash
# Pass token in Authorization header
grpcurl -plaintext \
  -H "Authorization: Bearer $AGILEPLUS_API_TOKEN" \
  -d '{"slug":"001-feature"}' \
  localhost:50051 agileplus.v1.AgilePlusCoreService/GetFeature
```

Set the token via environment variable:

```bash
export AGILEPLUS_API_TOKEN="your-secret-token"
export AGILEPLUS_GRPC_HOST="127.0.0.1"
export AGILEPLUS_GRPC_PORT="50051"
```

## Error Handling

All RPC methods return detailed error information via gRPC status codes:

| Code | Meaning |
|------|---------|
| `OK` | Success |
| `INVALID_ARGUMENT` | Malformed request (missing required field, invalid state) |
| `NOT_FOUND` | Feature or work package does not exist |
| `FAILED_PRECONDITION` | State transition not allowed (governance gate violation) |
| `INTERNAL` | Database or VCS error |

Example error response:

```json
{
  "code": "FAILED_PRECONDITION",
  "message": "Cannot transition from SPECIFY to REVIEW: governance gates not passed",
  "details": [
    {
      "field": "rule_id",
      "reason": "FR-REVIEW-001: At least one approved review required"
    }
  ]
}
```
