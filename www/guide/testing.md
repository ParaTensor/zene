# Testing Infrastructure
2: 
3: Zene prioritizes reliability and maintainability. Version 0.3.1 introduced a comprehensive testing infrastructure to ensure stability across releases.
4: 
5: ## Mocking System
6: 
7: To test agent logic without incurring LLM costs or network latency, we implemented a robust mocking system:
8: 
9: - **MockAgentClient**: Simulates LLM responses. It can be pre-loaded with a sequence of `ChatResponse` objects to test specific conversation flows (e.g., Tool Calls -> Output -> Reflection).
10: - **MockUserInterface**: Intercepts UI calls (like confirmation prompts) to allow headless testing of interactive features.
11: 
12: ## Test Levels
13: 
14: ### Unit Tests
15: Covered individual components like `Executor` and `Orchestrator`.
16: - **Location**: `src/agent/executor.rs`, `src/agent/orchestrator.rs`
17: - **Scope**: fast, isolated logic verification.
18: 
19: ### Integration Tests
20: Verifies the complete agent loop.
21: - **Location**: `tests/it_agent_flow.rs`
22: - **Scope**: Simulates a full "Plan -> Execute -> Reflect" cycle using mocks.
23: 
24: ## Running Tests
25: 
26: Developers can run the full suite using standard Cargo commands:
27: 
28: ```bash
29: cargo test
30: ```
31: 
32: To run specifically the integration tests:
33: 
34: ```bash
35: cargo test --test it_agent_flow
36: ```
