//! JSON-RPC 2.0 message types for MCP protocol

use crate::error::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcRequest {
    /// A String specifying the version of the JSON-RPC protocol to be used. MUST be exactly "2.0".
    pub jsonrpc: String,
    /// A String containing the name of the method to be invoked.
    pub method: String,
    /// A Structured value that holds the parameter values to be used during the invocation of the method. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// An identifier established by the Client. If it is not included it is assumed to be a notification. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(method: impl Into<String>, params: Option<Value>, id: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            method: method.into(),
            params,
            id,
        }
    }

    /// Check if this is a notification (no id field)
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

/// JSON-RPC 2.0 response message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcResponse {
    /// A String specifying the version of the JSON-RPC protocol to be used. MUST be exactly "2.0".
    pub jsonrpc: String,
    /// The result of the method invocation. REQUIRED on success, MUST NOT exist in case of error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// An Error Object in case a method invocation has caused an error. MUST NOT exist in case of success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// It MUST be the same as the value of the id member in the Request Object.
    pub id: Value,
}

impl JsonRpcResponse {
    /// Create a successful JSON-RPC response
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create an error JSON-RPC response
    pub fn error(id: Value, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            result: None,
            error: Some(error),
            id,
        }
    }

    /// Check if this response contains an error
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

/// JSON-RPC 2.0 error object
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JsonRpcError {
    /// A Number that indicates the error type that occurred.
    pub code: i32,
    /// A String providing a short description of the error.
    pub message: String,
    /// A Primitive or Structured value that contains additional information about the error. OPTIONAL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Create a new JSON-RPC error
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Create a JSON-RPC error with data
    pub fn with_data(code: i32, message: impl Into<String>, data: Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }
}

/// JSON-RPC message (request or response)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    /// A request message
    Request(JsonRpcRequest),
    /// A response message
    Response(JsonRpcResponse),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_request_new() {
        let req = JsonRpcRequest::new("test_method", None, Some(Value::Number(1.into())));
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "test_method");
        assert!(req.params.is_none());
        assert!(!req.is_notification());
    }

    #[test]
    fn test_jsonrpc_request_notification() {
        let req = JsonRpcRequest::new("notify_method", None, None);
        assert!(req.is_notification());
    }

    #[test]
    fn test_jsonrpc_response_success() {
        let resp = JsonRpcResponse::success(Value::Number(1.into()), json!({"status": "ok"}));
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(!resp.is_error());
    }

    #[test]
    fn test_jsonrpc_response_error() {
        let err = JsonRpcError::new(-32600, "Invalid Request");
        let resp = JsonRpcResponse::error(Value::Number(1.into()), err);
        assert!(resp.is_error());
    }

    #[test]
    fn test_jsonrpc_error_with_data() {
        let data = json!({"details": "something"});
        let err = JsonRpcError::with_data(-32603, "Internal error", data.clone());
        assert_eq!(err.data, Some(data));
    }

    // Helper macro for tests
    macro_rules! json {
        ($($json:tt)*) => {
            serde_json::json!($($json)*)
        };
    }
}
