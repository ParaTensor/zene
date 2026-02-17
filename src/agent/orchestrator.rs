use anyhow::Result;
use tracing::{info, warn};
use std::sync::Arc;

use crate::agent::client::AgentClient;
use crate::agent::planner::Planner;
use crate::agent::executor::Executor;
use crate::agent::reflector::Reflector;
use crate::agent::compactor::SessionCompactor;
use crate::engine::context::ContextEngine;
use crate::engine::plan::{Task, TaskStatus};
use crate::engine::session::Session;
use crate::engine::tools::ToolManager;
use crate::engine::ui::UserInterface;
use crate::config::AgentConfig;
use crate::engine::contracts::AgentEvent;
use llm_connector::types::Message;
use tokio::sync::mpsc::UnboundedSender;

pub struct Orchestrator {
    planner: Planner,
    executor: Executor,
    reflector: Reflector,
    compactor: SessionCompactor,
    context_engine: ContextEngine,
    user_interface: Box<dyn UserInterface>,
    config: AgentConfig,
    event_sender: Option<UnboundedSender<AgentEvent>>,
}

impl Orchestrator {
    pub fn new(
        config: AgentConfig,
        planner_client: AgentClient,
        executor_client: AgentClient,
        reflector_client: AgentClient,
        tool_manager: Arc<ToolManager>,
        context_engine: ContextEngine,
        user_interface: Box<dyn UserInterface>,
    ) -> Self {
        let planner = Planner::new(planner_client);
        let executor = Executor::new(executor_client.clone(), tool_manager);
        let reflector = Reflector::new(reflector_client);
        let compactor = SessionCompactor::new(executor_client); // Use executor client for compaction

        Self {
            planner,
            executor,
            reflector,
            compactor,
            context_engine,
            user_interface,
            config,
            event_sender: None,
        }
    }

    pub fn with_event_sender(mut self, sender: UnboundedSender<AgentEvent>) -> Self {
        self.executor = self.executor.with_event_sender(sender.clone());
        self.event_sender = Some(sender);
        self
    }

