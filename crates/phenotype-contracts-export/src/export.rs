//! Contract exporting and artifact generation.

use crate::models::{ContractArtifact, ContractBundle};
use crate::openapi::OpenAPIGenerator;
use crate::protobuf::ProtobufGenerator;
use crate::schema::JsonSchemaGenerator;
use crate::clients::{TypeScriptClientGenerator, GoClientGenerator};

/// Main contract exporter that orchestrates artifact generation.
pub struct ContractExporter;

impl ContractExporter {
    /// Generates all contract artifacts in all supported formats.
    pub fn export_all() -> ContractBundle {
        let mut bundle = ContractBundle::new();

        // Generate OpenAPI specification
        let openapi_json = serde_json::to_string_pretty(&OpenAPIGenerator::generate())
            .unwrap_or_default();
        bundle.add_artifact(ContractArtifact::new(
            "openapi.json".to_string(),
            "contracts/openapi.json".to_string(),
            "application/json".to_string(),
            openapi_json,
        ));

        // Generate Protocol Buffer definitions
        let proto_def = ProtobufGenerator::generate();
        bundle.add_artifact(ContractArtifact::new(
            "contracts.proto".to_string(),
            "contracts/contracts.proto".to_string(),
            "text/plain".to_string(),
            proto_def,
        ));

        // Generate JSON Schemas
        let schemas = JsonSchemaGenerator::generate();
        let schema_json = serde_json::to_string_pretty(&schemas).unwrap_or_default();
        bundle.add_artifact(ContractArtifact::new(
            "schema.json".to_string(),
            "contracts/schema.json".to_string(),
            "application/json".to_string(),
            schema_json,
        ));

        // Generate TypeScript client
        let ts_client = TypeScriptClientGenerator::generate();
        bundle.add_artifact(ContractArtifact::new(
            "client.ts".to_string(),
            "clients/typescript/index.ts".to_string(),
            "text/typescript".to_string(),
            ts_client,
        ));

        // Generate Go client
        let go_client = GoClientGenerator::generate();
        bundle.add_artifact(ContractArtifact::new(
            "client.go".to_string(),
            "clients/go/client.go".to_string(),
            "text/plain".to_string(),
            go_client,
        ));

        bundle
    }

    /// Generates specific artifacts by format.
    pub fn export_format(format: &str) -> Option<ContractArtifact> {
        match format.to_lowercase().as_str() {
            "openapi" | "openapi.json" => {
                let openapi_json = serde_json::to_string_pretty(&OpenAPIGenerator::generate())
                    .unwrap_or_default();
                Some(ContractArtifact::new(
                    "openapi.json".to_string(),
                    "contracts/openapi.json".to_string(),
                    "application/json".to_string(),
                    openapi_json,
                ))
            }
            "protobuf" | "proto" => {
                let proto_def = ProtobufGenerator::generate();
                Some(ContractArtifact::new(
                    "contracts.proto".to_string(),
                    "contracts/contracts.proto".to_string(),
                    "text/plain".to_string(),
                    proto_def,
                ))
            }
            "json-schema" | "schema.json" => {
                let schemas = JsonSchemaGenerator::generate();
                let schema_json = serde_json::to_string_pretty(&schemas).unwrap_or_default();
                Some(ContractArtifact::new(
                    "schema.json".to_string(),
                    "contracts/schema.json".to_string(),
                    "application/json".to_string(),
                    schema_json,
                ))
            }
            "typescript" | "ts" => {
                let ts_client = TypeScriptClientGenerator::generate();
                Some(ContractArtifact::new(
                    "client.ts".to_string(),
                    "clients/typescript/index.ts".to_string(),
                    "text/typescript".to_string(),
                    ts_client,
                ))
            }
            "go" | "golang" => {
                let go_client = GoClientGenerator::generate();
                Some(ContractArtifact::new(
                    "client.go".to_string(),
                    "clients/go/client.go".to_string(),
                    "text/plain".to_string(),
                    go_client,
                ))
            }
            _ => None,
        }
    }

    /// Gets metadata about the exported contracts.
    pub fn metadata() -> crate::models::ContractMetadata {
        crate::models::ContractMetadata::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_all_creates_all_artifacts() {
        let bundle = ContractExporter::export_all();
        assert_eq!(bundle.artifact_count(), 5); // OpenAPI, Proto, Schema, TS, Go
    }

    #[test]
    fn test_export_format_openapi() {
        let artifact = ContractExporter::export_format("openapi").unwrap();
        assert_eq!(artifact.name, "openapi.json");
        assert!(!artifact.content.is_empty());
    }

    #[test]
    fn test_export_format_protobuf() {
        let artifact = ContractExporter::export_format("proto").unwrap();
        assert_eq!(artifact.name, "contracts.proto");
        assert!(!artifact.content.is_empty());
    }

    #[test]
    fn test_export_format_typescript() {
        let artifact = ContractExporter::export_format("ts").unwrap();
        assert_eq!(artifact.name, "client.ts");
        assert!(!artifact.content.is_empty());
    }

    #[test]
    fn test_export_format_go() {
        let artifact = ContractExporter::export_format("go").unwrap();
        assert_eq!(artifact.name, "client.go");
        assert!(!artifact.content.is_empty());
    }

    #[test]
    fn test_metadata_contains_expected_fields() {
        let meta = ContractExporter::metadata();
        assert_eq!(meta.contract_version, "1.0.0");
        assert!(!meta.formats.is_empty());
        assert!(!meta.inbound_ports.is_empty());
        assert!(!meta.outbound_ports.is_empty());
    }
}
