use anyhow::Result;
use llm_connector::types::{Message, MessageBlock, Role, Tool};
use std::collections::HashMap;
use tracing::info;

use crate::agent::client::AgentClient;
use crate::agent::tool_handler::ToolHandler;
use crate::engine::context::ContextEngine;
use crate::engine::plan::Task;
use crate::engine::tools::ToolManager;
use crate::engine::ui::UserInterface;

pub struct Executor {
    client: AgentClient,
    max_iterations: usize,
}

impl Executor {
    pub fn new(client: AgentClient) -> Self {
        Self {
            client,
            max_iterations: 8, // Default task iteration limit
        }
    }

    /// Executes a single task
    pub async fn execute_task(
        &self,
        task: &Task,
        session_history: &[Message],
        env_vars: &mut HashMap<String, String>,
        context_engine: &mut ContextEngine,
        user_interface: &dyn UserInterface,
    ) -> Result<String> {
        // Clone history to create a local context (we don't want to pollute global history with every intermediate tool call step, 
        // OR we do? In the original runner, history was cloned passed in. 
        // WAIT: In P3, we generally want the global history to reflect major steps, but intermediate thought process might be ephemeral or summarized.
        // For now, let's stick to the current behavior: usage of a local history vector seeded with global history.
        let mut history = session_history.to_vec();

        // Add task-specific system prompt/instruction
        let task_prompt = format!(
            "Current Active Task: {}\n\
             Description: {}\n\
             \n\
             Please execute this specific task. \
             Use the tools available. \
             If you need to modify files, check if they exist first. \
             When the task is complete, provide a summary of what you did.",
            task.id, task.description
        );
        history.push(Message::user(&task_prompt));

        // Get tools
        let tools: Vec<Tool> = ToolManager::list_tools().await
            .into_iter()
            .map(|def| Tool::function(def.name, Some(def.description), def.input_schema))
            .collect();

        let mut current_iteration = 0;

        while current_iteration < self.max_iterations {
            current_iteration += 1;
            info!("  [Task {}] Execution Iteration {}/{}", task.id, current_iteration, self.max_iterations);

            // Call LLM
            let response = self.client
                .chat_with_history(history.clone(), Some(tools.clone()))
                .await?;

            let mut assistant_msg = Message::new(Role::Assistant, vec![]);

            if !response.content.is_empty() {
                assistant_msg.content.push(MessageBlock::text(&response.content));
            }

            if !response.tool_calls().is_empty() {
                assistant_msg.tool_calls = Some(response.tool_calls().to_vec());
            }

            history.push(assistant_msg);

            // If no tool calls, we assume the task is done (or the model is asking a question/providing final answer)
            if response.tool_calls().is_empty() {
                return Ok(response.content);
            }

            // Execute tools
            for tool_call in response.tool_calls() {
                info!("  Executing tool: {}", tool_call.function.name);
                let tool_name = tool_call.function.name.as_str();
                let args_str = &tool_call.function.arguments;
                let args: serde_json::Value = serde_json::from_str(args_str).unwrap_or(serde_json::Value::Null);

                let result_content = ToolHandler::execute(
                    user_interface,
                    tool_name,
                    &args,
                    args_str,
                    env_vars,
                    context_engine
                ).await;

                let tool_msg = Message {
                    role: Role::Tool,
                    content: vec![MessageBlock::text(&result_content)],
                    tool_call_id: Some(tool_call.id.clone()),
                    ..Default::default()
                };
                history.push(tool_msg);
            }
        }

        Ok("Task stopped: Max iterations reached without definitive completion.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockUserInterface;
    use crate::engine::session::SessionManager;
    use llm_connector::types::ChatResponse;
    
    #[tokio::test]
    async fn test_executor_simple_tool_flow() {
        // Setup Mocks
        let ui = Box::new(MockUserInterface::new());
        let mut context_engine = ContextEngine::new().unwrap();
        let mut session_manager = SessionManager::new().unwrap();
        let mut session = session_manager.create_session("test_executor".to_string());
        
        // Construct responses using JSON to bypass missing ChatChoice type export
        let tool_call_json = serde_json::json!({
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "gpt-3.5-turbo-0613",
            "content": "",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": [],
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": {
                            "name": "run_command",
                            "arguments": "{\"command\": \"echo hello\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }]
        });
        
        let tool_call_resp: ChatResponse = serde_json::from_value(tool_call_json).unwrap();
        
        let final_json = serde_json::json!({
            "id": "chatcmpl-124",
            "object": "chat.completion",
            "created": 1677652289,
            "model": "gpt-3.5-turbo-0613",
            "content": "Done",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": [{"type": "text", "text": "Done"}]
                },
                "finish_reason": "stop"
            }]
        });

        let final_resp: ChatResponse = serde_json::from_value(final_json).unwrap();
        
        let client = AgentClient::mock(vec![tool_call_resp, final_resp]);
        let executor = Executor::new(client);
        
        // ... (rest of test)
        
        let task = Task {
            id: 1,
            description: "Echo hello".to_string(),
            status: crate::engine::plan::TaskStatus::InProgress,
            result: None,
        };
        
        let result = executor.execute_task(
            &task,
            &session.history,
            &mut session.env_vars,
            &mut context_engine,
            ui.as_ref()
        ).await.unwrap();
        
        assert_eq!(result, "Done");
    }
}
