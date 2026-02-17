use anyhow::Result;
use llm_connector::types::{Message, MessageBlock, Role, Tool};
use tracing::{info, warn};

use crate::agent::client::AgentClient;
use crate::agent::planner::Planner;
use crate::agent::reflector::Reflector;
use crate::engine::context::ContextEngine;
use crate::agent::tool_handler::ToolHandler;
use crate::engine::plan::{Task, TaskStatus};
use crate::engine::session::Session;
use crate::engine::tools::ToolManager;
use crate::engine::ui::UserInterface;
use crate::config::AgentConfig;

pub struct AgentRunner {
    #[allow(dead_code)]
    config: AgentConfig,
    planner_client: AgentClient,
    executor_client: AgentClient,
    #[allow(dead_code)]
    reflector_client: AgentClient,
    context_engine: ContextEngine,
    user_interface: Box<dyn UserInterface>,
    max_iterations: usize,
}

impl AgentRunner {
    pub fn new(config: AgentConfig, user_interface: Box<dyn UserInterface>) -> Result<Self> {
        let planner_client = AgentClient::new(&config.planner)?;
        let executor_client = AgentClient::new(&config.executor)?;
        let reflector_client = AgentClient::new(&config.reflector)?;
        
        let context_engine = ContextEngine::new()?;
        
        Ok(Self {
            config,
            planner_client,
            executor_client,
            reflector_client,
            context_engine,
            user_interface,
            max_iterations: 10,
        })
    }

    // For now, we still use a single loop, but we use the EXECUTOR client for the main loop.
    // In the next phase (P3), we will split logic to use planner and reflector.
    pub async fn run(&mut self, task: &str, session: &mut Session) -> Result<String> {
        // Step 1: Initialize Plan if needed
        if session.plan.is_none() {
             info!("Initializing new plan for task: {}", task);
             
             // Context Retrieval (L1 - Project Scan) for planning
             let root = std::env::current_dir()?;
             let files = self.context_engine.list_files(&root, Some(2));
             let project_context = format!("{:?}", files);

             let planner = Planner::new(self.planner_client.clone());
             match planner.create_plan(task, &project_context).await {
                 Ok(plan) => {
                     info!("Plan created with {} tasks", plan.tasks.len());
                     session.plan = Some(plan);
                 }
                 Err(e) => {
                     warn!("Failed to create plan: {}. Proceeding with ad-hoc execution.", e);
                     // Fallback to legacy mode if planning fails (e.g. model error)
                 }
             }
        }

        // Step 2: Execute Plan Loop
        let mut final_result = String::new();

        // Check if we have a valid plan
        if let Some(plan) = &mut session.plan {
            // Use while loop with index to allow dynamic insertion of tasks
            let mut i = 0;
            while i < plan.tasks.len() {
                // Check if task is pending
                if plan.tasks[i].status != TaskStatus::Pending {
                    i += 1;
                    continue;
                }

                let current_task = &mut plan.tasks[i];
                info!("Executing Task {}: {}", current_task.id, current_task.description);
                
                // Add context to history about what we are working on
                session.history.push(Message::system(&format!(
                    "Switching context to Task {}: {}", 
                    current_task.id, current_task.description
                )));

                current_task.status = TaskStatus::InProgress;

                match self.execute_task(current_task, session.history.clone(), &mut session.env_vars).await {
                    Ok(output) => {
                        info!("Task {} Completed", current_task.id);
                        current_task.result = Some(output.clone());
                        final_result.push_str(&format!("Task {}: Completed\n", current_task.id));
                        
                        // Add task completion note to history
                        session.history.push(Message::system(&format!(
                            "Task '{}' completed. Summary of work:\n{}", 
                            current_task.description, 
                            output.chars().take(500).collect::<String>()
                        )));

                        // 3. Reflect (Self-Healing)
                        let reflector = Reflector::new(self.reflector_client.clone());
                        match reflector.review_task(current_task, &output).await {
                            Ok(review) => {
                                if review.passed {
                                    info!("Reflector APPROVED Task {}", current_task.id);
                                    current_task.status = TaskStatus::Completed;
                                } else {
                                    warn!("Reflector REJECTED Task {}: {}", current_task.id, review.reason);
                                    current_task.status = TaskStatus::Failed;
                                    
                                    // Create a Fix Task
                                    let fix_description = format!(
                                        "Fix issues in previous task ({}): {}. Suggestion: {}", 
                                        current_task.description, 
                                        review.reason, 
                                        review.suggestions.unwrap_or_default()
                                    );
                                    
                                    let fix_task = Task {
                                        id: plan.tasks.len() + 1, // Simple ID generation
                                        description: fix_description,
                                        status: TaskStatus::Pending,
                                        result: None,
                                    };
                                    
                                    info!("Adding FIX Task: {}", fix_task.description);
                                    plan.tasks.insert(i + 1, fix_task);
                                    // Don't increment i, so we process the next task (which is the fix task) immediately
                                    // Actually, we increment i at the end of loop, so inserting at i+1 puts it next.
                                }
                            }
                            Err(e) => {
                                warn!("Reflector failed: {}. Assuming task passed.", e);
                                current_task.status = TaskStatus::Completed;
                            }
                        }
                    }
                    Err(e) => {
                        current_task.status = TaskStatus::Failed;
                        let error_msg = format!("Task {} Failed: {}", current_task.id, e);
                        warn!("{}", error_msg);
                        final_result.push_str(&error_msg);
                        
                        session.history.push(Message::system(&format!(
                            "Task '{}' failed. Error: {}", 
                            current_task.description, e
                        )));
                        
                        return Ok(final_result); // Stop on failure for now
                    }
                }
                i += 1;
            }
            
            if final_result.is_empty() {
                return Ok("All tasks in the plan are already completed.".to_string());
            }
            return Ok(final_result);
        }

        // Legacy / Ad-hoc execution path (if no plan exists)
        self.execute_legacy_loop(task, session).await
    }

