//! OpenAPI 3.0.0 schema generation from Phenotype port traits.

use serde_json::{json, Value};

/// Generates OpenAPI 3.0.0 specification from Phenotype port traits.
pub struct OpenAPIGenerator;

impl OpenAPIGenerator {
    /// Generates a complete OpenAPI specification document.
    pub fn generate() -> Value {
        let mut spec = json!({
            "openapi": "3.0.0",
            "info": {
                "title": "Phenotype Contracts",
                "version": "1.0.0",
                "description": "Stable port contracts for the Phenotype hexagonal architecture",
                "contact": {
                    "name": "Phenotype Team",
                    "url": "https://github.com/KooshaPari/phenotype-infrakit"
                }
            },
            "servers": [
                {
                    "url": "http://localhost:8080",
                    "description": "Local development server"
                },
                {
                    "url": "https://api.phenotype.io",
                    "description": "Production API"
                }
            ],
            "paths": {},
            "components": {
                "schemas": {}
            },
            "tags": [
                {
                    "name": "Inbound Ports",
                    "description": "Driving ports - interfaces for external requests"
                },
                {
                    "name": "Outbound Ports",
                    "description": "Driven ports - interfaces for external services"
                },
                {
                    "name": "Models",
                    "description": "Domain models and value objects"
                }
            ]
        });

        // Add inbound port schemas
        Self::add_inbound_ports(&mut spec);

        // Add outbound port schemas
        Self::add_outbound_ports(&mut spec);

        // Add domain model schemas
        Self::add_domain_models(&mut spec);

        // Add REST API paths
        Self::add_rest_paths(&mut spec);

        spec
    }

    fn add_inbound_ports(spec: &mut Value) {
        let schemas = &mut spec["components"]["schemas"];

        // UseCase interface
        schemas["UseCase"] = json!({
            "type": "object",
            "description": "Use case port for executing business operations",
            "properties": {
                "execute": {
                    "type": "object",
                    "description": "Executes the use case with the given request",
                    "properties": {
                        "request": {
                            "type": "object",
                            "description": "Request object"
                        }
                    },
                    "required": ["request"]
                }
            },
            "tags": ["Inbound Ports"]
        });

        // CommandHandler interface
        schemas["CommandHandler"] = json!({
            "type": "object",
            "description": "Command handler port for processing commands",
            "properties": {
                "handle": {
                    "type": "object",
                    "description": "Handles a command",
                    "properties": {
                        "command": {
                            "type": "object",
                            "description": "Command to handle"
                        }
                    },
                    "required": ["command"]
                }
            },
            "tags": ["Inbound Ports"]
        });

        // QueryHandler interface
        schemas["QueryHandler"] = json!({
            "type": "object",
            "description": "Query handler port for processing queries",
            "properties": {
                "handle": {
                    "type": "object",
                    "description": "Handles a query and returns the result",
                    "properties": {
                        "query": {
                            "type": "object",
                            "description": "Query to handle"
                        }
                    },
                    "required": ["query"]
                }
            },
            "tags": ["Inbound Ports"]
        });

        // EventHandler interface
        schemas["EventHandler"] = json!({
            "type": "object",
            "description": "Event handler port for processing domain events",
            "properties": {
                "handle": {
                    "type": "object",
                    "description": "Handles a domain event",
                    "properties": {
                        "event": {
                            "type": "object",
                            "description": "Domain event to handle"
                        }
                    },
                    "required": ["event"]
                }
            },
            "tags": ["Inbound Ports"]
        });
    }

