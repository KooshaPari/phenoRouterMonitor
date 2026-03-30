//! Integration tests for phenotype-contracts.
//! @trace FR-ARCH-001 (Hexagonal Architecture Ports)

use phenotype_contracts::{
    UseCase, CommandHandler, QueryHandler, EventHandler,
    Repository, DomainEvent, Result,
};
use phenotype_contracts::error::ErrorKind;
use std::collections::HashMap;
use std::sync::Mutex;

struct TestUseCase;
impl UseCase for TestUseCase {
    type Request = String;
    type Response = String;
    fn execute(&self, request: Self::Request) -> Result<Self::Response> {
        Ok(format!("processed: {request}"))
    }
}

#[test]
fn test_use_case_port() {
    assert_eq!(TestUseCase.execute("hello".into()).unwrap(), "processed: hello");
}

struct TestCommandHandler { invoked: Mutex<bool> }
impl CommandHandler for TestCommandHandler {
    type Command = String;
    fn handle(&self, _cmd: Self::Command) -> Result<()> {
        *self.invoked.lock().unwrap() = true;
        Ok(())
    }
}

#[test]
fn test_command_handler_port() {
    let h = TestCommandHandler { invoked: Mutex::new(false) };
    h.handle("cmd".into()).unwrap();
    assert!(*h.invoked.lock().unwrap());
}

struct TestQueryHandler;
impl QueryHandler for TestQueryHandler {
    type Query = String;
    type Output = Vec<String>;
    fn handle(&self, q: Self::Query) -> Result<Self::Output> { Ok(vec![q]) }
}

#[test]
fn test_query_handler_port() {
    assert_eq!(TestQueryHandler.handle("search".into()).unwrap(), vec!["search".to_string()]);
}

struct TestEventHandler { events: Mutex<Vec<String>> }
impl EventHandler for TestEventHandler {
    type Event = String;
    fn handle(&self, event: Self::Event) -> Result<()> {
        self.events.lock().unwrap().push(event); Ok(())
    }
}

#[test]
fn test_event_handler_port() {
    let h = TestEventHandler { events: Mutex::new(vec![]) };
    h.handle("e1".into()).unwrap();
    h.handle("e2".into()).unwrap();
    assert_eq!(h.events.lock().unwrap().len(), 2);
}

struct InMemoryRepo { store: Mutex<HashMap<String, String>> }
impl Repository for InMemoryRepo {
    type Entity = String;
    type Id = String;
    fn save(&self, id: Self::Id, entity: Self::Entity) -> Result<()> {
        self.store.lock().unwrap().insert(id, entity); Ok(())
    }
    fn get(&self, id: &Self::Id) -> Result<Self::Entity> {
        self.store.lock().unwrap().get(id).cloned()
            .ok_or_else(|| ErrorKind::not_found(format!("entity {id}")))
    }
    fn delete(&self, id: &Self::Id) -> Result<()> {
        self.store.lock().unwrap().remove(id); Ok(())
    }
    fn list(&self) -> Result<Vec<Self::Entity>> {
        Ok(self.store.lock().unwrap().values().cloned().collect())
    }
}

#[test]
fn test_repository_crud() {
    let repo = InMemoryRepo { store: Mutex::new(HashMap::new()) };
    repo.save("1".into(), "entity-1".into()).unwrap();
    assert_eq!(repo.get(&"1".into()).unwrap(), "entity-1");
    assert_eq!(repo.list().unwrap().len(), 1);
    repo.delete(&"1".into()).unwrap();
    assert!(repo.get(&"1".into()).is_err());
}

#[test]
fn test_domain_event_creation() {
    let event = DomainEvent::new("agg-123".into(), "UserCreated".into(), serde_json::json!({"name": "Alice"}));
    assert_eq!(event.aggregate_id, "agg-123");
    assert_eq!(event.event_type, "UserCreated");
    assert!(!event.id.is_nil());
}
