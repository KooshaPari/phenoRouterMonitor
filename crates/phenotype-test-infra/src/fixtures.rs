//! Test fixture builders.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub active: bool,
}

impl Default for TestUser {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "test_user".to_string(),
            email: "test@example.com".to_string(),
            active: true,
        }
    }
}

pub struct UserBuilder {
    id: Option<String>,
    name: String,
    email: String,
    active: bool,
}

impl UserBuilder {
    pub fn new<S: Into<String>>(name: S) -> Self {
        let name_str = name.into();
        Self {
            id: None,
            name: name_str.clone(),
            email: format!("{}@example.com", name_str),
            active: true,
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = email.into();
        self
    }

    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn build(self) -> TestUser {
        TestUser {
            id: self.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            name: self.name,
            email: self.email,
            active: self.active,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestOrder {
    pub id: String,
    pub user_id: String,
    pub amount: f64,
    pub status: String,
}

impl Default for TestOrder {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id: Uuid::new_v4().to_string(),
            amount: 0.0,
            status: "pending".to_string(),
        }
    }
}

pub struct OrderBuilder {
    id: Option<String>,
    user_id: Option<String>,
    amount: f64,
    status: String,
}

impl Default for OrderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            user_id: None,
            amount: 0.0,
            status: "pending".to_string(),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn with_amount(mut self, amount: f64) -> Self {
        self.amount = amount;
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub fn build(self) -> TestOrder {
        TestOrder {
            id: self.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            user_id: self.user_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            amount: self.amount,
            status: self.status,
        }
    }
}
