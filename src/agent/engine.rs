use anyhow::Result;
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

pub struct ZeneEngine {
    config: AgentConfig,
    pub tool_manager: Arc<ToolManager>,
    session_manager: Arc<Mutex<SessionManager>>,
}

impl ZeneEngine {
    pub async fn new(config: AgentConfig, session_store: Arc<dyn SessionStore>) -> Result<Self> {
        let mcp_manager = Arc::new(McpManager::new(config.mcp.clone()));
        let tool_manager = Arc::new(ToolManager::new(Some(mcp_manager)));
        let session_manager = Arc::new(Mutex::new(SessionManager::new(session_store).await?));

        Ok(Self {
            config,
            tool_manager,
            session_manager,
        })
    }

    pub async fn run(&self, request: RunRequest) -> Result<RunResult> {
        self.run_with_events(request, None).await
    }

    pub async fn run_with_events(
        &self, 
        request: RunRequest, 
        event_sender: Option<mpsc::UnboundedSender<AgentEvent>>
    ) -> Result<RunResult> {
        // 1. Prepare UI (Auto for library usage)
        let ui = Box::new(AutoUserInterface);

        // 2. Setup Runner
        let mut runner = AgentRunner::new(self.config.clone(), self.tool_manager.clone(), ui)?;
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
        let output = runner.run(&request.prompt, &mut session).await?;

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
            usage: TokenUsage::default(), // TODO: Track actual usage
        })
    }
}
