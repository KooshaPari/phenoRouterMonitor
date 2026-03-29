//! # Tests for Phenotype Contracts

#[cfg(test)]
mod tests {
    use crate::models::{Entity, ValueObject, DomainEvent};
    use crate::ports::outbound::{CachePort, cache::CacheError};

    // Test Entity
    #[test]
    fn test_entity_creation() {
        let entity = Entity::new("user-123".to_string());
        assert_eq!(entity.id(), "user-123");
        assert!(!entity.is("other-id"));
        assert!(entity.is("user-123"));
    }

    #[test]
    fn test_entity_equality() {
        let entity1 = Entity::new("user-123".to_string());
        let entity2 = Entity::new("user-123".to_string());
        let entity3 = Entity::new("user-456".to_string());
        
        assert_eq!(entity1, entity2);
        assert_ne!(entity1, entity3);
    }

    #[test]
    fn test_entity_with_type() {
        let entity = Entity::with_type("order-123".to_string(), "Order");
        assert_eq!(entity.id(), "order-123");
        assert_eq!(entity.to_string(), "Order:order-123");
    }

    // Test Value Object
    #[test]
    fn test_value_object_creation() {
        let vo = ValueObject::new(serde_json::json!({
            "email": "test@example.com",
            "verified": true
        }));
        
        assert_eq!(vo.get("email").unwrap().as_str().unwrap(), "test@example.com");
        assert!(vo.get("verified").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_value_object_equality() {
        let vo1 = ValueObject::new(serde_json::json!({"name": "test"}));
        let vo2 = ValueObject::new(serde_json::json!({"name": "test"}));
        let vo3 = ValueObject::new(serde_json::json!({"name": "other"}));
        
        assert_eq!(vo1, vo2);
        assert_ne!(vo1, vo3);
    }

    #[test]
    fn test_value_object_json_roundtrip() {
        let vo = ValueObject::new(serde_json::json!({"key": "value"}));
        let json_str = vo.to_json();
        let vo_restored = ValueObject::from_json(&json_str).unwrap();
        assert_eq!(vo, vo_restored);
    }

    // Test Domain Event
    #[test]
    fn test_domain_event_creation() {
        let event = DomainEvent::new(
            "aggregate-123".to_string(),
            "OrderCreated".to_string(),
            serde_json::json!({"total": 100.0}),
            1,
        );
        
        assert_eq!(event.aggregate_id, "aggregate-123");
        assert_eq!(event.event_type, "OrderCreated");
        assert_eq!(event.version, 1);
        assert!(!event.event_id.is_empty());
    }

    #[test]
    fn test_domain_event_with_metadata() {
        let event = DomainEvent::new(
            "aggregate-123".to_string(),
            "OrderCreated".to_string(),
            serde_json::json!({"total": 100.0}),
            1,
        ).with_metadata("correlation_id", "corr-456")
         .with_metadata("causation_id", "cmd-789");
        
        assert_eq!(event.metadata.get("correlation_id").unwrap(), "corr-456");
        assert_eq!(event.metadata.get("causation_id").unwrap(), "cmd-789");
    }

    // Mock cache for testing
    struct MockCache;

    #[async_trait::async_trait]
    impl CachePort for MockCache {
        async fn get(&self, _key: &str) -> Result<Option<Vec<u8>>, CacheError> {
            Ok(Some(b"value".to_vec()))
        }

        async fn set(
            &self,
            _key: &str,
            _value: Vec<u8>,
            _ttl: Option<std::time::Duration>,
        ) -> Result<(), CacheError> {
            Ok(())
        }

        async fn delete(&self, _key: &str) -> Result<(), CacheError> {
            Ok(())
        }

        async fn exists(&self, _key: &str) -> Result<bool, CacheError> {
            Ok(true)
        }

        async fn clear(&self) -> Result<(), CacheError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_mock_cache_get() {
        let cache = MockCache;
        let result = cache.get("test-key").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(b"value".to_vec()));
    }

    #[tokio::test]
    async fn test_mock_cache_set() {
        let cache = MockCache;
        let result = cache.set("test-key", b"value".to_vec(), Some(std::time::Duration::from_secs(60))).await;
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
