use crate::engine::error::Result;
use tracing::{info, warn};
use std::sync::Arc;

use crate::agent::client::AgentClient;
use crate::agent::compactor::SessionCompactor;
use crate::agent::executor::Executor;
use crate::agent::planner::Planner;
use crate::agent::reflector::Reflector;
use crate::config::AgentConfig;
use crate::engine::context::ContextEngine;
use crate::engine::contracts::{AgentEvent, TokenUsage};
use crate::engine::error::ZeneError;
use crate::engine::plan::{Plan, Task, TaskStatus};
use crate::engine::runtime::CancellationToken;
use crate::engine::session::Session;
use crate::engine::strategy::ExecutionStrategy;
use crate::engine::tools::ToolManager;
use crate::engine::ui::UserInterface;
use llm_connector::types::Message;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;

pub struct Orchestrator {
    planner: Planner,
    executor: Executor,
    reflector: Reflector,
    compactor: SessionCompactor,
    context_engine: Arc<Mutex<ContextEngine>>,
    user_interface: Box<dyn UserInterface>,
    event_sender: Option<UnboundedSender<AgentEvent>>,
    cancellation_token: Option<CancellationToken>,
}

impl Orchestrator {
    pub fn new(
        _config: AgentConfig,
        planner_client: AgentClient,
        executor_client: AgentClient,
        reflector_client: AgentClient,
        tool_manager: Arc<ToolManager>,
        context_engine: Arc<Mutex<ContextEngine>>,
        user_interface: Box<dyn UserInterface>,
    ) -> Self {
        let planner = Planner::new(planner_client);
        let executor = Executor::new(executor_client.clone(), tool_manager);
        let reflector = Reflector::new(reflector_client);
        let compactor = SessionCompactor::new(executor_client);

        Self {
            planner,
            executor,
            reflector,
            compactor,
            context_engine,
            user_interface,
            event_sender: None,
            cancellation_token: None,
        }
    }

    pub fn with_event_sender(mut self, sender: UnboundedSender<AgentEvent>) -> Self {
        self.executor = self.executor.with_event_sender(sender.clone());
        self.event_sender = Some(sender);
        self
    }

    pub fn with_cancellation_token(mut self, token: CancellationToken) -> Self {
        self.executor = self.executor.with_cancellation_token(token.clone());
        self.cancellation_token = Some(token);
        self
    }

