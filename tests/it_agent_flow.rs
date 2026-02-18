use std::sync::Arc;
use zene::agent::runner::AgentRunner;
use zene::agent::orchestrator::Orchestrator;
use zene::agent::client::AgentClient;
use zene::config::AgentConfig;
use zene::engine::context::ContextEngine;
use zene::engine::session::SessionManager;
use zene::engine::session::store::InMemorySessionStore;
use zene::engine::tools::ToolManager;
use zene::testing::MockUserInterface;
use llm_connector::types::ChatResponse;

#[tokio::test]
async fn test_agent_integration_flow() {
    // 1. Setup Environment
    let ui = Box::new(MockUserInterface::new());
    let context_engine = ContextEngine::new().unwrap();
    let mut session_manager = SessionManager::new(Arc::new(InMemorySessionStore::new())).await.unwrap();
    let mut session = session_manager.create_session("it_test_user".to_string());
    let tool_manager = Arc::new(ToolManager::new(None));

    // 2. Setup Mocks
    let planner_json = serde_json::json!({
        "id": "plan-1",
        "object": "chat.completion",
        "created": 1,
        "model": "mock",
        "content": "{ \"tasks\": [\"Task A\"] }",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": [{"type": "text", "text": "{ \"tasks\": [\"Task A\"] }"}]
            },
            "finish_reason": "stop"
        }]
    });
    let planner_resp: ChatResponse = serde_json::from_value(planner_json).unwrap();
    let planner_client = AgentClient::mock(vec![planner_resp]);

    let executor_json = serde_json::json!({
        "id": "exec-1",
        "object": "chat.completion",
        "created": 2,
        "model": "mock",
        "content": "Execution Result A",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": [{"type": "text", "text": "Execution Result A"}]
            },
            "finish_reason": "stop"
        }]
    });
    let executor_resp: ChatResponse = serde_json::from_value(executor_json).unwrap();
    let executor_client = AgentClient::mock(vec![executor_resp]);

    let reflector_json = serde_json::json!({
        "id": "ref-1",
        "object": "chat.completion",
        "created": 3,
        "model": "mock",
        "content": "{ \"passed\": true, \"reason\": \"OK\", \"suggestions\": null }",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": [{"type": "text", "text": "{ \"passed\": true, \"reason\": \"OK\", \"suggestions\": null }"}]
            },
            "finish_reason": "stop"
        }]
    });
    let reflector_resp: ChatResponse = serde_json::from_value(reflector_json).unwrap();
    let reflector_client = AgentClient::mock(vec![reflector_resp]);

    let config = AgentConfig {
         planner: zene::config::RoleConfig { provider: "mock".to_string(), model: "mock".to_string(), api_key: "mock".to_string(), base_url: None },
         executor: zene::config::RoleConfig { provider: "mock".to_string(), model: "mock".to_string(), api_key: "mock".to_string(), base_url: None },
         reflector: zene::config::RoleConfig { provider: "mock".to_string(), model: "mock".to_string(), api_key: "mock".to_string(), base_url: None },
         mcp: zene::config::mcp::McpConfig::default(),
         simple_mode: false,
         xtrace_endpoint: None,
         xtrace_token: None,
    };

    // 3. Initialize Agent
    let orchestrator = Orchestrator::new(
        config.clone(),
        planner_client,
        executor_client,
        reflector_client,
        tool_manager,
        context_engine,
        ui,
    );
    
    let mut runner = AgentRunner::new_with_orchestrator(orchestrator);

    // 4. Run Task
    let (result, _usage) = runner.run("Integration Test Task", &mut session).await.unwrap();

    // 5. Verify
    assert!(result.contains("Task 1: Completed"));
    assert!(session.plan.is_some());
    assert_eq!(session.plan.as_ref().unwrap().tasks.len(), 1);
}
