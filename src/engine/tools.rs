use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

pub struct ToolManager;

impl ToolManager {
    pub fn list_tools() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "read_file".to_string(),
                description: "Read the complete contents of a file".to_string(),
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
        ]
    }

    pub fn read_file(path: &str) -> Result<String> {
        let content = fs::read_to_string(path)?;
        Ok(content)
    }

    pub fn write_file(path: &str, content: &str) -> Result<()> {
        let path = Path::new(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    pub async fn fetch_url(url: &str) -> Result<String> {
        let response = reqwest::get(url).await?.text().await?;
        // Optional: Convert HTML to Markdown if needed, but for now raw text
        Ok(response)
    }

    pub fn run_command(command: &str) -> Result<String> {
        // Security warning: This is dangerous. In a real product, we need sandboxing or user confirmation.
        // For this MVP, we execute directly.
        use std::process::Command;

        let output = Command::new("sh").arg("-c").arg(command).output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(stdout.to_string())
        } else {
            Ok(format!(
                "Command failed:\nStdout: {}\nStderr: {}",
                stdout, stderr
            ))
        }
    }
}
