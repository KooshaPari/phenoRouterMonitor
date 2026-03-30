//! JSON Schema generation from Phenotype models and port traits.

use serde_json::{json, Value};
use std::collections::HashMap;

/// Generates JSON Schema definitions for Phenotype models.
pub struct JsonSchemaGenerator;

impl JsonSchemaGenerator {
    /// Generates complete JSON Schema definitions.
    pub fn generate() -> HashMap<String, Value> {
        let mut schemas = HashMap::new();

        // DomainEvent schema
        schemas.insert(
            "DomainEvent".to_string(),
            json!({
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object",
                "title": "DomainEvent",
                "description": "Domain event representing a state change in the system",
                "properties": {
                    "id": {
                        "type": "string",
                        "format": "uuid",
                        "description": "Unique event identifier"
                    },
                    "aggregate_id": {
                        "type": "string",
                        "description": "Aggregate ID that generated the event"
                    },
                    "event_type": {
                        "type": "string",
                        "description": "Type of the event"
                    },
                    "timestamp": {
                        "type": "string",
                        "format": "date-time",
                        "description": "When the event occurred (ISO 8601)"
                    },
                    "data": {
                        "type": "object",
                        "description": "Event payload data"
                    }
                },
                "required": ["id", "aggregate_id", "event_type", "timestamp", "data"],
                "additionalProperties": false
            }),
        );

        // UseCase interface schema
        schemas.insert(
            "UseCase".to_string(),
            json!({
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object",
                "title": "UseCase",
                "description": "Use case port for executing business operations",
                "properties": {
                    "execute": {
                        "type": "object",
                        "description": "Execute the use case"
                    }
                }
            }),
        );

        // CommandHandler interface schema
        schemas.insert(
            "CommandHandler".to_string(),
            json!({
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object",
                "title": "CommandHandler",
                "description": "Command handler port for processing commands",
                "properties": {
                    "handle": {
                        "type": "object",
                        "description": "Handle a command"
                    }
                }
            }),
        );

        // QueryHandler interface schema
        schemas.insert(
            "QueryHandler".to_string(),
            json!({
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object",
                "title": "QueryHandler",
                "description": "Query handler port for processing queries",
                "properties": {
                    "handle": {
                        "type": "object",
                        "description": "Handle a query"
                    }
                }
            }),
        );

        // Repository interface schema
        schemas.insert(
            "Repository".to_string(),
            json!({
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object",
                "title": "Repository",
                "description": "Repository port for persisting and retrieving domain entities",
                "properties": {
                    "save": { "type": "object" },
                    "get": { "type": "object" },
                    "delete": { "type": "object" },
                    "list": { "type": "array" }
                }
            }),
        );

        // CachePort interface schema
        schemas.insert(
            "CachePort".to_string(),
            json!({
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object",
                "title": "CachePort",
                "description": "Cache port for storing and retrieving cached values",
                "properties": {
                    "get": { "type": "object" },
                    "set": { "type": "object" },
                    "invalidate": { "type": "object" }
                }
            }),
        );

        // EventBus interface schema
        schemas.insert(
            "EventBus".to_string(),
            json!({
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object",
                "title": "EventBus",
                "description": "Event bus port for publishing and subscribing to domain events",
                "properties": {
                    "publish": { "type": "object" },
                    "publish_batch": { "type": "object" }
                }
            }),
        );

        // SecretManager interface schema
        schemas.insert(
            "SecretManager".to_string(),
            json!({
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object",
                "title": "SecretManager",
                "description": "Secret manager port for secure credential storage and retrieval",
                "properties": {
                    "get": { "type": "object" },
                    "set": { "type": "object" },
                    "delete": { "type": "object" }
                }
            }),
        );

        // EventHandler interface schema
        schemas.insert(
            "EventHandler".to_string(),
            json!({
                "$schema": "http://json-schema.org/draft-07/schema#",
                "type": "object",
                "title": "EventHandler",
                "description": "Event handler port for processing domain events",
                "properties": {
                    "handle": { "type": "object" }
                }
            }),
        );

        schemas
    }
}
