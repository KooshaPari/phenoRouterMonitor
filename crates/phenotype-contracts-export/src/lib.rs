//! Phenotype Contracts Export
//!
//! This crate provides contract extraction and export functionality for the Phenotype hexagonal
//! architecture. It converts stable ports and public DTOs into versioned contract artifacts
//! consumable by Phenotype polyglot clients (TypeScript, Go, etc.) in multiple formats:
//! - OpenAPI 3.0.0
//! - Protocol Buffers (proto3)
//! - JSON Schema
//!
//! # Features
//!
//! - Extract inbound and outbound ports from `phenotype-contracts`
//! - Generate OpenAPI specifications for REST API contracts
//! - Generate Protocol Buffer definitions for gRPC/MCP contracts
//! - Generate JSON Schema for data models
//! - Create polyglot client POCs (TypeScript, Go)
//! - Versioned artifact publishing

pub mod openapi;
pub mod protobuf;
pub mod models;
pub mod schema;
pub mod clients;
pub mod export;

pub use openapi::OpenAPIGenerator;
pub use protobuf::ProtobufGenerator;
pub use schema::JsonSchemaGenerator;
pub use export::ContractExporter;
pub use models::{ContractMetadata, ContractArtifact, ContractBundle};
pub use clients::{TypeScriptClientGenerator, GoClientGenerator};

/// Contract version constant
pub const CONTRACT_VERSION: &str = "1.0.0";

/// OpenAPI version used
pub const OPENAPI_VERSION: &str = "3.0.0";

/// Default package name for protobuf definitions
pub const PROTOBUF_PACKAGE: &str = "phenotype.contracts";
