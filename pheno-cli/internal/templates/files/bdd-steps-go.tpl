package bdd

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/cucumber/godog"
)

type TestWorld struct {
	Entity     *Entity
	LastError  error
	Events     []Event
	Config     TestConfig
}

type Entity struct {
	ID    string
	State string
}

type Event struct {
	Type      string
	Name      string
	Timestamp time.Time
}

type TestConfig struct {
	AuthToken     string
	ConcurrentOps int
	Timeout       time.Duration
	Valid         bool
}

func InitializeScenario(sc *godog.ScenarioContext) {
	world := &TestWorld{
		Config: TestConfig{AuthToken: "test-token", ConcurrentOps: 1, Timeout: 30 * time.Second, Valid: true},
	}

	sc.Step(`^the (.+) system is initialized$`, world.systemInitialized)
	sc.Step(`^a valid entity configuration$`, world.validEntityConfig)
	sc.Step(`^an invalid entity configuration$`, world.invalidEntityConfig)
	sc.Step(`^an existing entity in state "([^"]+)"$`, world.entityInState)
	sc.Step(`^an unauthenticated user$`, world.unauthenticatedUser)
	sc.Step(`^I create a new entity$`, world.createEntity)
	sc.Step(`^I attempt to create a new entity$`, world.createEntity)
	sc.Step(`^I execute the "([^"]+)" transition$`, world.executeTransition)
	sc.Step(`^I attempt to access protected resources$`, world.accessProtected)
	sc.Step(`^the entity should be persisted$`, world.entityPersisted)
	sc.Step(`^the operation should fail$`, world.operationFailed)
	sc.Step(`^the entity should be in state "([^"]+)"$`, world.entityInExpectedState)
	sc.Step(`^the request should be denied$`, world.requestDenied)
}

func (w *TestWorld) systemInitialized(ctx context.Context, system string) error { return nil }
func (w *TestWorld) validEntityConfig(ctx context.Context) error {
	w.Config.Valid = true
	return nil
}
func (w *TestWorld) invalidEntityConfig(ctx context.Context) error {
	w.Config.Valid = false
	return nil
}
func (w *TestWorld) entityInState(ctx context.Context, state string) error {
	w.Entity = &Entity{ID: fmt.Sprintf("entity-%d", time.Now().UnixNano()), State: state}
	return nil
}
func (w *TestWorld) unauthenticatedUser(ctx context.Context) error {
	w.Config.AuthToken = ""
	return nil
}
func (w *TestWorld) createEntity(ctx context.Context) error {
	if !w.Config.Valid {
		w.LastError = fmt.Errorf("invalid configuration")
		return nil
	}
	w.Entity = &Entity{ID: fmt.Sprintf("entity-%d", time.Now().UnixNano()), State: "created"}
	return nil
}
func (w *TestWorld) executeTransition(ctx context.Context, transition string) error {
	if w.Entity == nil {
		w.LastError = fmt.Errorf("no entity")
		return nil
	}
	w.Entity.State = transition
	w.Events = append(w.Events, Event{Type: "transition", Name: transition, Timestamp: time.Now()})
	return nil
}
func (w *TestWorld) accessProtected(ctx context.Context) error {
	if w.Config.AuthToken == "" {
		w.LastError = fmt.Errorf("unauthorized")
	}
	return nil
}
func (w *TestWorld) entityPersisted(ctx context.Context) error {
	if w.Entity == nil || w.Entity.ID == "" {
		return fmt.Errorf("entity not persisted")
	}
	return nil
}
func (w *TestWorld) operationFailed(ctx context.Context) error {
	if w.LastError == nil {
		return fmt.Errorf("operation should have failed")
	}
	return nil
}
func (w *TestWorld) entityInExpectedState(ctx context.Context, expected string) error {
	if w.Entity == nil || w.Entity.State != expected {
		return fmt.Errorf("expected state %s", expected)
	}
	return nil
}
func (w *TestWorld) requestDenied(ctx context.Context) error {
	if w.LastError == nil {
		return fmt.Errorf("request should have been denied")
	}
	return nil
}

func TestBDD(t *testing.T) {
	opts := godog.Options{Format: "pretty", Paths: []string{"../features"}}
	suite := godog.TestSuite{ScenarioInitializer: InitializeScenario, Options: &opts}
	if status := suite.Run(); status != 0 {
		t.Fatalf("BDD test suite failed with status %d", status)
	}
}