    fn emit(&self, event: AgentEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event);
        }
    }

    fn ensure_not_cancelled(&self) -> Result<()> {
        if self
            .cancellation_token
            .as_ref()
            .map(|token| token.is_cancelled())
            .unwrap_or(false)
        {
            return Err(ZeneError::Cancelled("run cancelled by host".to_string()));
        }

        Ok(())
    }

    #[tracing::instrument(skip(self, session), fields(session_id = %session.id))]
    pub async fn run(&mut self, goal: &str, session: &mut Session, strategy: ExecutionStrategy) -> Result<(String, TokenUsage)> {
        let strategy = crate::agent::strategies::resolve_strategy(strategy);
        strategy.run(self, goal, session).await
    }

    pub(crate) async fn ensure_plan(&mut self, goal: &str, session: &mut Session) -> Result<()> {
        self.ensure_not_cancelled()?;

        if session.plan.is_some() {
            return Ok(());
        }

        self.emit(AgentEvent::PlanningStarted);
        info!("Orchestrator: Initializing plan for goal: {}", goal);

        let root = std::env::current_dir()?;
        let files = self.context_engine.lock().await.list_files(&root, Some(2));
        let project_context = format!("{:?}", files);

        match self.planner.create_plan(goal, &project_context).await {
            Ok(plan) => {
                info!("Orchestrator: Plan created with {} tasks", plan.tasks.len());
                self.emit(AgentEvent::PlanGenerated(plan.clone()));
                session.plan = Some(plan);
            }
            Err(e) => {
                let err_msg = e.to_string();
                self.emit(AgentEvent::Error {
                    code: "PLANNING_FAILED".to_string(),
                    message: err_msg.clone(),
                });
                warn!(
                    "Orchestrator: Planning failed: {}. Proceeding with legacy/ad-hoc mode not supported in strict P3 orchestrator yet.",
                    err_msg
                );
            }
        }

        Ok(())
    }

    pub(crate) async fn execute_planned(&mut self, session: &mut Session) -> Result<(String, TokenUsage)> {
        self.ensure_not_cancelled()?;
        let mut total_usage = TokenUsage::default();
        let mut final_summary = String::new();

            let Some(mut plan) = session.plan.take() else {
                return Ok(("Plan completed (or explicit empty plan).".to_string(), total_usage));
            };

            let mut index = 0;
            while index < plan.tasks.len() {
                self.ensure_not_cancelled()?;

                if plan.tasks[index].status != TaskStatus::Pending {
                    index += 1;
                    continue;
                }

                let task_id = plan.tasks[index].id;
                let task_description = plan.tasks[index].description.clone();
                info!("Orchestrator: Starting Task {}: {}", task_id, task_description);
                self.emit(AgentEvent::TaskStarted {
                    id: task_id,
                    description: task_description.clone(),
                });
                plan.tasks[index].status = TaskStatus::InProgress;

                session.history.push(Message::system(&format!(
                    "Orchestrator: Switching context to Task {}: {}",
                    task_id, task_description
                )));

                let task = &mut plan.tasks[index];
                match self.execute_task(task, session).await {
                    Ok((output, usage, duration)) => {
                        total_usage.prompt_tokens += usage.prompt_tokens;
                        total_usage.completion_tokens += usage.completion_tokens;
                        total_usage.total_tokens += usage.total_tokens;

                        tracing::info!(metric = "zene_task_latency", value = duration.as_secs_f64(), task_id = task_id.to_string(), agent_role = "Orchestrator");
                        tracing::info!(metric = "zene_prompt_tokens", value = usage.prompt_tokens as f64, task_id = task_id.to_string(), agent_role = "Orchestrator");
                        tracing::info!(metric = "zene_completion_tokens", value = usage.completion_tokens as f64, task_id = task_id.to_string(), agent_role = "Orchestrator");
                        tracing::info!(metric = "zene_total_tokens", value = usage.total_tokens as f64, task_id = task_id.to_string(), agent_role = "Orchestrator");

                        if let Some(summary) = self.reflect_task(&mut plan, index, &output).await? {
                            final_summary.push_str(&summary);
                        }
                    }
                    Err(e) => {
                        warn!("Orchestrator: Task {} Execution Failed: {}", task_id, e);
                        plan.tasks[index].status = TaskStatus::Failed;
                        session.history.push(Message::system(&format!(
                            "Task '{}' failed with error: {}",
                            task_description, e
                        )));
                        session.plan = Some(plan);
                        return Ok((format!("Execution halted due to task failure: {}", e), total_usage));
                    }
                }

                if let Err(e) = self.compactor.compact(&mut session.history).await {
                    warn!("Orchestrator: Compaction failed: {}", e);
                }

                index += 1;
            }

            session.plan = Some(plan);

            if final_summary.is_empty() {
                return Ok(("Plan completed (or explicit empty plan).".to_string(), total_usage));
            }

            Ok((final_summary, total_usage))
    }

    pub(crate) async fn execute_direct(&mut self, goal: &str, session: &mut Session) -> Result<(String, TokenUsage)> {
        self.ensure_not_cancelled()?;

        let task = Task {
            id: 0,
            description: goal.to_string(),
            status: TaskStatus::Pending,
            result: None,
        };

        let start_time = std::time::Instant::now();
        let env_vars_shared = Arc::new(tokio::sync::Mutex::new(session.env_vars.clone()));

        let result = self.executor.execute_task(
            &task,
            &mut session.history,
            env_vars_shared.clone(),
            self.context_engine.clone(),
            self.user_interface.as_ref(),
        ).await?;
        let duration = start_time.elapsed();

        let (output, usage) = result;

        tracing::info!(metric = "zene_task_latency", value = duration.as_secs_f64(), session_id = "adhoc", agent_role = "Orchestrator");
        tracing::info!(metric = "zene_prompt_tokens", value = usage.prompt_tokens as f64, session_id = "adhoc", agent_role = "Orchestrator");
        tracing::info!(metric = "zene_completion_tokens", value = usage.completion_tokens as f64, session_id = "adhoc", agent_role = "Orchestrator");
        tracing::info!(metric = "zene_total_tokens", value = usage.total_tokens as f64, session_id = "adhoc", agent_role = "Orchestrator");

        session.env_vars = Arc::try_unwrap(env_vars_shared)
            .map_err(|_| "Failed to unwrap env_vars_shared")
            .unwrap()
            .into_inner();

        Ok((output, usage))
    }

    async fn execute_task(
        &mut self,
        task: &mut Task,
        session: &mut Session,
    ) -> Result<(String, TokenUsage, std::time::Duration)> {
        let env_vars_shared = Arc::new(tokio::sync::Mutex::new(session.env_vars.clone()));
        let start_time = std::time::Instant::now();
        let result = self.executor.execute_task(
            task,
            &mut session.history,
            env_vars_shared.clone(),
            self.context_engine.clone(),
            self.user_interface.as_ref(),
        ).await;
        let duration = start_time.elapsed();

        session.env_vars = Arc::try_unwrap(env_vars_shared)
            .map_err(|_| "Failed to unwrap env_vars_shared")
            .unwrap()
            .into_inner();

        let (output, usage) = result?;
        info!("Orchestrator: Task {} Execution Finished. Usage: {:?}", task.id, usage);
        task.result = Some(output.clone());

        session.history.push(Message::system(&format!(
            "Task '{}' execution result:\n{}",
            task.description,
            output.chars().take(500).collect::<String>(),
        )));

        Ok((output, usage, duration))
    }

    async fn reflect_task(
        &mut self,
        plan: &mut Plan,
        task_index: usize,
        output: &str,
    ) -> Result<Option<String>> {
        let task = &mut plan.tasks[task_index];
        self.ensure_not_cancelled()?;
        info!("Orchestrator: Reflecting on Task {}", task.id);
        self.emit(AgentEvent::ReflectionStarted);

        match self.reflector.review_task(task, output).await {
            Ok(review) => {
                if review.passed {
                    info!("Orchestrator: Task {} Passed Review", task.id);
                    self.emit(AgentEvent::ReflectionResult {
                        passed: true,
                        reason: review.reason.clone(),
                    });
                    task.status = TaskStatus::Completed;
                    Ok(Some(format!("Task {}: Completed\n", task.id)))
                } else {
                    warn!("Orchestrator: Task {} Failed Review: {}", task.id, review.reason);
                    task.status = TaskStatus::Failed;

                    let fix_desc = format!(
                        "Fix issues in Task {}: {}. Suggestion: {}",
                        task.id,
                        review.reason,
                        review.suggestions.unwrap_or_default()
                    );

                    let fix_task = Task {
                        id: plan.tasks.len() + 1,
                        description: fix_desc,
                        status: TaskStatus::Pending,
                        result: None,
                    };

                    info!("Orchestrator: Adding Repair Task: {}", fix_task.description);
                    self.emit(AgentEvent::ReflectionResult {
                        passed: false,
                        reason: review.reason.clone(),
                    });
                    plan.tasks.insert(task_index + 1, fix_task);
                    Ok(None)
                }
            }
            Err(e) => {
                warn!("Reflector failed: {}. Assuming success.", e);
                task.status = TaskStatus::Completed;
                Ok(Some(format!("Task {}: Completed\n", task.id)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockUserInterface;
    use crate::engine::error::ZeneError;
    use crate::engine::runtime::CancellationToken;
    use crate::engine::session::SessionManager;
    use crate::engine::session::store::InMemorySessionStore;
    use crate::config::AgentConfig;
    use llm_connector::types::ChatResponse;

    #[tokio::test]
    async fn test_orchestrator_simple_plan() {
        // Setup Mocks
        let ui = Box::new(MockUserInterface::new());
        let context_engine = ContextEngine::new(false).unwrap();
        let context_engine_shared = Arc::new(tokio::sync::Mutex::new(context_engine));
        let mut session_manager = SessionManager::new(Arc::new(InMemorySessionStore::new())).await.unwrap();
        let mut session = session_manager.create_session("test_orch".to_string());
        let tool_manager = Arc::new(ToolManager::new(None, context_engine_shared.clone()));
        
        // Construct dummy config
        let config = AgentConfig {
             planner: crate::config::RoleConfig { provider: "mock".to_string(), model: "mock".to_string(), api_key: "mock".to_string(), base_url: None, region: None },
             executor: crate::config::RoleConfig { provider: "mock".to_string(), model: "mock".to_string(), api_key: "mock".to_string(), base_url: None, region: None },
             reflector: crate::config::RoleConfig { provider: "mock".to_string(), model: "mock".to_string(), api_key: "mock".to_string(), base_url: None, region: None },
             mcp: crate::config::mcp::McpConfig::default(),
             simple_mode: false,
             use_semantic_memory: false,
             xtrace_endpoint: None,
             xtrace_token: None,
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
            context_engine_shared,
            ui,
        );

        let (result, _usage) = orchestrator.run("Test Task", &mut session, ExecutionStrategy::Planned).await.unwrap();
        
        assert!(result.contains("Task 1: Completed"));
        assert!(session.plan.is_some());
        assert_eq!(session.plan.as_ref().unwrap().tasks.len(), 1);
        assert_eq!(session.plan.as_ref().unwrap().tasks[0].status, TaskStatus::Completed);
    }

    #[tokio::test]
    async fn test_orchestrator_cancellation_before_start() {
        let ui = Box::new(MockUserInterface::new());
        let context_engine = ContextEngine::new(false).unwrap();
        let context_engine_shared = Arc::new(tokio::sync::Mutex::new(context_engine));
        let mut session_manager = SessionManager::new(Arc::new(InMemorySessionStore::new())).await.unwrap();
        let mut session = session_manager.create_session("test_orch_cancel".to_string());
        let tool_manager = Arc::new(ToolManager::new(None, context_engine_shared.clone()));

        let config = AgentConfig {
            planner: crate::config::RoleConfig { provider: "mock".to_string(), model: "mock".to_string(), api_key: "mock".to_string(), base_url: None, region: None },
            executor: crate::config::RoleConfig { provider: "mock".to_string(), model: "mock".to_string(), api_key: "mock".to_string(), base_url: None, region: None },
            reflector: crate::config::RoleConfig { provider: "mock".to_string(), model: "mock".to_string(), api_key: "mock".to_string(), base_url: None, region: None },
            mcp: crate::config::mcp::McpConfig::default(),
            simple_mode: false,
            use_semantic_memory: false,
            xtrace_endpoint: None,
            xtrace_token: None,
        };

        let planner_client = AgentClient::mock(vec![]);
        let executor_client = AgentClient::mock(vec![]);
        let reflector_client = AgentClient::mock(vec![]);

        let token = CancellationToken::default();
        token.cancel();

        let mut orchestrator = Orchestrator::new(
            config,
            planner_client,
            executor_client,
            reflector_client,
            tool_manager,
            context_engine_shared,
            ui,
        )
        .with_cancellation_token(token);

        let err = orchestrator.run("Test Task", &mut session, ExecutionStrategy::Planned).await.unwrap_err();
        assert!(matches!(err, ZeneError::Cancelled(_)));
    }

    #[tokio::test]
    async fn test_orchestrator_direct_strategy_skips_plan() {
        let ui = Box::new(MockUserInterface::new());
        let context_engine = ContextEngine::new(false).unwrap();
        let context_engine_shared = Arc::new(tokio::sync::Mutex::new(context_engine));
        let mut session_manager = SessionManager::new(Arc::new(InMemorySessionStore::new())).await.unwrap();
        let mut session = session_manager.create_session("test_orch_direct".to_string());
        let tool_manager = Arc::new(ToolManager::new(None, context_engine_shared.clone()));

        let config = AgentConfig {
            planner: crate::config::RoleConfig { provider: "mock".to_string(), model: "mock".to_string(), api_key: "mock".to_string(), base_url: None, region: None },
            executor: crate::config::RoleConfig { provider: "mock".to_string(), model: "mock".to_string(), api_key: "mock".to_string(), base_url: None, region: None },
            reflector: crate::config::RoleConfig { provider: "mock".to_string(), model: "mock".to_string(), api_key: "mock".to_string(), base_url: None, region: None },
            mcp: crate::config::mcp::McpConfig::default(),
            simple_mode: false,
            use_semantic_memory: false,
            xtrace_endpoint: None,
            xtrace_token: None,
        };

        let planner_client = AgentClient::mock(vec![]);
        let executor_json = serde_json::json!({
            "id": "exec-direct",
            "object": "chat.completion",
            "created": 1234567893,
            "model": "gpt-4",
            "content": "Direct execution complete",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": [{"type": "text", "text": "Direct execution complete"}]
                },
                "finish_reason": "stop"
            }]
        });
        let executor_resp: ChatResponse = serde_json::from_value(executor_json).unwrap();
        let executor_client = AgentClient::mock(vec![executor_resp]);
        let reflector_client = AgentClient::mock(vec![]);

        let mut orchestrator = Orchestrator::new(
            config,
            planner_client,
            executor_client,
            reflector_client,
            tool_manager,
            context_engine_shared,
            ui,
        );

        let (result, _usage) = orchestrator
            .run("Test Direct Task", &mut session, ExecutionStrategy::Direct)
            .await
            .unwrap();

        assert!(result.is_empty() || result == "Task stopped: Max iterations reached without definitive completion.");
        assert!(session.plan.is_none());
    }
}


