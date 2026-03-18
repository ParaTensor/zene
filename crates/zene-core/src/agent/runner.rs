use crate::engine::error::Result;
use std::sync::Arc;

use crate::agent::client::AgentClient;
use crate::agent::orchestrator::Orchestrator;
use crate::engine::context::ContextEngine;
use crate::engine::session::Session;
use crate::engine::ui::UserInterface;
use crate::engine::tools::ToolManager;
use crate::config::AgentConfig;
use crate::engine::contracts::{AgentEvent, TokenUsage};
use crate::engine::runtime::CancellationToken;
use crate::engine::strategy::ExecutionStrategy;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Mutex;

pub struct AgentRunner {
    orchestrator: Orchestrator,
}

impl AgentRunner {
    pub fn new(
        config: AgentConfig, 
        tool_manager: Arc<ToolManager>, 
        context_engine: Arc<Mutex<ContextEngine>>,
        user_interface: Box<dyn UserInterface>
    ) -> Result<Self> {
        let planner_client = AgentClient::new(&config.planner)?;
        let executor_client = AgentClient::new(&config.executor)?;
        let reflector_client = AgentClient::new(&config.reflector)?;
        
        let orchestrator = Orchestrator::new(
            config,
            planner_client,
            executor_client,
            reflector_client,
            tool_manager,
            context_engine,
            user_interface,
        );

        Ok(Self { orchestrator })
    }

    pub async fn run(&mut self, task: &str, session: &mut Session, strategy: ExecutionStrategy) -> Result<(String, TokenUsage)> {
        self.orchestrator.run(task, session, strategy).await
    }

    pub fn new_with_orchestrator(orchestrator: Orchestrator) -> Self {
        Self { orchestrator }
    }

    pub fn with_event_sender(mut self, sender: UnboundedSender<AgentEvent>) -> Self {
        self.orchestrator = self.orchestrator.with_event_sender(sender);
        self
    }

    pub fn with_cancellation_token(mut self, token: CancellationToken) -> Self {
        self.orchestrator = self.orchestrator.with_cancellation_token(token);
        self
    }
}






