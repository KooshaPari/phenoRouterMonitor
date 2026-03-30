//! Integration tests for phenotype-port-traits.

use async_trait::async_trait;
use phenotype_port_traits::inbound::{CommandHandler, QueryHandler, UseCase, UseCaseError, UseCaseResult};
use phenotype_port_traits::models::{Entity, ValueObject};
use phenotype_port_traits::outbound::{Repository, Logger, PortResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Eq, PartialEq)]
struct UserId(String);

#[derive(Clone, Debug, Eq, PartialEq)]
struct Email {
    value: String,
}

impl ValueObject for Email {
    fn validate(&self) -> Result<(), String> {
        if self.value.contains('@') && !self.value.is_empty() {
            Ok(())
        } else {
            Err("Invalid email".to_string())
        }
    }
}

#[derive(Clone, Debug)]
struct User {
    id: UserId,
    email: Email,
    name: String,
}

impl Entity for User {
    type Id = UserId;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

struct InMemoryUserRepository {
    storage: Arc<Mutex<HashMap<UserId, User>>>,
}

#[async_trait]
impl Repository for InMemoryUserRepository {
    type Aggregate = User;
    type Id = UserId;

    async fn save(&self, aggregate: Self::Aggregate) -> PortResult<()> {
        let mut storage = self.storage.lock().unwrap();
        storage.insert(aggregate.id.clone(), aggregate);
        Ok(())
    }

    async fn find(&self, id: &Self::Id) -> PortResult<Self::Aggregate> {
        let storage = self.storage.lock().unwrap();
        storage
            .get(id)
            .cloned()
            .ok_or_else(|| phenotype_port_traits::PortError::NotFound(format!("User not found: {}", id.0)))
    }

    async fn delete(&self, id: &Self::Id) -> PortResult<()> {
        let mut storage = self.storage.lock().unwrap();
        if storage.remove(id).is_none() {
            Err(phenotype_port_traits::PortError::NotFound(format!("User not found: {}", id.0)))
        } else {
            Ok(())
        }
    }

    async fn exists(&self, id: &Self::Id) -> PortResult<bool> {
        let storage = self.storage.lock().unwrap();
        Ok(storage.contains_key(id))
    }
}

struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn error(&self, message: &str, _context: Option<&[(&str, &str)]>) {
        println!("[ERROR] {}", message);
    }

    fn warn(&self, message: &str, _context: Option<&[(&str, &str)]>) {
        println!("[WARN] {}", message);
    }

    fn info(&self, message: &str, _context: Option<&[(&str, &str)]>) {
        println!("[INFO] {}", message);
    }

    fn debug(&self, message: &str, _context: Option<&[(&str, &str)]>) {
        println!("[DEBUG] {}", message);
    }

    fn trace(&self, message: &str, _context: Option<&[(&str, &str)]>) {
        println!("[TRACE] {}", message);
    }
}

#[derive(Clone)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Clone)]
struct UserResponse {
    id: String,
    name: String,
    email: String,
}

struct CreateUserUseCase {
    repository: Arc<dyn Repository<Aggregate = User, Id = UserId>>,
}

#[async_trait]
impl UseCase for CreateUserUseCase {
    type Request = CreateUserRequest;
    type Response = UserResponse;

    async fn execute(&self, req: Self::Request) -> UseCaseResult<Self::Response> {
        let email = Email {
            value: req.email.clone(),
        };
        email
            .validate()
            .map_err(|e| UseCaseError::ValidationFailed(e))?;

        let user = User {
            id: UserId("user-1".to_string()),
            email,
            name: req.name.clone(),
        };

        self.repository.save(user.clone()).await.map_err(|e| {
            UseCaseError::Internal(format!("Failed to save user: {}", e))
        })?;

        Ok(UserResponse {
            id: user.id.0,
            name: user.name,
            email: user.email.value,
        })
    }
}

#[tokio::test]
async fn test_create_user_use_case() {
    let repository = Arc::new(InMemoryUserRepository {
        storage: Arc::new(Mutex::new(HashMap::new())),
    });

    let use_case = CreateUserUseCase {
        repository: repository.clone() as Arc<dyn Repository<Aggregate = User, Id = UserId>>,
    };

    let request = CreateUserRequest {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    let response = use_case.execute(request).await;
    assert!(response.is_ok());

    let user_response = response.unwrap();
    assert_eq!(user_response.name, "Alice");
    assert_eq!(user_response.email, "alice@example.com");
}

#[tokio::test]
async fn test_repository_operations() {
    let repository = InMemoryUserRepository {
        storage: Arc::new(Mutex::new(HashMap::new())),
    };

    let user_id = UserId("user-1".to_string());
    let user = User {
        id: user_id.clone(),
        email: Email {
            value: "test@example.com".to_string(),
        },
        name: "Test User".to_string(),
    };

    assert!(repository.save(user.clone()).await.is_ok());
    assert!(repository.exists(&user_id).await.unwrap());

    let found = repository.find(&user_id).await;
    assert!(found.is_ok());
    assert_eq!(found.unwrap().name, "Test User");

    assert!(repository.delete(&user_id).await.is_ok());
    assert!(!repository.exists(&user_id).await.unwrap());
}

#[test]
fn test_logger() {
    let logger = ConsoleLogger;
    logger.info("Test message", None);
}

#[test]
fn test_value_object_validation() {
    let valid = Email {
        value: "test@example.com".to_string(),
    };
    assert!(valid.validate().is_ok());

    let invalid = Email {
        value: "invalid".to_string(),
    };
    assert!(invalid.validate().is_err());
}
