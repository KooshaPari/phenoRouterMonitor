//! Integration tests for policy engine evaluation (simplified).
use phenotype_policy_engine::{
    EvaluationContext, Policy, PolicyEngine, Rule, RuleType,
};
use phenotype_policy_engine::policy::EvaluablePolicy;

fn create_context_with_facts(facts: Vec<(&str, &str)>) -> EvaluationContext {
    let mut ctx = EvaluationContext::new();
    for (key, value) in facts {
        ctx.set_string(key, value);
    }
    ctx
}

#[test]
fn test_allow_rule_pass() {
    let mut policy = Policy::new("test");
    policy.rules.push(Rule::new(RuleType::Allow, "role", "admin"));
    let ctx = create_context_with_facts(vec![("role", "admin")]);
    let result = policy.evaluate(&ctx).unwrap();
    assert!(result.passed);
}

#[test]
fn test_allow_rule_fail() {
    let mut policy = Policy::new("test");
    policy.rules.push(Rule::new(RuleType::Allow, "role", "admin"));
    let ctx = create_context_with_facts(vec![("role", "user")]);
    let result = policy.evaluate(&ctx).unwrap();
    assert!(!result.passed);
}

#[test]
fn test_deny_rule_pass() {
    let mut policy = Policy::new("test");
    policy.rules.push(Rule::new(RuleType::Deny, "role", "admin"));
    let ctx = create_context_with_facts(vec![("role", "user")]);
    let result = policy.evaluate(&ctx).unwrap();
    assert!(result.passed);
}

#[test]
fn test_require_rule_pass() {
    let mut policy = Policy::new("test");
    policy.rules.push(Rule::new(RuleType::Require, "email", ".*@example\\.com$"));
    let ctx = create_context_with_facts(vec![("email", "user@example.com")]);
    let result = policy.evaluate(&ctx).unwrap();
    assert!(result.passed);
}

#[test]
fn test_engine_add_and_retrieve() {
    let engine = PolicyEngine::new();
    let policy = Policy::new("test-policy");
    engine.add_policy(policy);
    assert_eq!(engine.policy_count(), 1);
}

#[test]
fn test_engine_remove_policy() {
    let engine = PolicyEngine::new();
    let policy = Policy::new("test-policy");
    engine.add_policy(policy);
    let removed = engine.remove_policy("test-policy");
    assert!(removed.is_some());
    assert_eq!(engine.policy_count(), 0);
}

#[test]
fn test_empty_policy() {
    let policy = Policy::new("empty");
    let ctx = EvaluationContext::new();
    let result = policy.evaluate(&ctx).unwrap();
    assert!(result.passed);
}

#[test]
fn test_disabled_policy() {
    let mut policy = Policy::new("disabled");
    policy.enabled = false;
    policy.rules.push(Rule::new(RuleType::Require, "field", "value"));
    let ctx = EvaluationContext::new();
    let result = policy.evaluate(&ctx).unwrap();
    assert!(result.passed);
}

#[test]
fn test_policy_names() {
    let engine = PolicyEngine::new();
    engine.add_policy(Policy::new("policy1"));
    engine.add_policy(Policy::new("policy2"));
    let names = engine.policy_names();
    assert_eq!(names.len(), 2);
}

#[test]
fn test_regex_email() {
    let mut policy = Policy::new("email");
    policy.rules.push(Rule::new(
        RuleType::Require,
        "email",
        r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
    ));

    let ctx = create_context_with_facts(vec![("email", "user@example.com")]);
    assert!(policy.evaluate(&ctx).unwrap().passed);

    let ctx = create_context_with_facts(vec![("email", "invalid")]);
    assert!(!policy.evaluate(&ctx).unwrap().passed);
}

#[test]
fn test_multiple_rules() {
    let mut policy = Policy::new("multi");
    policy.rules.push(Rule::new(RuleType::Allow, "role", "admin"));
    policy.rules.push(Rule::new(RuleType::Deny, "banned", "true"));

    let ctx = create_context_with_facts(vec![
        ("role", "admin"),
        ("banned", "false"),
    ]);
    assert!(policy.evaluate(&ctx).unwrap().passed);
}

#[test]
fn test_concurrent_policy_adds() {
    let engine = std::sync::Arc::new(PolicyEngine::new());
    let mut handles = vec![];

    for i in 0..10 {
        let engine_clone = std::sync::Arc::clone(&engine);
        let handle = std::thread::spawn(move || {
            engine_clone.add_policy(Policy::new(format!("policy-{}", i)));
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(engine.policy_count(), 10);
}
