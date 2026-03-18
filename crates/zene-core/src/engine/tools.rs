use crate::engine::context::ContextEngine;
use crate::engine::python_env::PythonEnv;
use crate::engine::mcp::manager::McpManager;
use crate::engine::error::{Result, ZeneError};
use xtrace_client::current_trace_id;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

use crate::engine::contracts::{AgentEvent, FileChange};
use tokio::sync::mpsc::UnboundedSender;
use tokio::task;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

pub struct ToolManager {
    mcp_manager: Option<Arc<McpManager>>,
    context_engine: Arc<Mutex<ContextEngine>>,
}

impl ToolManager {
    pub fn new(mcp_manager: Option<Arc<McpManager>>, context_engine: Arc<Mutex<ContextEngine>>) -> Self {
        Self { mcp_manager, context_engine }
    }

    pub async fn list_tools(&self) -> Vec<ToolDefinition> {
        let mut tools = vec![
            ToolDefinition {
                name: "read_file".to_string(),
                description: "Read the complete contents of a file. CRITICAL: You MUST provide the 'path' parameter!".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Relative path to the file" }
                    },
                    "required": ["path"]
                }),
            },
            ToolDefinition {
                name: "write_file".to_string(),
                description: "Overwrite a file with new content".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Relative path to the file" },
                        "content": { "type": "string", "description": "New content for the file" }
                    },
                    "required": ["path", "content"]
                }),
            },
            ToolDefinition {
                name: "fetch_url".to_string(),
                description: "Fetch content from a URL (HTTP/HTTPS)".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": { "type": "string", "description": "The URL to fetch" }
                    },
                    "required": ["url"]
                }),
            },
            ToolDefinition {
                name: "run_command".to_string(),
                description: "Execute a shell command".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": { "type": "string", "description": "The shell command to execute" }
                    },
                    "required": ["command"]
                }),
            },
            ToolDefinition {
                name: "search_code".to_string(),
                description: "Search for a pattern in the codebase using ripgrep. CRITICAL: You MUST provide the 'pattern' parameter!".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string", "description": "The regex pattern to search for" }
                    },
                    "required": ["pattern"]
                }),
            },
            ToolDefinition {
                name: "list_files".to_string(),
                description: "List files and directories".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Directory to list (defaults to current dir)" },
                        "depth": { "type": "integer", "description": "Depth limit (optional)" }
                    }
                }),
            },
            ToolDefinition {
                name: "apply_patch".to_string(),
                description: "Apply a partial update to a file using search and replace blocks".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string", "description": "Path to the file to modify" },
                        "original_snippet": { "type": "string", "description": "The exact block of code to be replaced" },
                        "new_snippet": { "type": "string", "description": "The new block of code to insert" },
                        "start_line": { "type": "integer", "description": "Optional: approximate start line number to help disambiguate matches" }
                    },
                    "required": ["path", "original_snippet", "new_snippet"]
                }),
            },
            ToolDefinition {
                name: "set_env".to_string(),
                description: "Set an environment variable for the session".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "key": { "type": "string", "description": "Environment variable name" },
                        "value": { "type": "string", "description": "Value to set" }
                    },
                    "required": ["key", "value"]
                }),
            },
            ToolDefinition {
                name: "get_env".to_string(),
                description: "Get the value of an environment variable".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "key": { "type": "string", "description": "Environment variable name" }
                    },
                    "required": ["key"]
                }),
            },
            ToolDefinition {
                name: "run_python".to_string(),
                description: "Execute a Python script in a dedicated virtual environment".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "script_path": { "type": "string", "description": "Path to the .py file" },
                        "args": { "type": "array", "items": { "type": "string" }, "description": "Arguments to pass to the script" }
                    },
                    "required": ["script_path"]
                }),
            },
        ];
        
        // Add Memory Tools
        #[cfg(feature = "knowledge")]
        {
            tools.push(ToolDefinition {
                name: "memory_search".to_string(),
                description: "Search for relevant code and documentation using semantic search (RAG)".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "The natural language query describing what you are looking for" }
                    },
                    "required": ["query"]
                }),
            });

            tools.push(ToolDefinition {
                name: "memory_index".to_string(),
                description: "Index the current project files into the vector memory. Use this if you think the memory is outdated.".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {},
                }),
            });
        }

        // Append MCP tools dynamically
        if let Some(manager) = &self.mcp_manager {
            let mcp_tools: Vec<ToolDefinition> = manager.list_tools().await;
            tools.extend(mcp_tools);
        }

        tools
    }

    pub async fn read_file(&self, path: &str) -> Result<String> {
        let path = path.to_string();
        task::spawn_blocking(move || {
            std::fs::read_to_string(&path)
                .map_err(|e| ZeneError::InternalError(e.to_string()))
        })
        .await
        .map_err(|e| ZeneError::InternalError(format!("spawn_blocking failed: {}", e)))?
    }

    pub async fn write_file(&self, path: &str, content: &str, event_sender: Option<&UnboundedSender<AgentEvent>>) -> Result<()> {
        let path = path.to_string();
        let content = content.to_string();
        let sender_clone = event_sender.cloned();

        task::spawn_blocking(move || {
            let p = Path::new(&path);
            if let Some(parent) = p.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            let change_type = if p.exists() { "modified" } else { "created" };
            std::fs::write(p, content)?;

            if let Some(sender) = sender_clone {
                let _ = sender.send(AgentEvent::FileStateChanged(FileChange {
                    path: path.to_string(),
                    change_type: change_type.to_string(),
                    diff: None, 
                }));
            }
            Ok(())
        })
        .await
        .map_err(|e| ZeneError::InternalError(format!("spawn_blocking failed: {}", e)))?
    }

    pub async fn fetch_url(&self, url: &str) -> Result<String> {
        let mut builder = reqwest::Client::builder();
        
        if let Some(tid) = current_trace_id() {
            let mut headers = reqwest::header::HeaderMap::new();
            if let Ok(val) = reqwest::header::HeaderValue::from_str(&tid.to_string()) {
                headers.insert("X-Trace-Id", val);
                builder = builder.default_headers(headers);
            }
        }

        let client = builder.build()?;
        let response = client.get(url).send().await?.text().await?;
        Ok(response)
    }

    pub async fn run_command(&self, command: &str, envs: &HashMap<String, String>) -> Result<String> {
        // Security warning: This is dangerous. In a real product, we need sandboxing or user confirmation.
        // For this MVP, we execute directly but with timeout and stdin blocking.
        
        // Timeout: 60 seconds
        let timeout_duration = Duration::from_secs(60);

        let mut final_envs = envs.clone();
        if let Some(tid) = current_trace_id() {
            final_envs.insert("ZENE_TRACE_ID".to_string(), tid.to_string());
        }

        let child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .envs(&final_envs)
            .stdin(Stdio::null()) // Block stdin to prevent zombie processes
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        match timeout(timeout_duration, child.wait_with_output()).await {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                if output.status.success() {
                    Ok(stdout.to_string())
                } else {
                    Ok(format!("Command failed with status {}:\nStdout: {}\nStderr: {}", output.status, stdout, stderr))
                }
            },
            Ok(Err(e)) => Err(ZeneError::ToolError(format!("Command execution failed: {}", e))),
            Err(_) => {
                Err(ZeneError::ToolError("Command execution timed out (limit: 60s). Note: Process might linger as we lost ownership.".to_string()))
            }
        }
    }

    pub async fn run_python(&self, script_path: &str, args: &[String], envs: &HashMap<String, String>) -> Result<String> {
         let root = std::env::current_dir()?;
         let python_env = PythonEnv::new(root);

         // Ensure venv exists (async)
         let python_bin = python_env.ensure_venv().await?;
         
         // Lazily try to install requirements
         let _ = python_env.install_requirements().await;

         let mut cmd_builder = String::new();
         cmd_builder.push_str(python_bin.to_str().unwrap());
         cmd_builder.push(' ');
         cmd_builder.push_str(script_path);

         for arg in args {
             cmd_builder.push(' ');
             cmd_builder.push_str(&format!("'{}'", arg));
         }

         self.run_command(&cmd_builder, envs).await
    }

    pub async fn search_code(&self, pattern: &str) -> Result<Vec<String>> {
        let root = std::env::current_dir()?;
        let engine = self.context_engine.lock().await;
        engine.search_code_async(&root, pattern).await
    }

    pub async fn list_files(&self, path: Option<&str>, depth: Option<i64>) -> Result<Vec<String>> {
        let root = std::env::current_dir()?;
        let target_path = if let Some(p) = path {
            root.join(p)
        } else {
            root
        };
        let depth = depth.map(|d| d as usize);
        let engine = self.context_engine.lock().await;
        Ok(engine.list_files_async(&target_path, depth).await)
    }

    pub async fn apply_patch(&self, path: &str, original_snippet: &str, new_snippet: &str, start_line: Option<i64>, event_sender: Option<&UnboundedSender<AgentEvent>>) -> Result<()> {
        let path = path.to_string();
        let original = original_snippet.to_string();
        let new_s = new_snippet.to_string();
        let start = start_line;
        let sender_clone = event_sender.cloned();

        task::spawn_blocking(move || {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| ZeneError::InternalError(e.to_string()))?;

            // Normalize line endings to LF
            let content_lf = content.replace("\r\n", "\n");
            let original_lf = original.replace("\r\n", "\n");

            if start.is_none() {
                if let Some(start_idx) = content_lf.find(&original_lf) {
                    let end_idx = start_idx + original_lf.len();
                    let mut new_content = String::with_capacity(content_lf.len() - original_lf.len() + new_s.len());
                    new_content.push_str(&content_lf[..start_idx]);
                    new_content.push_str(&new_s);
                    new_content.push_str(&content_lf[end_idx..]);
                    std::fs::write(&path, new_content)?;
                    
                    if let Some(sender) = sender_clone {
                        let _ = sender.send(AgentEvent::FileStateChanged(FileChange {
                            path: path.to_string(),
                            change_type: "modified".to_string(),
                            diff: None, 
                        }));
                    }
                    return Ok(());
                }
            }

            let content_lines: Vec<&str> = content_lf.lines().collect();
            let original_lines: Vec<&str> = original_lf.lines().collect();
            
            if original_lines.is_empty() {
                 return Err(ZeneError::InternalError("Original snippet is empty".to_string()));
            }

            let mut match_found = false;
            let mut match_start_line = 0;

            let search_start = start.map(|l| (l as usize).saturating_sub(1)).unwrap_or(0);

            for i in search_start..=content_lines.len().saturating_sub(original_lines.len()) {
                let mut current_match = true;
                for j in 0..original_lines.len() {
                    if content_lines[i + j].trim() != original_lines[j].trim() {
                        current_match = false;
                        break;
                    }
                }

                if current_match {
                    match_found = true;
                    match_start_line = i;
                    break;
                }
            }

            if match_found {
                let mut sb = String::new();
                
                for k in 0..match_start_line {
                    sb.push_str(content_lines[k]);
                    sb.push('\n');
                }
                sb.push_str(&new_s);
                
                let match_end_line = match_start_line + original_lines.len();
                if match_end_line < content_lines.len() {
                     if !new_s.ends_with('\n') {
                         sb.push('\n');
                     }
                     for k in match_end_line..content_lines.len() {
                         sb.push_str(content_lines[k]);
                         if k < content_lines.len() - 1 {
                             sb.push('\n');
                         }
                     }
                     if !sb.ends_with('\n') && content.ends_with('\n') {
                         sb.push('\n');
                     }
                }
                std::fs::write(&path, sb)?;
                if let Some(sender) = sender_clone {
                    let _ = sender.send(AgentEvent::FileStateChanged(FileChange {
                        path: path.to_string(),
                        change_type: "modified".to_string(),
                        diff: None, 
                    }));
                }
                return Ok(());
            }

            Err(ZeneError::InternalError("Original snippet not found (tried exact and fuzzy match).".to_string()))
        })
        .await
        .map_err(|e| ZeneError::InternalError(format!("spawn_blocking failed: {}", e)))?
    }
}
