use crate::engine::context::ContextEngine;
use crate::engine::error::Result;
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
}

impl ZeneEngine {
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
        })
    }

    pub async fn run(&self, request: RunRequest) -> Result<RunResult> {
        self.run_with_events(request, None).await
    }

    pub async fn run_stream(&self, request: RunRequest) -> Result<mpsc::UnboundedReceiver<AgentEvent>> {
        let (tx, rx) = mpsc::unbounded_channel::<AgentEvent>();
        let engine = self.clone();
        
        tokio::spawn(async move {
            match engine.run_with_events(request, Some(tx.clone())).await {
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

        Ok(rx)
    }

    pub async fn run_envelope_stream(&self, request: RunRequest) -> Result<mpsc::UnboundedReceiver<EventEnvelope>> {
        let (tx, mut rx) = mpsc::unbounded_channel::<AgentEvent>();
        let (out_tx, out_rx) = mpsc::unbounded_channel::<EventEnvelope>();
        
        let engine = self.clone();
        let session_id = request.session_id.clone();
        let run_id = uuid::Uuid::new_v4().to_string();
        
        // Spawn the runner
        tokio::spawn(async move {
            match engine.run_with_events(request, Some(tx.clone())).await {
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
        let run_id_clone = run_id.clone();
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

    pub async fn run_with_events(
        &self, 
        request: RunRequest, 
        event_sender: Option<mpsc::UnboundedSender<AgentEvent>>
    ) -> Result<RunResult> {
        // 1. Prepare UI (Auto for library usage)
        let ui = Box::new(AutoUserInterface);

        // 2. Setup Runner
        let mut runner = AgentRunner::new(
            self.config.clone(), 
            self.tool_manager.clone(), 
            self.context_engine.clone(),
            ui
        )?;
        if let Some(sender) = event_sender {
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
        let (output, usage) = runner.run(&request.prompt, &mut session).await?;

        // 5. Save Session
        {
            let session_manager = self.session_manager.lock().await;
            if let Err(e) = session_manager.save_session(&session).await {
                error!("ZeneEngine: Failed to save session {}: {}", request.session_id, e);
            }
        }

        Ok(RunResult {
            output,
            session_id: request.session_id,
            usage,
        })
    }
}
