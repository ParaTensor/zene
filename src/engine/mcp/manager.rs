use anyhow::{Result, Context};
use crate::config::mcp::McpConfig;
use crate::engine::tools::ToolDefinition;
use zene_mcp::McpClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

pub struct McpManager {
    // Map of server_name -> Client
    clients: RwLock<HashMap<String, Arc<McpClient>>>,
    config: McpConfig,
}

impl McpManager {
    pub fn new(config: McpConfig) -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Connect to all configured servers in parallel
    pub async fn connect_all(&self) {
        for (name, server_config) in &self.config.servers {
            info!("Connecting to MCP Server: {} ({})...", name, server_config.command);
            
            // TODO: zene-mcp client initialization
            match McpClient::new(&server_config.command, &server_config.args).await {
                Ok(client) => {
                    info!("✅ Connected to MCP Server: {}", name);
                    self.clients.write().await.insert(name.clone(), Arc::new(client));
                }
                Err(e) => {
                    error!("❌ Failed to connect to MCP Server {}: {}", name, e);
                }
            }
        }
    }

    /// List tools from all connected clients
    pub async fn list_tools(&self) -> Vec<ToolDefinition> {
        let clients = self.clients.read().await;
        let mut all_tools = Vec::new();

        for (server_name, client) in clients.iter() {
            match client.list_tools().await {
                Ok(tools) => {
                    for tool in tools {
                        all_tools.push(ToolDefinition {
                            name: format!("{}__{}", server_name, tool.name), // Namespacing: git__commit
                            description: tool.description.unwrap_or_else(|| "No description".to_string()),
                            input_schema: tool.input_schema,
                        });
                    }
                }
                Err(e) => {
                    warn!("Failed to list tools from {}: {}", server_name, e);
                }
            }
        }

        all_tools
    }

    /// Execute a tool on the appropriate server
    pub async fn call_tool(&self, name: &str, args: serde_json::Value) -> Result<String> {
        // Parse "server__tool" format
        let parts: Vec<&str> = name.splitn(2, "__").collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid MCP tool name: {}. Expected format: server__tool", name));
        }

        let server_name = parts[0];
        let tool_name = parts[1];

        let clients = self.clients.read().await;
        if let Some(client) = clients.get(server_name) {
            let result = client.call_tool(tool_name, args).await?;
            // Extract text content from MCP result
            // Assuming simple text response for MVP, use Debug formatting as fail-safe
             Ok(format!("{:?}", result))
        } else {
            Err(anyhow::anyhow!("MCP Server not found: {}", server_name))
        }
    }
}
