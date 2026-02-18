# Testing & Architecture

Zene prioritizes reliability and maintainability. This section covers the testing infrastructure and architectural verification patterns.

## Mocking System

To test agent logic without incurring LLM costs or network latency, we implemented a robust mocking system:

- **MockAgentClient**: Simulates LLM responses. It can be pre-loaded with a sequence of `ChatResponse` objects to test specific conversation flows (e.g., Tool Calls -> Output -> Reflection).
- **MockUserInterface**: Intercepts UI calls (like confirmation prompts) to allow headless testing of interactive features.

## Test Levels

### Unit Tests
Covered individual components like `Executor` and `Orchestrator`.
- **Location**: `src/agent/executor.rs`, `src/agent/orchestrator.rs`
- **Scope**: fast, isolated logic verification.

### Integration Tests
Verifies the complete agent loop.
- **Location**: `tests/it_agent_flow.rs`
- **Scope**: Simulates a full "Plan -> Execute -> Reflect" cycle using mocks.

## Running Tests

Developers can run the full suite using standard Cargo commands:

```bash
cargo test
```

To run specifically the integration tests:

```bash
cargo test --test it_agent_flow
```
