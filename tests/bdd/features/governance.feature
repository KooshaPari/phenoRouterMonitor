Feature: Governance enforcement
  As a project with quality standards
  I want governance contracts to enforce evidence requirements
  So that no feature ships without proper verification

  @FR-018 @smoke
  Scenario: FR-018 - Governance contract binds to feature
    Given a feature "test-feature" in state "researched"
    When I run "agileplus plan" for feature "test-feature"
    Then a governance contract is created for the feature
    And the contract contains rules for each state transition

  @FR-019
  Scenario: FR-019 - Validate checks governance compliance
    Given a feature "test-feature" in state "implementing"
    And the governance contract requires test_result evidence for FR-001
    And evidence exists for FR-001 with type "test_result"
    When I run "agileplus validate" for feature "test-feature"
    Then validation passes
    And the feature transitions to "validated"

  @FR-019 @negative
  Scenario: FR-019 - Validate blocks on missing evidence
    Given a feature "test-feature" in state "implementing"
    And the governance contract requires test_result evidence for FR-001
    And no evidence exists for FR-001
    When I run "agileplus validate" for feature "test-feature"
    Then validation fails
    And the report shows FR-001 evidence is missing
    And the feature state remains "implementing"

  @FR-019
  Scenario: FR-019 - Validate passes when all evidence present
    Given a feature "multi-evidence-feature" in state "implementing"
    And the governance contract requires test_result evidence for FR-001
    And the governance contract requires review_approval evidence for FR-002
    And evidence exists for FR-001 with type "test_result"
    And evidence exists for FR-002 with type "review_approval"
    When I run "agileplus validate" for feature "multi-evidence-feature"
    Then validation passes
    And the feature transitions to "validated"
