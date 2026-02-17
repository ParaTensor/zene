use anyhow::Result;


use crate::agent::client::AgentClient;
use crate::agent::orchestrator::Orchestrator;
use crate::engine::context::ContextEngine;
use crate::engine::session::Session;
use crate::engine::ui::UserInterface;
use crate::config::AgentConfig;

pub struct AgentRunner {
    orchestrator: Orchestrator,
}

impl AgentRunner {
    pub fn new(config: AgentConfig, user_interface: Box<dyn UserInterface>) -> Result<Self> {
        let planner_client = AgentClient::new(&config.planner)?;
        let executor_client = AgentClient::new(&config.executor)?;
        let reflector_client = AgentClient::new(&config.reflector)?;
        
        let context_engine = ContextEngine::new()?;
        
        let orchestrator = Orchestrator::new(
            config,
            planner_client,
            executor_client,
            reflector_client,
            context_engine,
            user_interface,
        );

        Ok(Self { orchestrator })
    }

    pub async fn run(&mut self, task: &str, session: &mut Session) -> Result<String> {
        self.orchestrator.run(task, session).await
    }

    /// Create a runner with a pre-configured orchestrator (useful for testing)
    pub fn new_with_orchestrator(orchestrator: Orchestrator) -> Self {
        Self { orchestrator }
    }
}






