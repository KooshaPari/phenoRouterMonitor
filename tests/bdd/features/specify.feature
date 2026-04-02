Feature: Specification workflow
  As a developer using AgilePlus
  I want to create and refine feature specifications
  So that my development work is well-defined and traceable

  @FR-001 @smoke
  Scenario: FR-001 - Specify creates spec in git and SQLite
    Given a fresh AgilePlus project with no features
    When I run "agileplus specify" with feature slug "test-feature"
    And I provide specification details via stdin
    Then a spec.md file exists at "kitty-specs/test-feature/spec.md"
    And the feature "test-feature" exists in SQLite with state "specified"
    And an audit entry records the "created -> specified" transition

  @FR-008
  Scenario: FR-008 - Re-running specify triggers refinement
    Given a feature "test-feature" in state "specified"
    When I run "agileplus specify" with feature slug "test-feature"
    And I provide updated specification details
    Then the spec.md file is updated with a new spec_hash
    And an audit entry records a "refinement" event with diff reference

  @FR-033
  Scenario: FR-033 - Specify rejects feature in wrong state
    Given a feature "test-feature" in state "implementing"
    When I run "agileplus specify" with feature slug "test-feature"
    Then the command fails with an invalid state error
    And the feature state remains "implementing"

  @FR-001
  Scenario: FR-001 - Specify stores spec hash for content integrity
    Given a fresh AgilePlus project with no features
    When I run "agileplus specify" with feature slug "hash-test-feature"
    And I provide specification details via stdin
    Then the feature "hash-test-feature" exists in SQLite with state "specified"
    And the stored spec_hash is a 64-character hex string
