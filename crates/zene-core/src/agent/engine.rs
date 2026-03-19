use crate::engine::context::ContextEngine;
use crate::engine::error::Result;
use crate::engine::error::ZeneError;
use crate::engine::runtime::{CancellationToken, RunHandle, RunSnapshot, RunStatus};
use crate::engine::strategy::ExecutionStrategy;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::error;

pub use crate::config::AgentConfig;
pub use crate::engine::contracts::{AgentEvent, RunRequest, RunResult, TokenUsage};
use crate::engine::session::SessionManager;
use crate::engine::session::store::SessionStore;
use crate::engine::tools::ToolManager;
use crate::engine::ui::AutoUserInterface;
use crate::agent::runner::AgentRunner;
use crate::engine::mcp::manager::McpManager;

use crate::engine::contracts::EventEnvelope;
use chrono::Utc;

#[derive(Clone)]
pub struct ZeneEngine {
    pub config: AgentConfig,
    pub tool_manager: Arc<ToolManager>,
    pub context_engine: Arc<Mutex<ContextEngine>>,
    pub session_manager: Arc<Mutex<SessionManager>>,
    runs: Arc<Mutex<HashMap<String, RunEntry>>>,
}

struct RunEntry {
    snapshot: RunSnapshot,
    cancellation: CancellationToken,
}

impl ZeneEngine {
    fn classify_error_code(err: &ZeneError) -> String {
        match err {
            ZeneError::ConfigError(_) => "INVALID_REQUEST".to_string(),
            ZeneError::ProviderError(_) | ZeneError::LlmConnectorError(_) | ZeneError::ModelError(_) => {
                "PROVIDER_ERROR".to_string()
            }
            ZeneError::ReqwestError(_) => "PROVIDER_DOWN".to_string(),
            ZeneError::Cancelled(_) => "CANCELED".to_string(),
            _ => "INTERNAL".to_string(),
        }
    }

