Feature: Implementation workflow
  As a developer using AgilePlus
  I want to dispatch agents to implement work packages
  So that code is written according to the plan

  @FR-004 @smoke
  Scenario: FR-004 - Implement dispatches agent to worktree
    Given a feature "test-feature" in state "planned" with 2 work packages
    When I run "agileplus implement" for feature "test-feature"
    Then a worktree is created for WP01
    And an agent is dispatched with the WP01 prompt
    And an audit entry records the "planned -> implementing" transition

  @FR-010
  Scenario: FR-010 - Implement creates PR with WP context
    Given a feature "test-feature" with WP01 in state "doing"
    And the agent has committed code in the WP01 worktree
    When the agent completes WP01 implementation
    Then a PR is created with title containing "WP01"
    And the PR body contains the WP goal and FR references

  @FR-038
  Scenario: FR-038 - Non-overlapping WPs run in parallel
    Given a feature with WP01 file_scope "src/a.rs" and WP02 file_scope "src/b.rs"
    When I run "agileplus implement" for the feature
    Then WP01 and WP02 are dispatched concurrently

  @FR-039
  Scenario: FR-039 - Overlapping WPs are serialized
    Given a feature with WP01 file_scope "src/a.rs" and WP02 file_scope "src/a.rs"
    When I run "agileplus implement" for the feature
    Then WP02 waits until WP01 completes before starting
