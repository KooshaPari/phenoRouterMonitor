Feature: Entity domain operations
  As a system user
  I want to manage entities through their lifecycle
  So that I can track state transitions and ensure data integrity

  Background:
    Given the entity system is initialized

  @FR-001 @smoke @critical
  Scenario: Creating a new entity with valid configuration
    Given a valid entity configuration
    When I create a new entity
    Then the entity should be persisted
    And the entity should be in state "created"

  @FR-002 @negative
  Scenario: Attempting to create entity with invalid configuration
    Given an invalid entity configuration
    When I attempt to create a new entity
    Then the operation should fail

  @FR-003 @integration
  Scenario: Transitioning entity through workflow states
    Given an existing entity in state "created"
    When I execute the "process" transition
    Then the entity should be in state "processing"
    When I execute the "complete" transition
    Then the entity should be in state "completed"

  @FR-004 @security
  Scenario: Unauthorized access attempt
    Given an unauthenticated user
    When I attempt to access protected resources
    Then the request should be denied

  @FR-005 @performance
  Scenario: Concurrent entity operations
    Given 100 concurrent entity creation requests
    When all requests are processed
    Then all entities should be persisted successfully
    And no data corruption should occur
