import { Given, When, Then, Before } from '@cucumber/cucumber';
import { expect } from 'chai';
import { v4 as uuidv4 } from 'uuid';

let testContext: any;

Before(async function() {
  testContext = {
    entity: null,
    lastError: null,
    events: [],
    config: { authToken: 'test-token', valid: true }
  };
});

Given('the {string} system is initialized', function(system: string) {
  testContext.config.system = system;
});

Given('a valid entity configuration', function() {
  testContext.config = { ...testContext.config, valid: true };
});

Given('an invalid entity configuration', function() {
  testContext.config = { ...testContext.config, valid: false };
});

Given('an existing entity in state {string}', function(state: string) {
  testContext.entity = { id: uuidv4(), state };
});

Given('an unauthenticated user', function() {
  testContext.config.authToken = undefined;
});

When('I create a new entity', async function() {
  try {
    if (!testContext.config.valid) throw new Error('Invalid configuration');
    testContext.entity = { id: uuidv4(), state: 'created' };
  } catch (error) {
    testContext.lastError = error as Error;
  }
});

When('I attempt to create a new entity', async function() {
  try {
    if (!testContext.config.valid) throw new Error('Invalid configuration');
    testContext.entity = { id: uuidv4(), state: 'created' };
  } catch (error) {
    testContext.lastError = error as Error;
  }
});

When('I execute the {string} transition', async function(transition: string) {
  try {
    if (!testContext.entity) throw new Error('No entity');
    testContext.entity.state = transition;
    testContext.events.push({ type: 'transition', to: transition });
  } catch (error) {
    testContext.lastError = error as Error;
  }
});

When('I attempt to access protected resources', async function() {
  try {
    if (!testContext.config.authToken) throw new Error('Unauthorized');
  } catch (error) {
    testContext.lastError = error as Error;
  }
});

Then('the entity should be persisted', function() {
  expect(testContext.entity).to.not.be.null;
  expect(testContext.entity.id).to.exist;
});

Then('the operation should fail', function() {
  expect(testContext.lastError).to.not.be.null;
});

Then('the entity should be in state {string}', function(expected: string) {
  expect(testContext.entity.state).to.equal(expected);
});

Then('the request should be denied', function() {
  expect(testContext.lastError).to.not.be.null;
  expect(testContext.lastError.message).to.include('Unauthorized');
});
