//! Protocol Buffer 3 definition generation from Phenotype port traits.

/// Generates Protocol Buffer 3 definitions from Phenotype port traits.
pub struct ProtobufGenerator;

impl ProtobufGenerator {
    /// Generates a complete .proto file for Phenotype contracts.
    pub fn generate() -> String {
        let mut proto = String::new();

        proto.push_str("syntax = \"proto3\";\n");
        proto.push_str("package phenotype.contracts;\n\n");

        proto.push_str("// Inbound Ports (Driving Side)\n");
        proto.push_str("// These define interfaces for external actors to drive the application\n\n");

        // UseCase message
        proto.push_str("message UseCase {\n");
        proto.push_str("  message ExecuteRequest {\n");
        proto.push_str("    bytes request = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("  message ExecuteResponse {\n");
        proto.push_str("    bytes response = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("}\n\n");

        // CommandHandler message
        proto.push_str("message CommandHandler {\n");
        proto.push_str("  message HandleRequest {\n");
        proto.push_str("    bytes command = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("}\n\n");

        // QueryHandler message
        proto.push_str("message QueryHandler {\n");
        proto.push_str("  message HandleRequest {\n");
        proto.push_str("    bytes query = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("  message HandleResponse {\n");
        proto.push_str("    bytes result = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("}\n\n");

        // EventHandler message
        proto.push_str("message EventHandler {\n");
        proto.push_str("  message HandleRequest {\n");
        proto.push_str("    bytes event = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("}\n\n");

        proto.push_str("// Outbound Ports (Driven Side)\n");
        proto.push_str("// These define interfaces for external services\n\n");

        // Repository message
        proto.push_str("message Repository {\n");
        proto.push_str("  message SaveRequest {\n");
        proto.push_str("    string id = 1;\n");
        proto.push_str("    bytes entity = 2;\n");
        proto.push_str("  }\n");
        proto.push_str("  message GetRequest {\n");
        proto.push_str("    string id = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("  message GetResponse {\n");
        proto.push_str("    bytes entity = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("  message DeleteRequest {\n");
        proto.push_str("    string id = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("  message ListResponse {\n");
        proto.push_str("    repeated bytes entities = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("}\n\n");

        // CachePort message
        proto.push_str("message CachePort {\n");
        proto.push_str("  message GetRequest {\n");
        proto.push_str("    string key = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("  message GetResponse {\n");
        proto.push_str("    bytes value = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("  message SetRequest {\n");
        proto.push_str("    string key = 1;\n");
        proto.push_str("    bytes value = 2;\n");
        proto.push_str("  }\n");
        proto.push_str("  message InvalidateRequest {\n");
        proto.push_str("    string key = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("}\n\n");

        // EventBus message
        proto.push_str("message EventBus {\n");
        proto.push_str("  message PublishRequest {\n");
        proto.push_str("    bytes event = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("  message PublishBatchRequest {\n");
        proto.push_str("    repeated bytes events = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("}\n\n");

        // SecretManager message
        proto.push_str("message SecretManager {\n");
        proto.push_str("  message GetRequest {\n");
        proto.push_str("    string name = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("  message GetResponse {\n");
        proto.push_str("    string value = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("  message SetRequest {\n");
        proto.push_str("    string name = 1;\n");
        proto.push_str("    string value = 2;\n");
        proto.push_str("  }\n");
        proto.push_str("  message DeleteRequest {\n");
        proto.push_str("    string name = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("}\n\n");

        // ConfigLoader message
        proto.push_str("message ConfigLoader {\n");
        proto.push_str("  message LoadResponse {\n");
        proto.push_str("    map<string, string> config = 1;\n");
        proto.push_str("  }\n");
        proto.push_str("}\n\n");

        proto.push_str("// Domain Models\n\n");

        // DomainEvent message
        proto.push_str("message DomainEvent {\n");
        proto.push_str("  string id = 1;\n");
        proto.push_str("  string aggregate_id = 2;\n");
        proto.push_str("  string event_type = 3;\n");
        proto.push_str("  int64 timestamp = 4;\n");
        proto.push_str("  bytes data = 5;\n");
        proto.push_str("}\n");

        proto
    }
}
