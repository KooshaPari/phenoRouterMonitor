# Phenotype Governance Specification

> Detailed specification for the policy-as-code governance engine

## Overview

Phenotype Governance provides a comprehensive policy engine for defining, evaluating, and enforcing organizational standards.

## Core Components

### Policy Engine

```rust
pub struct PolicyEngine {
    policies: Arc<RwLock<HashMap<String, Policy>>>,
    evaluators: HashMap<String, Box<dyn Evaluator>>,
    enforcers: Vec<Box<dyn Enforcer>>,
    audit_log: Arc<dyn AuditLog>,
}

impl PolicyEngine {
    pub async fn evaluate(&self, target: &Target) -> EvaluationResult;
    pub async fn enforce(&self, result: &EvaluationResult) -> EnforcementResult;
    pub fn register_evaluator(&mut self, name: &str, evaluator: Box<dyn Evaluator>);
}
```

### Policy Definition

```rust
pub struct Policy {
    pub id: String,
    pub version: Version,
    pub metadata: PolicyMetadata,
    pub rules: Vec<Rule>,
    pub enforcement: EnforcementConfig,
}

pub struct PolicyMetadata {
    pub name: String,
    pub description: String,
    pub category: PolicyCategory,
    pub severity: Severity,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum PolicyCategory {
    Security,
    Quality,
    Process,
    Compliance,
    Custom(String),
}
```

### Rule System

```rust
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: Condition,
    pub message: String,
    pub severity: Severity,
    pub autofix: Option<AutoFix>,
}

pub enum Condition {
    // Pattern matching
    Regex { pattern: Regex, target: String },
    
    // File operations
    FileExists { path: String },
    FileContains { path: String, content: String },
    FileMatches { path: String, pattern: Regex },
    
    // Git operations
    CommitMessageMatches { pattern: Regex },
    BranchMatches { pattern: Regex },
    AuthorMatches { pattern: Regex },
    
    // Logical operators
    And(Vec<Condition>),
    Or(Vec<Condition>),
    Not(Box<Condition>),
    
    // Custom evaluators
    Custom(Box<dyn Fn(&Context) -> bool>),
}
```

### Evaluation Context

```rust
pub struct EvaluationContext {
    pub target: Target,
    pub environment: Environment,
    pub history: Vec<EvaluationRecord>,
    pub cache: EvaluationCache,
}

pub enum Target {
    File { path: PathBuf, content: String },
    Directory { path: PathBuf, files: Vec<FileInfo> },
    GitCommit { hash: String, message: String, diff: String },
    GitBranch { name: String, commits: Vec<String> },
    PullRequest { id: u64, title: String, diff: String },
}

pub struct Environment {
    pub cwd: PathBuf,
    pub git_root: Option<PathBuf>,
    pub env_vars: HashMap<String, String>,
    pub config: Config,
}
```

### Enforcement System

```rust
pub trait Enforcer: Send + Sync {
    fn name(&self) -> &str;
    fn can_handle(&self, policy: &Policy) -> bool;
    fn enforce(&self, result: &EvaluationResult) -> EnforcementResult;
}

pub struct AuditEnforcer {
    logger: Arc<dyn AuditLog>,
}

pub struct WarningEnforcer {
    output: Box<dyn Write>,
}

pub struct BlockingEnforcer {
    exit_code: i32,
}
```

## Policy Evaluation Flow

```
1. Load Policies
   ↓
2. Parse Target
   ↓
3. For each Policy:
   a. Check if applies to target
   b. For each Rule:
      i. Evaluate condition
      ii. If failed, record violation
   c. Aggregate violations
   ↓
4. Apply Enforcement
   a. Determine enforcement level
   b. Execute enforcers
   c. Record audit trail
   ↓
5. Return Result
```

## Built-in Policies

### Security Policies

| Policy ID | Description | Severity |
|-----------|-------------|----------|
| `no-secrets` | No hardcoded secrets | Critical |
| `no-plaintext-passwords` | No plaintext passwords | Critical |
| `mfa-required` | Multi-factor auth required | High |
| `dependency-scan` | No known vulnerable dependencies | High |

### Quality Policies

| Policy ID | Description | Severity |
|-----------|-------------|----------|
| `test-coverage` | Minimum test coverage | Medium |
| `no-todos` | No TODO comments | Low |
| `code-format` | Code formatting compliance | Low |
| `lint-clean` | No linting errors | Medium |

### Process Policies

| Policy ID | Description | Severity |
|-----------|-------------|----------|
| `pr-review` | PR requires review | High |
| `ci-pass` | CI must pass | High |
| `signed-commits` | Commits must be signed | Medium |

## Configuration Schema

```toml
# governance.toml
[governance]
version = "1.0"
enabled = true

[policies]
dir = "./policies"
auto_reload = true

[enforcement]
default = "warn"
override_by_category = true

[enforcement.categories]
security = "block"
compliance = "block"
quality = "warn"
process = "audit"

[output]
format = "table"
quiet = false
color = true
show_passed = false

[cache]
enabled = true
ttl = 300

[audit]
enabled = true
destination = "file"  # file | webhook | database
path = "./audit.log"
```

## API Design

```protobuf
service GovernanceService {
    rpc Evaluate(EvaluateRequest) returns (EvaluateResponse);
    rpc BatchEvaluate(BatchEvaluateRequest) returns (BatchEvaluateResponse);
    rpc GetPolicy(GetPolicyRequest) returns (GetPolicyResponse);
    rpc ListPolicies(ListPoliciesRequest) returns (ListPoliciesResponse);
    rpc UpdatePolicy(UpdatePolicyRequest) returns (UpdatePolicyResponse);
    rpc DeletePolicy(DeletePolicyRequest) returns (DeletePolicyResponse);
    rpc GetComplianceReport(GetComplianceReportRequest) returns (GetComplianceReportResponse);
}

message EvaluateRequest {
    string target = 1;
    repeated string policy_ids = 2;
}

message EvaluateResponse {
    EvaluationResult result = 1;
    repeated Violation violations = 2;
}

message Violation {
    string policy_id = 1;
    string rule_id = 2;
    string message = 3;
    Severity severity = 4;
    Location location = 5;
    optional AutoFix autofix = 6;
}
```

## Performance Requirements

| Operation | Target |
|-----------|--------|
| Policy load | <100ms |
| Single file evaluation | <50ms |
| Directory evaluation | <500ms |
| Git commit evaluation | <100ms |
| Policy cache hit | <1ms |

## Security Considerations

1. **Policy Injection**: Validate all policies before loading
2. **Regex DoS**: Use regex timeouts
3. **Path Traversal**: Sanitize all file paths
4. **Audit Integrity**: Cryptographically signed audit logs
5. **Secret Scanning**: Built-in patterns for common secrets

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_no_secrets_policy() {
        let engine = PolicyEngine::new();
        engine.load_policy("./fixtures/no-secrets.toml").await.unwrap();
        
        let target = Target::File {
            path: "test.rs".into(),
            content: r#"let api_key = "secret123456789";"#.into(),
        };
        
        let result = engine.evaluate(&target).await;
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].rule_id, "detect-api-keys");
    }
}
```

## References

- [Open Policy Agent](https://www.openpolicyagent.org/)
- [Rego Language](https://www.openpolicyagent.org/docs/latest/policy-language/)
- [Conftest](https://conftest.dev/)
- [Semgrep](https://semgrep.dev/)
