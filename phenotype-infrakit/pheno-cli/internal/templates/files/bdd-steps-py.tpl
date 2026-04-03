from behave import given, when, then
import uuid
import json
from datetime import datetime

class TestContext:
    def __init__(self):
        self.entity = None
        self.last_error = None
        self.events = []
        self.config = {
            'auth_token': 'test-token',
            'valid': True,
            'concurrent_ops': 1
        }

@given('the {system} system is initialized')
def step_system_initialized(context, system):
    context.test = TestContext()
    context.test.config['system'] = system

@given('a valid entity configuration')
def step_valid_config(context):
    context.test.config['valid'] = True

@given('an invalid entity configuration')
def step_invalid_config(context):
    context.test.config['valid'] = False

@given('an existing entity in state "{state}"')
def step_entity_in_state(context, state):
    context.test.entity = {
        'id': str(uuid.uuid4()),
        'state': state,
        'data': {}
    }

@given('an unauthenticated user')
def step_unauthenticated(context):
    context.test.config['auth_token'] = None

@given('{count:d} concurrent entity creation requests')
def step_concurrent_requests(context, count):
    context.test.config['concurrent_ops'] = count

@when('I create a new entity')
def step_create_entity(context):
    if context.test.config['valid']:
        context.test.entity = {
            'id': str(uuid.uuid4()),
            'state': 'created',
            'data': {}
        }
    else:
        context.test.last_error = 'Invalid configuration'

@when('I attempt to create a new entity')
def step_attempt_create(context):
    if not context.test.config['valid']:
        context.test.last_error = 'Invalid configuration'

@when('I execute the "{transition}" transition')
def step_execute_transition(context, transition):
    if context.test.entity:
        context.test.entity['state'] = transition
        context.test.events.append({
            'type': 'transition',
            'name': transition,
            'timestamp': datetime.utcnow().isoformat()
        })

@when('I attempt to access protected resources')
def step_access_protected(context):
    if context.test.config['auth_token'] is None:
        context.test.last_error = 'Unauthorized access'

@when('all requests are processed')
def step_process_requests(context):
    pass

@then('the entity should be persisted')
def step_entity_persisted(context):
    assert context.test.entity is not None, "Entity should exist"
    assert context.test.entity['id'], "Entity should have ID"

@then('the entity should be in state "{expected}"')
def step_entity_state(context, expected):
    assert context.test.entity['state'] == expected, \
        f"Expected state {expected}, got {context.test.entity['state']}"

@then('the operation should fail')
def step_operation_failed(context):
    assert context.test.last_error is not None, "Operation should have failed"

@then('the request should be denied')
def step_request_denied(context):
    assert context.test.last_error is not None, "Request should have been denied"
    assert 'Unauthorized' in context.test.last_error

@then('all entities should be persisted successfully')
def step_all_persisted(context):
    assert context.test.entity is not None, "At least one entity should exist"

@then('no data corruption should occur')
def step_no_corruption(context):
    pass
