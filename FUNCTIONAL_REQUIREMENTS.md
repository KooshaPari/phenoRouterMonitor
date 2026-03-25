# Functional Requirements - phenotype-infrakit

## FR-EVT-001: Event Append
The system SHALL append events with monotonically increasing sequence numbers.

## FR-EVT-002: Hash Chain
Each event SHALL include a SHA-256 hash linking to the previous event for integrity verification.

## FR-EVT-003: Snapshot Support
The event store SHALL support creating and loading snapshots to optimize replay.

## FR-EVT-004: Pluggable Storage
The event store SHALL define a trait for storage backends (in-memory provided by default).

## FR-CACHE-001: Two-Tier Lookup
Cache lookups SHALL check L1 (LRU) first, then L2 (DashMap) on L1 miss.

## FR-CACHE-002: TTL Expiration
Cache entries SHALL be evicted after their TTL expires.

## FR-CACHE-003: Metrics Hook
The cache SHALL accept an optional `MetricsHook` for hit/miss observability.

## FR-POL-001: Rule Loading
The policy engine SHALL load rules from TOML strings and files.

## FR-POL-002: Allow/Deny/Require Actions
Rules SHALL support `allow`, `deny`, and `require` actions with field matching.

## FR-POL-003: Severity Levels
Rules SHALL have configurable severity levels for graduated enforcement.

## FR-POL-004: Context Evaluation
The engine SHALL evaluate rules against a `PolicyContext` key-value map.

## FR-SM-001: Forward-Only Transitions
The state machine SHALL reject transitions to states with lower ordinal values.

## FR-SM-002: Transition Guards
The state machine SHALL support guard callbacks that can reject transitions.

## FR-SM-003: History Tracking
The state machine SHALL maintain a full history of state transitions.
