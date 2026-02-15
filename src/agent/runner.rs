use anyhow::Result;
use llm_connector::types::{Message, MessageBlock, Role, Tool};
use tracing::info;

use crate::agent::client::AgentClient;
use crate::engine::context::ContextEngine;
use crate::engine::tools::ToolManager;

pub struct AgentRunner {
    client: AgentClient,
    context_engine: ContextEngine,
    // Max iterations to prevent infinite loops
    max_iterations: usize,
}

impl AgentRunner {
    pub fn new(client: AgentClient) -> Result<Self> {
        let context_engine = ContextEngine::new()?;
        Ok(Self {
            client,
            context_engine,
            max_iterations: 10,
        })
    }

    pub async fn run(&mut self, task: &str) -> Result<String> {
        let mut messages = Vec::new();

        // 1. Context Retrieval (L1 - Project Scan)
        // TODO: Get root path from config or arguments. Assuming current dir for now.
        let root = std::env::current_dir()?;
        let files = self.context_engine.scan_project(&root);

        // System Prompt construction
        let system_prompt = format!(
            "You are Zene, an expert AI coding agent.\n\
             Current Project Structure:\n{:?}\n\n\
             Your goal is to complete the user's task. \
             You have access to tools to read and write files. \
             Always read a file before modifying it. \
             Think step-by-step.",
            files
        );

        messages.push(Message::system(&system_prompt));
        messages.push(Message::user(task));

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
                .chat_with_history(messages.clone(), Some(tools.clone()))
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

            messages.push(assistant_msg);

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

                let result_content = match tool_name {
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
                    _ => format!("Error: Unknown tool {}", tool_name),
                };

                // Add tool result to history
                // We create a Tool message.
                let tool_msg = Message {
                    role: Role::Tool,
                    content: vec![MessageBlock::text(&result_content)],
                    tool_call_id: Some(tool_call.id.clone()),
                    ..Default::default()
                };
                messages.push(tool_msg);
            }
        }

        Ok("Task stopped: Max iterations reached".to_string())
    }
}