    fn emit(&self, event: AgentEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event);
        }
    }

    pub async fn run(&mut self, goal: &str, session: &mut Session) -> Result<String> {
        // Step 1: Planning
        if !self.config.simple_mode && session.plan.is_none() {
            self.emit(AgentEvent::PlanningStarted);
            info!("Orchestrator: Initializing plan for goal: {}", goal);
            
            let root = std::env::current_dir()?;
            let files = self.context_engine.list_files(&root, Some(2));
            let project_context = format!("{:?}", files);

            match self.planner.create_plan(goal, &project_context).await {
                Ok(plan) => {
                    info!("Orchestrator: Plan created with {} tasks", plan.tasks.len());
                    self.emit(AgentEvent::PlanGenerated(plan.clone()));
                    session.plan = Some(plan);
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    self.emit(AgentEvent::Error { code: "PLANNING_FAILED".to_string(), message: err_msg.clone() });
                    warn!("Orchestrator: Planning failed: {}. Proceeding with legacy/ad-hoc mode not supported in strict P3 orchestrator yet.", err_msg);
                    // In a robust system we might fall back, but for now let's error or handle gracefully
                }
            }
        }

        // Step 2: Execution Loop
        let mut final_summary = String::new();

        if let Some(plan) = &mut session.plan {
             // Use index-based loop to allow dynamic task insertion
             let mut i = 0;
             while i < plan.tasks.len() {
                 // Skip non-pending tasks
                 if plan.tasks[i].status != TaskStatus::Pending {
                     i += 1;
                     continue;
                 }

                 let current_task = &mut plan.tasks[i];
                 info!("Orchestrator: Starting Task {}: {}", current_task.id, current_task.description);
                 self.emit(AgentEvent::TaskStarted { id: current_task.id, description: current_task.description.clone() });
                 current_task.status = TaskStatus::InProgress;

                 // Notify context in history
                 session.history.push(Message::system(&format!(
                    "Orchestrator: Switching context to Task {}: {}", 
                    current_task.id, current_task.description
                 )));

                 // Execute
                 let result = self.executor.execute_task(
                     current_task,
                     &session.history,
                     &mut session.env_vars,
                     &mut self.context_engine,
                     self.user_interface.as_ref()
                 ).await;

                 match result {
                     Ok(output) => {
                         info!("Orchestrator: Task {} Execution Finished", current_task.id);
                         current_task.result = Some(output.clone());
                         
                         // Update history with result
                         session.history.push(Message::system(&format!(
                            "Task '{}' execution result:\n{}", 
                            current_task.description, 
                            output.chars().take(500).collect::<String>()
                         )));

                         // Reflection
                         info!("Orchestrator: Reflecting on Task {}", current_task.id);
                         self.emit(AgentEvent::ReflectionStarted);
                         match self.reflector.review_task(current_task, &output).await {
                             Ok(review) => {
                                 if review.passed {
                                     info!("Orchestrator: Task {} Passed Review", current_task.id);
                                     self.emit(AgentEvent::ReflectionResult { passed: true, reason: review.reason.clone() });
                                     current_task.status = TaskStatus::Completed;
                                     final_summary.push_str(&format!("Task {}: Completed\n", current_task.id));
                                 } else {
                                     warn!("Orchestrator: Task {} Failed Review: {}", current_task.id, review.reason);
                                     current_task.status = TaskStatus::Failed;
                                     
                                     // Self-Healing: Create Fix Task
                                     let fix_desc = format!(
                                         "Fix issues in Task {}: {}. Suggestion: {}", 
                                         current_task.id, review.reason, review.suggestions.unwrap_or_default()
                                     );
                                     
                                     let fix_task = Task {
                                         id: plan.tasks.len() + 1,
                                         description: fix_desc,
                                         status: TaskStatus::Pending,
                                         result: None,
                                     };
                                     
                                     info!("Orchestrator: Adding Repair Task: {}", fix_task.description);
                                     self.emit(AgentEvent::ReflectionResult { passed: false, reason: review.reason.clone() });
                                     plan.tasks.insert(i + 1, fix_task);
                                     // Loop will naturally pick it up next increment
                                 }
                             }
                             Err(e) => {
                                 warn!("Reflector failed: {}. Assuming success.", e);
                                 current_task.status = TaskStatus::Completed;
                             }
                         }
                     }
                     Err(e) => {
                         warn!("Orchestrator: Task {} Execution Failed: {}", current_task.id, e);
                         current_task.status = TaskStatus::Failed;
                         session.history.push(Message::system(&format!(
                            "Task '{}' failed with error: {}", 
                            current_task.description, e
                         )));
                         return Ok(format!("Execution halted due to task failure: {}", e));
                     }
                 }

                 // Compaction
                 if let Err(e) = self.compactor.compact(&mut session.history).await {
                     warn!("Orchestrator: Compaction failed: {}", e);
                 }

                 i += 1;
             }
             
             if final_summary.is_empty() {
                 self.emit(AgentEvent::Finished("Plan completed".to_string()));
                 return Ok("Plan completed (or explicit empty plan).".to_string());
             }
             self.emit(AgentEvent::Finished(final_summary.clone()));
             return Ok(final_summary);
        } else {
            // No plan (Simple Mode or Planning failed) - Execute as single ad-hoc task
            // We can wrap the goal in a generic task
             let adhoc_task = Task {
                 id: 0,
                 description: goal.to_string(),
                 status: TaskStatus::Pending,
                 result: None,
             };
             
             let result = self.executor.execute_task(
                 &adhoc_task, 
                 &session.history, 
                 &mut session.env_vars, 
                 &mut self.context_engine, 
                 self.user_interface.as_ref()
             ).await?;
             
             return Ok(result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockUserInterface;
    use crate::engine::session::SessionManager;
    use crate::engine::session::store::InMemorySessionStore;
    use crate::config::{AgentConfig, RoleConfig};
    use llm_connector::types::ChatResponse;

    #[tokio::test]
    async fn test_orchestrator_simple_plan() {
        // Setup Mocks
        let ui = Box::new(MockUserInterface::new());
        let context_engine = ContextEngine::new().unwrap();
        let mut session_manager = SessionManager::new(Arc::new(InMemorySessionStore::new())).await.unwrap();
        let mut session = session_manager.create_session("test_orch".to_string());
        let tool_manager = Arc::new(ToolManager::new(None));
        
        // Construct dummy config
        let config = AgentConfig {
             planner: crate::config::RoleConfig { provider: "mock".to_string(), model: "mock".to_string(), api_key: "mock".to_string(), base_url: None },
             executor: crate::config::RoleConfig { provider: "mock".to_string(), model: "mock".to_string(), api_key: "mock".to_string(), base_url: None },
             reflector: crate::config::RoleConfig { provider: "mock".to_string(), model: "mock".to_string(), api_key: "mock".to_string(), base_url: None },
             mcp: crate::config::mcp::McpConfig::default(),
             simple_mode: false,
        };

        // 1. Planner Mock: Returns a plan with 1 task
        let planner_json = serde_json::json!({
            "id": "plan-1",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "gpt-4",
            "content": "```json\n{\n    \"tasks\": [\"Say hello\"]\n}\n```",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": [{"type": "text", "text": "```json\n{\n    \"tasks\": [\"Say hello\"]\n}\n```"}]
                },
                "finish_reason": "stop"
            }]
        });
        let planner_resp: ChatResponse = serde_json::from_value(planner_json).unwrap();
        let planner_client = AgentClient::mock(vec![planner_resp]);

        // 2. Executor Mock: Returns "Hello World"
        let executor_json = serde_json::json!({
            "id": "exec-1",
            "object": "chat.completion",
            "created": 1234567891,
            "model": "gpt-4",
            "content": "Hello World",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": [{"type": "text", "text": "Hello World"}]
                },
                "finish_reason": "stop"
            }]
        });
        let executor_resp: ChatResponse = serde_json::from_value(executor_json).unwrap();
        let executor_client = AgentClient::mock(vec![executor_resp]);

        // 3. Reflector Mock: Returns Passed
        let reflector_json = serde_json::json!({
            "id": "ref-1",
            "object": "chat.completion",
            "created": 1234567892,
            "model": "gpt-4",
            "content": "{\"passed\": true, \"reason\": \"Good\", \"suggestions\": null}",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": [{"type": "text", "text": "{\"passed\": true, \"reason\": \"Good\", \"suggestions\": null}"}]
                },
                "finish_reason": "stop"
            }]
        });
        let reflector_resp: ChatResponse = serde_json::from_value(reflector_json).unwrap();
        let reflector_client = AgentClient::mock(vec![reflector_resp]);

        let mut orchestrator = Orchestrator::new(
            config,
            planner_client,
            executor_client,
            reflector_client,
            tool_manager,
            context_engine,
            ui,
        );

        let result = orchestrator.run("Do something", &mut session).await.unwrap();
        
        assert!(result.contains("Task 1: Completed"));
        assert!(session.plan.is_some());
        assert_eq!(session.plan.as_ref().unwrap().tasks.len(), 1);
        assert_eq!(session.plan.as_ref().unwrap().tasks[0].status, TaskStatus::Completed);
    }
}


