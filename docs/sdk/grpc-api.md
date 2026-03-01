---
audience: [sdk, developers]
---

# gRPC API Reference

AgilePlus exposes a gRPC API for programmatic access to all core operations.

## Service Definition

```protobuf
service AgilePlus {
  // Feature lifecycle
  rpc CreateFeature (CreateFeatureRequest) returns (Feature);
  rpc GetFeature (GetFeatureRequest) returns (Feature);
  rpc ListFeatures (ListFeaturesRequest) returns (ListFeaturesResponse);

  // Specification
  rpc GenerateSpec (GenerateSpecRequest) returns (Spec);
  rpc UpdateSpec (UpdateSpecRequest) returns (Spec);

  // Planning
  rpc GeneratePlan (GeneratePlanRequest) returns (Plan);
  rpc ListWorkPackages (ListWorkPackagesRequest) returns (ListWorkPackagesResponse);

  // Execution
  rpc DispatchAgent (DispatchRequest) returns (DispatchResponse);
  rpc GetWorkPackageStatus (GetStatusRequest) returns (WorkPackageStatus);

  // Sync
  rpc SyncTracker (SyncRequest) returns (SyncResponse);
}
```

## Connection

```bash
# Default endpoint
grpcurl -plaintext localhost:50051 list
```

```python
import grpc
from agileplus_pb2_grpc import AgilePlusStub

channel = grpc.insecure_channel('localhost:50051')
client = AgilePlusStub(channel)
```

## Key Types

### Feature

```protobuf
message Feature {
  string id = 1;
  string slug = 2;
  string title = 3;
  FeatureState state = 4;
  google.protobuf.Timestamp created_at = 5;
}

enum FeatureState {
  SPECIFY = 0;
  PLAN = 1;
  IMPLEMENT = 2;
  REVIEW = 3;
  DONE = 4;
}
```

### WorkPackage

```protobuf
message WorkPackage {
  string id = 1;
  string feature_id = 2;
  string title = 3;
  WorkPackageLane lane = 4;
  repeated string dependencies = 5;
  repeated Task subtasks = 6;
}

enum WorkPackageLane {
  PLANNED = 0;
  DOING = 1;
  FOR_REVIEW = 2;
  DONE = 3;
}
```

## Authentication

The gRPC API uses token-based authentication:

```bash
export AGILEPLUS_API_TOKEN="your-token"
grpcurl -H "Authorization: Bearer $AGILEPLUS_API_TOKEN" localhost:50051 ...
```