    // Extracted legacy logic for backward compatibility or simple tasks
    async fn execute_legacy_loop(&self, task: &str, session: &mut Session) -> Result<String> {
        let client = &self.executor_client;

        if session.history.is_empty() {
            // 1. Context Retrieval (L1 - Project Scan)
            let root = std::env::current_dir()?;
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

        // ... (Reuse existing loop logic via execute_task but slightly modified for ad-hoc)
        // Actually, let's just create a dummy task wrapper to reuse `execute_task` logic
        // But `execute_task` expects a specific prompt structure.
        
        // Let's implement `execute_adhoc` which is similar to the old loop
        
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
            let response = client
                .chat_with_history(session.history.clone(), Some(tools.clone()))
                .await?;

            // Add assistant response to history
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
                // ... (Tool execution logic - same as before)
                info!("Executing tool: {}", tool_call.function.name);
                let tool_name = tool_call.function.name.as_str();
                let args_str = &tool_call.function.arguments;
                let args: serde_json::Value = serde_json::from_str(args_str).unwrap_or(serde_json::Value::Null);

                let result_content = ToolHandler::execute(
                    self.user_interface.as_ref(),
                    tool_name,
                    &args,
                    args_str,
                    &mut session.env_vars
                ).await;

                // Add tool result to history
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

    async fn execute_task(&self, task: &Task, mut history: Vec<Message>, env_vars: &mut std::collections::HashMap<String, String>) -> Result<String> {
        let client = &self.executor_client;
        
        // Add specific instruction for this task
        // We append this as a User message to guide the Executor
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

        let tools: Vec<Tool> = ToolManager::list_tools()
            .into_iter()
            .map(|def| Tool::function(def.name, Some(def.description), def.input_schema))
            .collect();

        let mut current_iteration = 0;
        // Limit task execution to fewer steps to prevent getting stuck
        let task_max_iterations = 8; 

        while current_iteration < task_max_iterations {
            current_iteration += 1;
            info!("  [Task {}] Iteration {}/{}", task.id, current_iteration, task_max_iterations);

            // Call LLM
            let response = client
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

            if response.tool_calls().is_empty() {
                // Task considered done
                return Ok(response.content);
            }

            for tool_call in response.tool_calls() {
                info!("  Executing tool: {}", tool_call.function.name);
                let tool_name = tool_call.function.name.as_str();
                let args_str = &tool_call.function.arguments;
                let args: serde_json::Value = serde_json::from_str(args_str).unwrap_or(serde_json::Value::Null);

                let result_content = ToolHandler::execute(
                    self.user_interface.as_ref(),
                    tool_name,
                    &args,
                    args_str,
                    env_vars
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

        Ok("Task stopped: Max iterations reached".to_string())
    }


}

