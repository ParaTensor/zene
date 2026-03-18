use crate::engine::contracts::AgentEvent;
use crate::engine::context::ContextEngine;
use crate::engine::tools::ToolManager;
use crate::engine::ui::UserInterface;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;

pub struct ToolHandler;

impl ToolHandler {
    pub async fn execute(
        tool_manager: &ToolManager,
        user_interface: &dyn UserInterface,
        tool_name: &str,
        args: &serde_json::Value,
        args_str: &str,
        env_vars_shared: Arc<Mutex<HashMap<String, String>>>,
        context_engine_shared: Arc<Mutex<ContextEngine>>,
        event_sender: Option<&UnboundedSender<AgentEvent>>,
    ) -> String {
        let _ = context_engine_shared;
        // Confirmation check for sensitive tools
        if (tool_name == "run_command" || tool_name == "write_file" || tool_name == "apply_patch")
            && !user_interface.confirm_execution(tool_name, args_str)
        {
            return "User denied execution".to_string();
        }

        match tool_name {
            "read_file" => {
                if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
                    match tool_manager.read_file(path).await {
                        Ok(content) => content,
                        Err(e) => format!("Error reading file: {}", e),
                    }
                } else {
                    "Error: Missing path argument".to_string()
                }
            }
            "write_file" => {
                let path = args.get("path").and_then(|v| v.as_str());
                let content = args.get("content").and_then(|v| v.as_str());
                if let (Some(p), Some(c)) = (path, content) {
                    match tool_manager.write_file(p, c, event_sender).await {
                        Ok(_) => "File written successfully".to_string(),
                        Err(e) => format!("Error writing file: {}", e),
                    }
                } else {
                    "Error: Missing path or content argument".to_string()
                }
            }
            "fetch_url" => {
                if let Some(url) = args.get("url").and_then(|v| v.as_str()) {
                    match tool_manager.fetch_url(url).await {
                        Ok(content) => content,
                        Err(e) => format!("Error fetching URL: {}", e),
                    }
                } else {
                    "Error: Missing url argument".to_string()
                }
            }
            "run_command" => {
                if let Some(cmd) = args.get("command").and_then(|v| v.as_str()) {
                    let envs = env_vars_shared.lock().await.clone();
                    match tool_manager.run_command(cmd, &envs).await {
                        Ok(output) => output,
                        Err(e) => format!("Error running command: {}", e),
                    }
                } else {
                    "Error: Missing command argument".to_string()
                }
            }
            "run_python" => {
                let script_path = args.get("script_path").and_then(|v| v.as_str());
                let script_args = args.get("args").and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>())
                    .unwrap_or_default();

                if let Some(path) = script_path {
                     let envs = env_vars_shared.lock().await.clone();
                     match tool_manager.run_python(path, &script_args, &envs).await {
                         Ok(output) => output,
                         Err(e) => format!("Error running python: {}", e),
                     }
                } else {
                    "Error: Missing script_path argument".to_string()
                }
            }
            "set_env" => {
                let key = args.get("key").and_then(|v| v.as_str());
                let value = args.get("value").and_then(|v| v.as_str());
                if let (Some(k), Some(v)) = (key, value) {
                    if k.len() > 100 || v.len() > 5000 {
                         "Error: key or value too large".to_string()
                    } else {
                         env_vars_shared.lock().await.insert(k.to_string(), v.to_string());
                         format!("Environment variable '{}' set.", k)
                    }
                } else {
                    "Error: Missing key or value".to_string()
                }
            }
            "get_env" => {
                if let Some(key) = args.get("key").and_then(|v| v.as_str()) {
                    match env_vars_shared.lock().await.get(key) {
                        Some(v) => v.clone(),
                        None => "Environment variable not set".to_string(),
                    }
                } else {
                    "Error: Missing key".to_string()
                }
            }
            "search_code" => {
                if let Some(pattern) = args.get("pattern").and_then(|v| v.as_str()) {
                    match tool_manager.search_code(pattern).await {
                        Ok(matches) => matches.join("\n"),
                        Err(e) => format!("Error searching code: {}", e),
                    }
                } else {
                    "Error: Missing pattern argument".to_string()
                }
            }
            "list_files" => {
                let path = args.get("path").and_then(|v| v.as_str());
                let depth = args.get("depth").and_then(|v| v.as_i64());
                match tool_manager.list_files(path, depth).await {
                    Ok(files) => format!("Files:\n{}", files.join("\n")),
                    Err(e) => format!("Error listing files: {}", e),
                }
            }
            "apply_patch" => {
                let path = args.get("path").and_then(|v| v.as_str());
                let original = args.get("original_snippet").and_then(|v| v.as_str());
                let new = args.get("new_snippet").and_then(|v| v.as_str());
                let start_line = args.get("start_line").and_then(|v| v.as_i64());
                
                if let (Some(p), Some(o), Some(n)) = (path, original, new) {
                    match tool_manager.apply_patch(p, o, n, start_line, event_sender).await {
                        Ok(_) => "Patch applied successfully".to_string(),
                        Err(e) => format!("Error applying patch: {}", e),
                    }
                } else {
                    "Error: Missing arguments (path, original_snippet, new_snippet)".to_string()
                }
            }
            #[cfg(feature = "knowledge")]
            "memory_search" => {
                if let Some(query) = args.get("query").and_then(|v| v.as_str()) {
                    let mut context_engine = context_engine_shared.lock().await;
                    if let Some(memory) = &mut context_engine.memory {
                        match memory.search(query, 5).await {
                            Ok(results) => {
                                if results.is_empty() {
                                    "No relevant results found in memory.".to_string()
                                } else {
                                    results.iter().map(|(doc, dist)| {
                                        format!("- [Score: {:.2}] {}:\n{}", 1.0 - dist, doc.path, doc.content.lines().take(3).collect::<Vec<_>>().join("\n"))
                                    }).collect::<Vec<_>>().join("\n\n")
                                }
                            }
                            Err(e) => format!("Error searching memory: {}", e),
                        }
                    } else {
                         "Error: Memory engine not initialized.".to_string()
                    }
                } else {
                    "Error: Missing query argument".to_string()
                }
            }
            #[cfg(feature = "knowledge")]
            "memory_index" => {
                let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                match context_engine_shared.lock().await.index_project(&root).await {
                    Ok(msg) => msg,
                    Err(e) => format!("Error indexing project: {}", e),
                }
            }
            _ => format!("Error: Unknown tool {}", tool_name),
        }
    }
}
