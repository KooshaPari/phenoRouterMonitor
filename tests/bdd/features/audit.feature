Feature: Audit trail integrity
  As an auditor
  I want a tamper-evident hash-chained audit log
  So that I can verify the complete history of a feature

  @FR-016 @smoke
  Scenario: FR-016 - Audit entries form a hash chain
    Given a feature "test-feature" with 5 audit entries
    When I verify the audit chain for "test-feature"
    Then all entries have valid hash linkage
    And the verification returns success with count 5

  @FR-016 @negative
  Scenario: FR-016 - Tampered audit entry detected
    Given a feature "test-feature" with 5 audit entries
    And audit entry 3 has been tampered with
    When I verify the audit chain for "test-feature"
    Then verification fails at entry 3
    And the error identifies the hash mismatch

  @FR-016
  Scenario: FR-016 - Audit records every state transition
    Given a fresh AgilePlus project with no features
    When I run "agileplus specify" with feature slug "audit-test"
    And I provide specification details via stdin
    Then the audit trail for "audit-test" contains 1 entry
    And the first entry has transition "created -> specified"

  @FR-016
  Scenario: FR-016 - Empty audit chain returns error
    Given a feature "empty-feature" with 0 audit entries
    When I verify the audit chain for "empty-feature"
    Then verification fails with empty chain error
