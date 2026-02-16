use crate::engine::context::ContextEngine;
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
            ToolDefinition {
                name: "search_code".to_string(),
                description: "Search for a pattern in the codebase using ripgrep".to_string(),
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

    pub fn search_code(pattern: &str) -> Result<Vec<String>> {
        let root = std::env::current_dir()?;
        let engine = ContextEngine::new()?;
        engine.search_code(&root, pattern)
    }

    pub fn list_files(path: Option<&str>, depth: Option<i64>) -> Result<Vec<String>> {
        let root = std::env::current_dir()?;
        let target_path = if let Some(p) = path {
            root.join(p)
        } else {
            root
        };
        let depth = depth.map(|d| d as usize);
        let engine = ContextEngine::new()?;
        Ok(engine.list_files(&target_path, depth))
    }

    pub fn apply_patch(path: &str, original_snippet: &str, new_snippet: &str, start_line: Option<i64>) -> Result<()> {
        let content = fs::read_to_string(path)?;

        // Normalize line endings to LF
        let content_lf = content.replace("\r\n", "\n");
        let original_lf = original_snippet.replace("\r\n", "\n");

        // Strategy 1: Exact String Match
        // Only use Strategy 1 if NO start_line hint is provided, because `find` always finds the first occurrence.
        // If a hint is provided, we want to respect it (via Strategy 2 which supports seeking).
        if start_line.is_none() {
            if let Some(start_idx) = content_lf.find(&original_lf) {
                let end_idx = start_idx + original_lf.len();
                let mut new_content = String::with_capacity(content_lf.len() - original_lf.len() + new_snippet.len());
                new_content.push_str(&content_lf[..start_idx]);
                new_content.push_str(new_snippet);
                new_content.push_str(&content_lf[end_idx..]);
                fs::write(path, new_content)?;
                return Ok(());
            }
        }

        // Strategy 2: Line-based Fuzzy Match
        let content_lines: Vec<&str> = content_lf.lines().collect();
        let original_lines: Vec<&str> = original_lf.lines().collect();
        
        if original_lines.is_empty() {
             return Err(anyhow::anyhow!("Original snippet is empty"));
        }

        let mut match_found = false;
        let mut match_start_line = 0;

        // If start_line is provided, use it as the search start point
        // We use strict start_line if provided to avoid finding the previous occurrence
        let search_start = start_line.map(|l| (l as usize).saturating_sub(1)).unwrap_or(0);

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
            
            // 1. Lines before match
            for k in 0..match_start_line {
                sb.push_str(content_lines[k]);
                sb.push('\n');
            }
            
            // 2. New snippet
            sb.push_str(new_snippet);
            
            // 3. Lines after match
            let match_end_line = match_start_line + original_lines.len();
            if match_end_line < content_lines.len() {
                 // Ensure separation if new_snippet doesn't have trailing newline
                 if !new_snippet.ends_with('\n') {
                     sb.push('\n');
                 }
                 
                 for k in match_end_line..content_lines.len() {
                     sb.push_str(content_lines[k]);
                     // Add newline for all lines except the very last one, 
                     // unless the original file had a newline at EOF (which `lines()` hides).
                     // For safety/simplicity, we add newlines between lines.
                     if k < content_lines.len() - 1 {
                         sb.push('\n');
                     }
                 }
                 // If the file originally ended with a newline (which is standard), `lines()` removed it.
                 // We should probably ensure the file ends with a newline.
                 if !sb.ends_with('\n') && content.ends_with('\n') {
                     sb.push('\n');
                 }
            } else {
                 // Replaced until the end of file.
                 // Respect `new_snippet`'s trailing newline status.
            }
            
            fs::write(path, sb)?;
            return Ok(());
        }

        Err(anyhow::anyhow!("Original snippet not found (tried exact and fuzzy match)."))
    }
}
