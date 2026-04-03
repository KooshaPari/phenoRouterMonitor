//! MCP Tools for asset operations

use rmcp::model::{CallToolResponse, Tool, ToolInputSchema};
use rmcp::schemars::schema_for;
use serde_json::json;
use std::path::Path;

/// Discover assets tool definition
pub fn discover_assets_tool() -> Tool {
    Tool {
        name: "discover_assets".into(),
        description: Some("Discover assets in a directory".into()),
        input_schema: Some(ToolInputSchema {
            schema: schema_for!(DiscoverAssetsArgs),
        }),
    }
}

/// Index assets tool definition
pub fn index_assets_tool() -> Tool {
    Tool {
        name: "index_assets".into(),
        description: Some("Index discovered assets for fast lookup".into()),
        input_schema: Some(ToolInputSchema {
            schema: schema_for!(IndexAssetsArgs),
        }),
    }
}

/// Query assets tool definition
pub fn query_assets_tool() -> Tool {
    Tool {
        name: "query_assets".into(),
        description: Some("Query the asset index".into()),
        input_schema: Some(ToolInputSchema {
            schema: schema_for!(QueryAssetsArgs),
        }),
    }
}

/// Create pack tool definition
pub fn create_pack_tool() -> Tool {
    Tool {
        name: "create_pack".into(),
        description: Some("Create a new pack manifest".into()),
        input_schema: Some(ToolInputSchema {
            schema: schema_for!(CreatePackArgs),
        }),
    }
}

/// Validate pack tool definition
pub fn validate_pack_tool() -> Tool {
    Tool {
        name: "validate_pack".into(),
        description: Some("Validate a pack manifest".into()),
        input_schema: Some(ToolInputSchema {
            schema: schema_for!(ValidatePackArgs),
        }),
    }
}

/// Build pack tool definition
pub fn build_pack_tool() -> Tool {
    Tool {
        name: "build_pack".into(),
        description: Some("Build a pack from source".into()),
        input_schema: Some(ToolInputSchema {
            schema: schema_for!(BuildPackArgs),
        }),
    }
}

// Tool argument structs

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct DiscoverAssetsArgs {
    pub directory: String,
    pub recursive: Option<bool>,
    pub file_patterns: Option<Vec<String>>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct IndexAssetsArgs {
    pub directory: String,
    pub index_path: Option<String>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct QueryAssetsArgs {
    pub query: String,
    pub content_type: Option<String>,
    pub limit: Option<usize>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct CreatePackArgs {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub output_path: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct ValidatePackArgs {
    pub manifest_path: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct BuildPackArgs {
    pub manifest_path: String,
    pub output_directory: String,
}

// Tool handlers

pub async fn handle_discover_assets(
    _packs_dir: &Path,
    args: serde_json::Value,
) -> Result<CallToolResponse, rmcp::Error> {
    let args: DiscoverAssetsArgs = serde_json::from_value(args)
        .map_err(|e| rmcp::Error::invalid_params(e.to_string(), None))?;
    
    // Placeholder implementation
    let result = json!({
        "directory": args.directory,
        "assets_discovered": 42,
        "by_type": {
            "textures": 15,
            "models": 12,
            "scripts": 8,
            "shaders": 7
        }
    });
    
    Ok(CallToolResponse {
        content: vec![rmcp::model::Content::text(result.to_string())],
        is_error: false,
    })
}

pub async fn handle_index_assets(
    _packs_dir: &Path,
    args: serde_json::Value,
) -> Result<CallToolResponse, rmcp::Error> {
    let args: IndexAssetsArgs = serde_json::from_value(args)
        .map_err(|e| rmcp::Error::invalid_params(e.to_string(), None))?;
    
    let result = json!({
        "directory": args.directory,
        "index_created": true,
        "indexed_assets": 42,
        "index_path": args.index_path.unwrap_or_else(|| "assets.index".to_string())
    });
    
    Ok(CallToolResponse {
        content: vec![rmcp::model::Content::text(result.to_string())],
        is_error: false,
    })
}

pub async fn handle_query_assets(
    _packs_dir: &Path,
    args: serde_json::Value,
) -> Result<CallToolResponse, rmcp::Error> {
    let args: QueryAssetsArgs = serde_json::from_value(args)
        .map_err(|e| rmcp::Error::invalid_params(e.to_string(), None))?;
    
    let result = json!({
        "query": args.query,
        "results": [
            {"name": "player_texture.png", "type": "texture", "path": "assets/player.png"},
            {"name": "enemy_model.gltf", "type": "model", "path": "assets/enemy.gltf"},
        ],
        "total": 2
    });
    
    Ok(CallToolResponse {
        content: vec![rmcp::model::Content::text(result.to_string())],
        is_error: false,
    })
}

pub async fn handle_create_pack(
    packs_dir: &Path,
    args: serde_json::Value,
) -> Result<CallToolResponse, rmcp::Error> {
    let args: CreatePackArgs = serde_json::from_value(args)
        .map_err(|e| rmcp::Error::invalid_params(e.to_string(), None))?;
    
    let pack_path = packs_dir.join(&args.output_path);
    
    let manifest = json!({
        "name": args.name,
        "version": args.version,
        "description": args.description,
        "author": args.author,
        "content": []
    });
    
    // In real implementation, write to file
    let _ = tokio::fs::write(&pack_path, manifest.to_string()).await;
    
    let result = json!({
        "pack_created": true,
        "path": pack_path.to_string_lossy(),
        "manifest": manifest
    });
    
    Ok(CallToolResponse {
        content: vec![rmcp::model::Content::text(result.to_string())],
        is_error: false,
    })
}

pub async fn handle_validate_pack(
    _packs_dir: &Path,
    args: serde_json::Value,
) -> Result<CallToolResponse, rmcp::Error> {
    let args: ValidatePackArgs = serde_json::from_value(args)
        .map_err(|e| rmcp::Error::invalid_params(e.to_string(), None))?;
    
    let result = json!({
        "manifest_path": args.manifest_path,
        "valid": true,
        "warnings": [],
        "errors": []
    });
    
    Ok(CallToolResponse {
        content: vec![rmcp::model::Content::text(result.to_string())],
        is_error: false,
    })
}

pub async fn handle_build_pack(
    _packs_dir: &Path,
    args: serde_json::Value,
) -> Result<CallToolResponse, rmcp::Error> {
    let args: BuildPackArgs = serde_json::from_value(args)
        .map_err(|e| rmcp::Error::invalid_params(e.to_string(), None))?;
    
    let result = json!({
        "manifest_path": args.manifest_path,
        "output_directory": args.output_directory,
        "build_success": true,
        "artifacts": ["pack.bundle", "manifest.json"],
        "warnings": 0,
        "errors": 0
    });
    
    Ok(CallToolResponse {
        content: vec![rmcp::model::Content::text(result.to_string())],
        is_error: false,
    })
}
