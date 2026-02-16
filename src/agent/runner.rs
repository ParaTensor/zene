use anyhow::Result;
use llm_connector::types::{Message, MessageBlock, Role, Tool};
use tracing::info;

use crate::agent::client::AgentClient;
use crate::engine::context::ContextEngine;
use crate::engine::session::Session;
use crate::engine::tools::ToolManager;
use crate::engine::ui::UserInterface;

pub struct AgentRunner {
    client: AgentClient,
    context_engine: ContextEngine,
    user_interface: Box<dyn UserInterface>,
    // Max iterations to prevent infinite loops
    max_iterations: usize,
}

impl AgentRunner {
    pub fn new(client: AgentClient, user_interface: Box<dyn UserInterface>) -> Result<Self> {
        let context_engine = ContextEngine::new()?;
        Ok(Self {
            client,
            context_engine,
            user_interface,
            max_iterations: 10,
        })
    }

    pub async fn run(&mut self, task: &str, session: &mut Session) -> Result<String> {
        // If session history is empty, initialize system prompt
        if session.history.is_empty() {
            // 1. Context Retrieval (L1 - Project Scan)
            // Get root path from config or arguments. Assuming current dir for now.
            let root = std::env::current_dir()?;
            // Only list top-level files to save context. Agent can explore deeper if needed.
            let files = self.context_engine.list_files(&root, Some(1));

            // System Prompt construction
            let system_prompt = format!(
                "You are Zene, an expert AI coding agent.\n\
                 Current Top-Level Project Structure:\n{:?}\n\n\
                 Your goal is to complete the user's task. \
                 \n\
                 **Capabilities & Tools**:\n\
                 1. EXPLORE: Use `list_files` to explore subdirectories.\n\
                 2. SEARCH: Use `search_code` to find relevant code patterns (grep). This is preferred over reading every file.\n\
                 3. READ: Use `read_file` to examine specific file contents.\n\
                 4. EDIT: Use `write_file` to create new files or overwrite small files.\n\
                 5. PATCH: Use `apply_patch` to modify large files by replacing specific code blocks (Search & Replace).\n\
                 6. EXECUTE: Use `run_command` for shell operations.\n\
                 \n\
                 Think step-by-step. Do not guess file paths.",
                files
            );
            session.history.push(Message::system(&system_prompt));
        }

        session.history.push(Message::user(task));

        // Convert ToolDefinitions to llm_connector Tools
        let tools: Vec<Tool> = ToolManager::list_tools()
            .into_iter()
            .map(|def| Tool::function(def.name, Some(def.description), def.input_schema))
            .collect();

        let mut current_iteration = 0;

        while current_iteration < self.max_iterations {
            current_iteration += 1;
            info!("Iteration {}/{}", current_iteration, self.max_iterations);

            // Call LLM
            let response = self
                .client
                .chat_with_history(session.history.clone(), Some(tools.clone()))
                .await?;

            // Add assistant response to history
            // We need to create a Message with the response content and tool calls
            let mut assistant_msg = Message::new(Role::Assistant, vec![]);

            if !response.content.is_empty() {
                assistant_msg
                    .content
                    .push(MessageBlock::text(&response.content));
            }

            if !response.tool_calls().is_empty() {
                assistant_msg.tool_calls = Some(response.tool_calls().to_vec());
            }

            session.history.push(assistant_msg);

            // Check for tool calls
            if response.tool_calls().is_empty() {
                return Ok(response.content);
            }

            // Execute tools
            for tool_call in response.tool_calls() {
                info!("Executing tool: {}", tool_call.function.name);

                let tool_name = tool_call.function.name.as_str();
                let args_str = &tool_call.function.arguments;

                // Parse arguments
                let args: serde_json::Value =
                    serde_json::from_str(args_str).unwrap_or(serde_json::Value::Null);

                let result_content = if (tool_name == "run_command" || tool_name == "write_file" || tool_name == "apply_patch")
                    && !self.user_interface.confirm_execution(tool_name, args_str)
                {
                    "User denied execution".to_string()
                } else {
                    match tool_name {
                        "read_file" => {
                            if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
                                match ToolManager::read_file(path) {
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
                                match ToolManager::write_file(p, c) {
                                    Ok(_) => "File written successfully".to_string(),
                                    Err(e) => format!("Error writing file: {}", e),
                                }
                            } else {
                                "Error: Missing path or content argument".to_string()
                            }
                        }
                        "fetch_url" => {
                            if let Some(url) = args.get("url").and_then(|v| v.as_str()) {
                                match ToolManager::fetch_url(url).await {
                                    Ok(content) => content,
                                    Err(e) => format!("Error fetching URL: {}", e),
                                }
                            } else {
                                "Error: Missing url argument".to_string()
                            }
                        }
                        "run_command" => {
                            if let Some(cmd) = args.get("command").and_then(|v| v.as_str()) {
                                match ToolManager::run_command(cmd) {
                                    Ok(output) => output,
                                    Err(e) => format!("Error running command: {}", e),
                                }
                            } else {
                                "Error: Missing command argument".to_string()
                            }
                        }
                        "search_code" => {
                            if let Some(pattern) = args.get("pattern").and_then(|v| v.as_str()) {
                                match ToolManager::search_code(pattern) {
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
                            match ToolManager::list_files(path, depth) {
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
                                match ToolManager::apply_patch(p, o, n, start_line) {
                                    Ok(_) => "Patch applied successfully".to_string(),
                                    Err(e) => format!("Error applying patch: {}", e),
                                }
                            } else {
                                "Error: Missing arguments (path, original_snippet, new_snippet)".to_string()
                            }
                        }
                        _ => format!("Error: Unknown tool {}", tool_name),
                    }
                };

                // Add tool result to history
                // We create a Tool message.
                let tool_msg = Message {
                    role: Role::Tool,
                    content: vec![MessageBlock::text(&result_content)],
                    tool_call_id: Some(tool_call.id.clone()),
                    ..Default::default()
                };
                session.history.push(tool_msg);
            }
        }

        Ok("Task stopped: Max iterations reached".to_string())
    }
}
