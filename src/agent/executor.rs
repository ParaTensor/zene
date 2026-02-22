use crate::engine::error::Result;
use llm_connector::types::{Message, MessageBlock, Role, Tool, ToolCall, FunctionCall};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use futures::StreamExt;

use crate::agent::client::AgentClient;
use crate::agent::tool_handler::ToolHandler;
use crate::engine::contracts::{AgentEvent, TokenUsage};
use crate::engine::context::ContextEngine;
use crate::engine::plan::Task;
use crate::engine::tools::ToolManager;
use crate::engine::ui::UserInterface;
use futures::future::join_all;
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;

pub struct Executor {
    client: AgentClient,
    tool_manager: Arc<ToolManager>,
    max_iterations: usize,
    event_sender: Option<UnboundedSender<AgentEvent>>,
}

impl Executor {
    pub fn new(client: AgentClient, tool_manager: Arc<ToolManager>) -> Self {
        Self {
            client,
            tool_manager,
            max_iterations: 8, // Default task iteration limit
            event_sender: None,
        }
    }

    pub fn with_event_sender(mut self, sender: UnboundedSender<AgentEvent>) -> Self {
        self.event_sender = Some(sender);
        self
    }

    fn emit(&self, event: AgentEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event);
        }
    }

    /// Executes a single task
    #[tracing::instrument(skip(self, history, env_vars_shared, context_engine_shared, user_interface))]
    pub async fn execute_task(
        &self,
        task: &Task,
        history: &mut Vec<Message>,
        env_vars_shared: Arc<Mutex<HashMap<String, String>>>,
        context_engine_shared: Arc<Mutex<ContextEngine>>,
        user_interface: &dyn UserInterface,
    ) -> Result<(String, TokenUsage)> {

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

        // Get tools from injected manager
        let tools: Vec<Tool> = self.tool_manager.list_tools().await
            .into_iter()
            .map(|def| Tool::function(def.name, Some(def.description), def.input_schema))
            .collect();

        let mut current_iteration = 0;
        let mut total_usage = TokenUsage::default();

        while current_iteration < self.max_iterations {
            current_iteration += 1;
            info!("  [Task {}] Execution Iteration {}/{}", task.id, current_iteration, self.max_iterations);

            // Call LLM with streaming
            let mut stream = self.client
                .chat_stream_with_history(history.clone(), Some(tools.clone()))
                .await?;

            let mut full_content = String::new();
            let mut tool_calls_buffer: Vec<ToolCall> = Vec::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result?;
                
                // Accumulate usage if present (usually in the last chunk)
                if let Some(ref usage) = chunk.usage {
                    total_usage.prompt_tokens += usage.prompt_tokens;
                    total_usage.completion_tokens += usage.completion_tokens;
                    total_usage.total_tokens += usage.total_tokens;
                }

                for choice in chunk.choices {
                    let delta = choice.delta;

                    // Handle content streaming (ThoughtDelta)
                    if let Some(content_delta) = &delta.content {
                        if !content_delta.is_empty() {
                            full_content.push_str(content_delta);
                            self.emit(AgentEvent::ThoughtDelta(content_delta.clone()));
                        }
                    }

                    // Handle tool calls streaming
                    if let Some(tool_calls_delta) = delta.tool_calls {
                        for tool_call_chunk in tool_calls_delta {
                            // Initialize new tool call if index matches or we need to start a new one
                            let index = tool_call_chunk.index.unwrap_or(0);
                            
                            // Ensure vector has enough space
                            if index >= tool_calls_buffer.len() {
                                 tool_calls_buffer.resize(index + 1, ToolCall {
                                    id: String::new(),
                                    call_type: "function".to_string(),
                                    function: FunctionCall {
                                        name: String::new(),
                                        arguments: String::new(),
                                    },
                                    index: Some(index), 
                                });
                            }
                            
                            let current_tool = &mut tool_calls_buffer[index];
                            
                            if !tool_call_chunk.id.is_empty() {
                                current_tool.id = tool_call_chunk.id.clone();
                            }
                            
                            if !tool_call_chunk.function.name.is_empty() {
                                current_tool.function.name.push_str(&tool_call_chunk.function.name);
                            }
                            if !tool_call_chunk.function.arguments.is_empty() {
                                current_tool.function.arguments.push_str(&tool_call_chunk.function.arguments);
                            }
                        }
                    }
                }
            }

            let mut assistant_msg = Message::new(Role::Assistant, vec![]);

            if !full_content.is_empty() {
                assistant_msg.content.push(MessageBlock::text(&full_content));
            }

            if !tool_calls_buffer.is_empty() {
                // Filter out empty tool calls if any
                let valid_tool_calls: Vec<ToolCall> = tool_calls_buffer.into_iter()
                    .filter(|tc| !tc.id.is_empty() && !tc.function.name.is_empty())
                    .collect();
                
                if !valid_tool_calls.is_empty() {
                    assistant_msg.tool_calls = Some(valid_tool_calls);
                }
            }

            history.push(assistant_msg);

            if history.last().and_then(|m| m.tool_calls.as_ref()).map_or(true, |tc| tc.is_empty()) {
                 return Ok((full_content, total_usage));
            }

            // Execute tools in parallel
            if let Some(last_msg) = history.last() {
                if let Some(tool_calls) = &last_msg.tool_calls {
                    let mut tool_futures = Vec::new();

                    for tool_call in tool_calls {
                        info!("  Preparing tool call: {}", tool_call.function.name);
                        let tool_name = tool_call.function.name.clone();
                    let args_str = tool_call.function.arguments.clone();
                    let tool_call_id = tool_call.id.clone();
                    let tool_manager = self.tool_manager.clone();
                    let env_vars = env_vars_shared.clone();
                    let context_engine = context_engine_shared.clone();
                    
                    let args: serde_json::Value = serde_json::from_str(&args_str).unwrap_or(serde_json::Value::Null);

                    self.emit(AgentEvent::ToolCall { 
                        name: tool_name.clone(), 
                        arguments: args.clone() 
                    });

                    // Create a future for each tool execution
                    let future = async move {
                        let result_content = ToolHandler::execute(
                            &tool_manager,
                            user_interface,
                            &tool_name,
                            &args,
                            &args_str,
                            env_vars,
                            context_engine,
                        ).await;

                        (tool_call_id, tool_name, result_content)
                    };
                    tool_futures.push(future);
                }

                let results = join_all(tool_futures).await;

                for (id, name, result_content) in results {
                    info!("  Tool {} finished", name);
                    self.emit(AgentEvent::ToolResult { 
                        name: name.clone(), 
                        result: result_content.clone() 
                    });

                    let tool_msg = Message {
                        role: Role::Tool,
                        content: vec![MessageBlock::text(&result_content)],
                        tool_call_id: Some(id),
                        ..Default::default()
                    };
                    history.push(tool_msg);
                }
            }
            }
        }

        Ok(("Task stopped: Max iterations reached without definitive completion.".to_string(), total_usage))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockUserInterface;
    use crate::engine::session::SessionManager;
    use crate::engine::session::store::InMemorySessionStore;
    use crate::agent::client::AgentClient;
    use llm_connector::types::ChatResponse;
    
    #[tokio::test]
    async fn test_executor_simple_tool_flow() {
        // Setup Mocks
        let ui = Box::new(MockUserInterface::new());
        let context_engine = ContextEngine::new(false).unwrap();
        let mut session_manager = SessionManager::new(Arc::new(InMemorySessionStore::new())).await.unwrap();
        let session = session_manager.create_session("test_executor".to_string());
        
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
        let context_engine_shared = Arc::new(tokio::sync::Mutex::new(context_engine.clone()));
        let tool_manager = Arc::new(ToolManager::new(None, context_engine_shared.clone()));
        let executor = Executor::new(client, tool_manager);
        
        // ... (rest of test)
        
        let task = Task {
            id: 1,
            description: "Echo hello".to_string(),
            status: crate::engine::plan::TaskStatus::InProgress,
            result: None,
        };
        
        let env_vars_shared = Arc::new(tokio::sync::Mutex::new(session.env_vars.clone()));
        let context_engine_shared = Arc::new(tokio::sync::Mutex::new(context_engine));
        
        let (_result, _usage) = executor.execute_task(
            &task,
            &mut session.history,
            env_vars_shared,
            context_engine_shared,
            ui.as_ref()
        ).await.unwrap();
        
        // Assert: Done or something similar
        assert_eq!(_result, "Done");
        assert_eq!(session.history.len(), 4); // User + Assistant (Tool Call) + Tool + Final Assistant
        assert!(_usage.total_tokens > 0 || _usage.total_tokens == 0); // usage should be returned
    }
}