    pub async fn new(config: AgentConfig, session_store: Arc<dyn SessionStore>) -> Result<Self> {
        let context_engine = Arc::new(Mutex::new(ContextEngine::new(config.use_semantic_memory)?));
        let mcp_manager = Arc::new(McpManager::new(config.mcp.clone()));
        let tool_manager = Arc::new(ToolManager::new(Some(mcp_manager), context_engine.clone()));
        let session_manager = Arc::new(Mutex::new(SessionManager::new(session_store).await?));
        
        Ok(Self {
            config,
            tool_manager,
            context_engine,
            session_manager,
            runs: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn run(&self, request: RunRequest) -> Result<RunResult> {
        let run_id = uuid::Uuid::new_v4().to_string();
        let cancellation = CancellationToken::default();
        self.register_run(run_id.clone(), request.session_id.clone(), cancellation.clone()).await;
        self.execute_run(run_id, request, None, cancellation).await
    }

    pub async fn run_stream(&self, request: RunRequest) -> Result<mpsc::UnboundedReceiver<AgentEvent>> {
        let handle = self.submit(request).await?;
        Ok(handle.events)
    }

    pub async fn submit(&self, request: RunRequest) -> Result<RunHandle> {
        let (tx, rx) = mpsc::unbounded_channel::<AgentEvent>();
        let engine = self.clone();
        let run_id = uuid::Uuid::new_v4().to_string();
        let run_id_for_handle = run_id.clone();
        let session_id = request.session_id.clone();
        let cancellation = CancellationToken::default();

        self.register_run(run_id.clone(), session_id.clone(), cancellation.clone()).await;
        
        tokio::spawn(async move {
            match engine.execute_run(run_id, request, Some(tx.clone()), cancellation).await {
                Ok(res) => {
                    let _ = tx.send(AgentEvent::Finished(res.output));
                }
                Err(e) => {
                    let _ = tx.send(AgentEvent::Error { 
                        code: "RUN_FAILED".to_string(), 
                        message: e.to_string() 
                    });
                }
            }
        });

        Ok(RunHandle {
            run_id: run_id_for_handle,
            session_id,
            events: rx,
        })
    }

    pub async fn run_envelope_stream(&self, request: RunRequest) -> Result<mpsc::UnboundedReceiver<EventEnvelope>> {
        let (tx, mut rx) = mpsc::unbounded_channel::<AgentEvent>();
        let (out_tx, out_rx) = mpsc::unbounded_channel::<EventEnvelope>();
        
        let engine = self.clone();
        let session_id = request.session_id.clone();
        let run_id = uuid::Uuid::new_v4().to_string();
        let run_id_for_events = run_id.clone();
        let cancellation = CancellationToken::default();
        self.register_run(run_id.clone(), session_id.clone(), cancellation.clone()).await;
        
        // Spawn the runner
        tokio::spawn(async move {
            match engine.execute_run(run_id.clone(), request, Some(tx.clone()), cancellation).await {
                Ok(res) => {
                    let _ = tx.send(AgentEvent::Finished(res.output));
                }
                Err(e) => {
                    let _ = tx.send(AgentEvent::Error { 
                        code: "RUN_FAILED".to_string(), 
                        message: e.to_string() 
                    });
                }
            }
        });

        // Spawn the envelope processor
        let session_manager = self.session_manager.clone();
        let run_id_clone = run_id_for_events;
        let session_id_clone = session_id.clone();
        
        tokio::spawn(async move {
            let mut seq = 0;
            while let Some(event) = rx.recv().await {
                seq += 1;
                // Convert to JSON Value for payload
                let payload = serde_json::to_value(&event).unwrap_or_default();
                
                // Detailed type mapping
                let event_type = match &event {
                    AgentEvent::PlanningStarted => "PlanningStarted",
                    AgentEvent::PlanGenerated(_) => "PlanGenerated",
                    AgentEvent::TaskStarted { .. } => "TaskStarted",
                    AgentEvent::ThoughtDelta(_) => "ThoughtDelta",
                    AgentEvent::ToolCall { .. } => "ToolCall",
                    AgentEvent::ToolOutputDelta(_) => "ToolOutputDelta",
                    AgentEvent::ToolResult { .. } => "ToolResult",
                    AgentEvent::FileStateChanged { .. } => "FileStateChanged",
                    AgentEvent::ReflectionStarted => "ReflectionStarted",
                    AgentEvent::ReflectionResult { .. } => "ReflectionResult",
                    AgentEvent::Finished(_) => "Finished",
                    AgentEvent::Error { .. } => "Error",
                }.to_string();
                
                let envelope = EventEnvelope {
                    run_id: run_id_clone.clone(),
                    session_id: session_id_clone.clone(),
                    seq,
                    ts: Utc::now(),
                    event_type,
                    payload,
                };

                // Persist
                {
                    let sm = session_manager.lock().await;
                    if let Err(e) = sm.append_event(&session_id_clone, &envelope).await {
                         error!("Failed to persist event: {}", e);
                    }
                }

                // Forward
                if out_tx.send(envelope).is_err() {
                    break;
                }
            }
        });

        Ok(out_rx)
    }

    pub async fn get_run_snapshot(&self, run_id: &str) -> Option<RunSnapshot> {
        let runs = self.runs.lock().await;
        runs.get(run_id).map(|entry| entry.snapshot.clone())
    }

    pub async fn cancel_run(&self, run_id: &str) -> bool {
        let mut runs = self.runs.lock().await;
        if let Some(entry) = runs.get_mut(run_id) {
            entry.cancellation.cancel();
            entry.snapshot.mark_cancelled();
            return true;
        }
        false
    }

    async fn execute_run(
        &self, 
        run_id: String,
        request: RunRequest, 
        event_sender: Option<mpsc::UnboundedSender<AgentEvent>>,
        cancellation: CancellationToken,
    ) -> Result<RunResult> {
        self.update_run_status(&run_id, RunStatus::Running).await;

        // 1. Prepare UI (Auto for library usage)
        let ui = Box::new(AutoUserInterface);

        // 2. Setup Runner
        let runner_result = AgentRunner::new(
            self.config.clone(), 
            self.tool_manager.clone(), 
            self.context_engine.clone(),
            ui
        );
        let mut runner = match runner_result {
            Ok(runner) => runner,
            Err(err) => {
                match &err {
                    ZeneError::Cancelled(_) => self.mark_run_cancelled(&run_id).await,
                    _ => self.mark_run_failed(&run_id, &err).await,
                }
                return Err(err);
            }
        };
        runner = runner.with_cancellation_token(cancellation.clone());
        if let Some(sender) = event_sender.clone() {
            runner = runner.with_event_sender(sender);
        }

        // 3. Get/Create Session
        let session_clone = {
            let mut session_manager = self.session_manager.lock().await;
            // Merge request env_vars if provided
            let session = session_manager.create_session(request.session_id.clone());
            if let Some(envs) = request.env_vars {
                session.env_vars.extend(envs);
            }
            session.clone()
        };
        
        // 4. Run Task
        let mut session = session_clone;
        let strategy = request
            .strategy
            .clone()
            .unwrap_or_else(|| self.default_strategy());
        let run_result = runner.run(&request.prompt, &mut session, strategy).await;

        let (output, usage) = match run_result {
            Ok(result) => result,
            Err(err) => {
                match &err {
                    ZeneError::Cancelled(_) => self.mark_run_cancelled(&run_id).await,
                    _ => self.mark_run_failed(&run_id, &err).await,
                }
                return Err(err);
            }
        };

        // 5. Save Session
        {
            let session_manager = self.session_manager.lock().await;
            if let Err(e) = session_manager.save_session(&session).await {
                error!("ZeneEngine: Failed to save session {}: {}", request.session_id, e);
            }
        }

        self.mark_run_completed(&run_id, output.clone()).await;

        Ok(RunResult {
            run_id,
            output,
            session_id: request.session_id,
            usage,
        })
    }

    async fn register_run(&self, run_id: String, session_id: String, cancellation: CancellationToken) {
        let mut runs = self.runs.lock().await;
        runs.insert(
            run_id.clone(),
            RunEntry {
                snapshot: RunSnapshot::new(run_id, session_id),
                cancellation,
            },
        );
    }

    async fn update_run_status(&self, run_id: &str, status: RunStatus) {
        let mut runs = self.runs.lock().await;
        if let Some(entry) = runs.get_mut(run_id) {
            entry.snapshot.mark_status(status);
        }
    }

    async fn mark_run_completed(&self, run_id: &str, output: String) {
        let mut runs = self.runs.lock().await;
        if let Some(entry) = runs.get_mut(run_id) {
            entry.snapshot.mark_completed(output);
        }
    }

    async fn mark_run_failed(&self, run_id: &str, err: &ZeneError) {
        let mut runs = self.runs.lock().await;
        if let Some(entry) = runs.get_mut(run_id) {
            entry
                .snapshot
                .mark_failed(err.to_string(), Some(Self::classify_error_code(err)));
        }
    }

    async fn mark_run_cancelled(&self, run_id: &str) {
        let mut runs = self.runs.lock().await;
        if let Some(entry) = runs.get_mut(run_id) {
            entry.snapshot.mark_cancelled();
        }
    }

    fn default_strategy(&self) -> ExecutionStrategy {
        if self.config.simple_mode {
            ExecutionStrategy::Direct
        } else {
            ExecutionStrategy::Planned
        }
    }
}
