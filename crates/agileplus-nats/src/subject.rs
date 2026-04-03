//! Hierarchical subject (topic) addressing for the event bus.
//!
//! Subjects follow the NATS dot-separated convention:
//!   `agileplus.feature.42.state_transitioned`
//!   `agileplus.wp.7.created`
//!
//! Wildcards:
//! - `*` matches a single token  (`agileplus.feature.*.created`)
//! - `>` matches one or more tokens (`agileplus.feature.>`)

use std::fmt;

/// A validated subject string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Subject(String);

impl Subject {
    /// Build a subject from dot-separated tokens.
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    /// Convenience: `{prefix}.{entity_type}.{entity_id}.{event_type}`
    pub fn for_event(prefix: &str, entity_type: &str, entity_id: i64, event_type: &str) -> Self {
        Self(format!("{prefix}.{entity_type}.{entity_id}.{event_type}"))
    }

    /// Wildcard subject matching all events for an entity type:
    /// `{prefix}.{entity_type}.>`
    pub fn all_for_entity(prefix: &str, entity_type: &str) -> Self {
        Self(format!("{prefix}.{entity_type}.>"))
    }

    /// Wildcard subject matching a specific event type across all entities:
    /// `{prefix}.{entity_type}.*.{event_type}`
    pub fn all_of_type(prefix: &str, entity_type: &str, event_type: &str) -> Self {
        Self(format!("{prefix}.{entity_type}.*.{event_type}"))
    }

    /// Return the raw subject string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check whether this subject matches a concrete (non-wildcard) subject.
    pub fn matches(&self, concrete: &Subject) -> bool {
        let pat_tokens: Vec<&str> = self.0.split('.').collect();
        let sub_tokens: Vec<&str> = concrete.0.split('.').collect();
        matches_tokens(&pat_tokens, &sub_tokens)
    }
}

fn matches_tokens(pattern: &[&str], subject: &[&str]) -> bool {
    if pattern.is_empty() && subject.is_empty() {
        return true;
    }
    if pattern.is_empty() {
        return false;
    }
    if pattern[0] == ">" {
        return !subject.is_empty();
    }
    if subject.is_empty() {
        return false;
    }
    if pattern[0] == "*" || pattern[0] == subject[0] {
        return matches_tokens(&pattern[1..], &subject[1..]);
    }
    false
}

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match() {
        let s = Subject::new("agileplus.feature.1.created");
        assert!(s.matches(&Subject::new("agileplus.feature.1.created")));
        assert!(!s.matches(&Subject::new("agileplus.feature.2.created")));
    }

    #[test]
    fn star_wildcard() {
        let s = Subject::all_of_type("agileplus", "feature", "created");
        assert!(s.matches(&Subject::new("agileplus.feature.1.created")));
        assert!(s.matches(&Subject::new("agileplus.feature.99.created")));
        assert!(!s.matches(&Subject::new("agileplus.feature.1.deleted")));
    }

    #[test]
    fn chevron_wildcard() {
        let s = Subject::all_for_entity("agileplus", "feature");
        assert!(s.matches(&Subject::new("agileplus.feature.1.created")));
        assert!(s.matches(&Subject::new("agileplus.feature.1.state_transitioned")));
        assert!(!s.matches(&Subject::new("agileplus.wp.1.created")));
    }

    #[test]
    fn for_event_builds_correct_subject() {
        let s = Subject::for_event("agileplus", "feature", 42, "state_transitioned");
        assert_eq!(s.as_str(), "agileplus.feature.42.state_transitioned");
    }
}
