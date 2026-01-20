//! MCP (Model Context Protocol) Server
//!
//! Exposes OxideKit knowledge as tools for AI assistants.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP Server for AI assistant integration
#[derive(Debug, Clone)]
pub struct McpServer {
    tools: HashMap<String, McpTool>,
    resources: HashMap<String, McpResource>,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register_tool(&mut self, tool: McpTool) {
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Register a resource
    pub fn register_resource(&mut self, resource: McpResource) {
        self.resources.insert(resource.uri.clone(), resource);
    }

    /// Get all registered tools
    pub fn tools(&self) -> impl Iterator<Item = &McpTool> {
        self.tools.values()
    }

    /// Handle an MCP request
    pub fn handle_request(&self, request: McpRequest) -> McpResponse {
        match request.method {
            McpMethod::ListTools => McpResponse::tools(self.tools.values().cloned().collect()),
            McpMethod::ListResources => McpResponse::resources(self.resources.values().cloned().collect()),
            McpMethod::CallTool { name, arguments } => {
                if let Some(_tool) = self.tools.get(&name) {
                    // Tool execution would happen here
                    McpResponse::success(serde_json::json!({"result": "ok", "tool": name, "args": arguments}))
                } else {
                    McpResponse::error(format!("Unknown tool: {}", name))
                }
            }
            McpMethod::ReadResource { uri } => {
                if let Some(resource) = self.resources.get(&uri) {
                    McpResponse::success(serde_json::json!({"content": resource.description}))
                } else {
                    McpResponse::error(format!("Unknown resource: {}", uri))
                }
            }
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// MCP Resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
}

/// MCP Request method
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case")]
pub enum McpMethod {
    ListTools,
    ListResources,
    CallTool { name: String, arguments: serde_json::Value },
    ReadResource { uri: String },
}

/// MCP Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub id: String,
    pub method: McpMethod,
}

/// MCP Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub id: Option<String>,
    pub result: Option<serde_json::Value>,
    pub error: Option<McpError>,
}

impl McpResponse {
    pub fn success(result: serde_json::Value) -> Self {
        Self { id: None, result: Some(result), error: None }
    }

    pub fn error(message: String) -> Self {
        Self { id: None, result: None, error: Some(McpError { code: -1, message }) }
    }

    pub fn tools(tools: Vec<McpTool>) -> Self {
        Self::success(serde_json::json!({"tools": tools}))
    }

    pub fn resources(resources: Vec<McpResource>) -> Self {
        Self::success(serde_json::json!({"resources": resources}))
    }
}

/// MCP Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_creation() {
        let server = McpServer::new();
        assert_eq!(server.tools().count(), 0);
    }
}
