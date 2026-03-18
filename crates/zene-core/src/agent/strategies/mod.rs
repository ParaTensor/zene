use async_trait::async_trait;

use crate::agent::orchestrator::Orchestrator;
use crate::engine::error::Result;
use crate::engine::session::Session;
use crate::engine::strategy::ExecutionStrategy;
use crate::engine::contracts::TokenUsage;

pub struct PlannedStrategy;
pub struct DirectStrategy;

#[async_trait]
pub trait AgentStrategy: Send + Sync {
    async fn run(
        &self,
        orchestrator: &mut Orchestrator,
        goal: &str,
        session: &mut Session,
    ) -> Result<(String, TokenUsage)>;
}

#[async_trait]
impl AgentStrategy for PlannedStrategy {
    async fn run(
        &self,
        orchestrator: &mut Orchestrator,
        goal: &str,
        session: &mut Session,
    ) -> Result<(String, TokenUsage)> {
        orchestrator.ensure_plan(goal, session).await?;
        orchestrator.execute_planned(session).await
    }
}

#[async_trait]
impl AgentStrategy for DirectStrategy {
    async fn run(
        &self,
        orchestrator: &mut Orchestrator,
        goal: &str,
        session: &mut Session,
    ) -> Result<(String, TokenUsage)> {
        orchestrator.execute_direct(goal, session).await
    }
}

pub fn resolve_strategy(strategy: ExecutionStrategy) -> Box<dyn AgentStrategy> {
    match strategy {
        ExecutionStrategy::Planned => Box::new(PlannedStrategy),
        ExecutionStrategy::Direct => Box::new(DirectStrategy),
    }
}
