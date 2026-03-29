//! # Tests for Phenotype Contracts

#[cfg(test)]
mod tests {
    use crate::models::{Entity, ValueObject, EntityExt};
    use crate::ports::outbound::{CachePort, Error};

    // Test Value Object
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Email(String);

    impl ValueObject for Email {
        fn validate(&self) -> Result<(), String> {
            if self.0.contains('@') {
                Ok(())
            } else {
                Err("Invalid email".to_string())
            }
        }
    }

    #[test]
    fn test_value_object_validation() {
        let valid = Email("test@example.com".to_string());
        assert!(valid.validate().is_ok());

        let invalid = Email("invalid".to_string());
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_value_object_equality() {
        let email1 = Email("test@example.com".to_string());
        let email2 = Email("test@example.com".to_string());
        let email3 = Email("other@example.com".to_string());

        assert_eq!(email1, email2);
        assert_ne!(email1, email3);
    }

    // Test Entity
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct UserId(String);

    #[derive(Debug, Clone)]
    struct User {
        id: UserId,
        _name: String,
    }

    impl Entity for User {
        type Id = UserId;

        fn id(&self) -> &Self::Id {
            &self.id
        }
    }

    #[test]
    fn test_entity_identity() {
        let user1 = User {
            id: UserId("user-1".to_string()),
            _name: "Alice".to_string(),
        };

        let user2 = User {
            id: UserId("user-1".to_string()),
            _name: "Alice (updated)".to_string(),
        };

        let user3 = User {
            id: UserId("user-2".to_string()),
            _name: "Bob".to_string(),
        };

        // Same ID = same entity
        assert!(user1.is_same(&user2));

        // Different ID = different entity
        assert!(!user1.is_same(&user3));
    }

    #[test]
    fn test_entity_ext() {
        let user = User {
            id: UserId("user-1".to_string()),
            _name: "Alice".to_string(),
        };

        let id_ref = user.id_ref();
        assert_eq!(id_ref, UserId("user-1".to_string()));
    }

    // Mock cache for testing
    struct MockCache;

    #[async_trait::async_trait]
    impl CachePort for MockCache {
        async fn get(&self, _key: &str) -> Result<Option<String>, Error> {
            Ok(Some("value".to_string()))
        }

        async fn set(&self, _key: &str, _value: &str, _ttl: std::time::Duration) -> Result<(), Error> {
            Ok(())
        }

        async fn set_nx(&self, _key: &str, _value: &str, _ttl: std::time::Duration) -> Result<bool, Error> {
            Ok(true)
        }

        async fn delete(&self, _key: &str) -> Result<(), Error> {
            Ok(())
        }

        async fn exists(&self, _key: &str) -> Result<bool, Error> {
            Ok(true)
        }

        async fn expire(&self, _key: &str, _ttl: std::time::Duration) -> Result<(), Error> {
            Ok(())
        }

        async fn ttl(&self, _key: &str) -> Result<Option<std::time::Duration>, Error> {
            Ok(Some(std::time::Duration::from_secs(3600)))
        }

        async fn ping(&self) -> Result<(), Error> {
            Ok(())
        }

        async fn close(&self) -> Result<(), Error> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_mock_cache_get() {
        let cache = MockCache;
        let result = cache.get("test-key").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("value".to_string()));
    }

    #[tokio::test]
    async fn test_mock_cache_set() {
        let cache = MockCache;
        let result = cache.set("test-key", "value", std::time::Duration::from_secs(60)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_cache_exists() {
        let cache = MockCache;
        let result = cache.exists("test-key").await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