    fn add_outbound_ports(spec: &mut Value) {
        let schemas = &mut spec["components"]["schemas"];

        // Repository interface
        schemas["Repository"] = json!({
            "type": "object",
            "description": "Repository port for persisting and retrieving domain entities",
            "properties": {
                "save": {
                    "type": "object",
                    "description": "Saves an entity",
                    "properties": {
                        "id": { "type": "string" },
                        "entity": { "type": "object" }
                    }
                },
                "get": {
                    "type": "object",
                    "description": "Retrieves an entity by ID",
                    "properties": {
                        "id": { "type": "string" }
                    }
                },
                "delete": {
                    "type": "object",
                    "description": "Deletes an entity by ID",
                    "properties": {
                        "id": { "type": "string" }
                    }
                },
                "list": {
                    "type": "array",
                    "description": "Lists all entities"
                }
            },
            "tags": ["Outbound Ports"]
        });

        // CachePort interface
        schemas["CachePort"] = json!({
            "type": "object",
            "description": "Cache port for storing and retrieving cached values",
            "properties": {
                "get": {
                    "type": "object",
                    "description": "Gets a value from cache",
                    "properties": {
                        "key": { "type": "string" }
                    }
                },
                "set": {
                    "type": "object",
                    "description": "Sets a value in cache",
                    "properties": {
                        "key": { "type": "string" },
                        "value": { "type": "object" }
                    }
                },
                "invalidate": {
                    "type": "object",
                    "description": "Invalidates a cache entry",
                    "properties": {
                        "key": { "type": "string" }
                    }
                }
            },
            "tags": ["Outbound Ports"]
        });

        // EventBus interface
        schemas["EventBus"] = json!({
            "type": "object",
            "description": "Event bus port for publishing and subscribing to domain events",
            "properties": {
                "publish": {
                    "type": "object",
                    "description": "Publishes an event to the bus",
                    "properties": {
                        "event": { "type": "object" }
                    }
                },
                "publish_batch": {
                    "type": "object",
                    "description": "Publishes multiple events",
                    "properties": {
                        "events": {
                            "type": "array",
                            "items": { "type": "object" }
                        }
                    }
                }
            },
            "tags": ["Outbound Ports"]
        });

        // SecretManager interface
        schemas["SecretManager"] = json!({
            "type": "object",
            "description": "Secret manager port for secure credential storage and retrieval",
            "properties": {
                "get": {
                    "type": "object",
                    "description": "Retrieves a secret by name",
                    "properties": {
                        "name": { "type": "string" }
                    }
                },
                "set": {
                    "type": "object",
                    "description": "Stores a secret",
                    "properties": {
                        "name": { "type": "string" },
                        "value": { "type": "string" }
                    }
                },
                "delete": {
                    "type": "object",
                    "description": "Deletes a secret",
                    "properties": {
                        "name": { "type": "string" }
                    }
                }
            },
            "tags": ["Outbound Ports"]
        });

        // ConfigLoader interface
        schemas["ConfigLoader"] = json!({
            "type": "object",
            "description": "Configuration loader port",
            "properties": {
                "load": {
                    "type": "object",
                    "description": "Loads configuration and returns as a map",
                    "properties": {}
                }
            },
            "tags": ["Outbound Ports"]
        });
    }

    fn add_domain_models(spec: &mut Value) {
        let schemas = &mut spec["components"]["schemas"];

        // DomainEvent model
        schemas["DomainEvent"] = json!({
            "type": "object",
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
                    "description": "When the event occurred"
                },
                "data": {
                    "type": "object",
                    "description": "Event payload data"
                }
            },
            "required": ["id", "aggregate_id", "event_type", "timestamp", "data"],
            "tags": ["Models"]
        });
    }

    fn add_rest_paths(spec: &mut Value) {
        let paths = &mut spec["paths"];

        // Health check endpoint
        paths["/health"] = json!({
            "get": {
                "summary": "Health check endpoint",
                "tags": ["Health"],
                "responses": {
                    "200": {
                        "description": "Service is healthy"
                    }
                }
            }
        });

        // Events endpoint
        paths["/events"] = json!({
            "post": {
                "summary": "Publish domain event",
                "tags": ["Events"],
                "requestBody": {
                    "required": true,
                    "content": {
                        "application/json": {
                            "schema": { "$ref": "#/components/schemas/DomainEvent" }
                        }
                    }
                },
                "responses": {
                    "201": {
                        "description": "Event published successfully"
                    }
                }
            }
        });
    }
}
