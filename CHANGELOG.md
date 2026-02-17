# Changelog

All notable changes to this project will be documented in this file.

## [0.4.0] - 2026-02-17

### Added
- **High-Level API**: Introduced `ZeneEngine` facade, providing a simplified and stable entrance for library users.
- **Session Abstraction**: Defined `SessionStore` trait and implemented `FileSessionStore` and `InMemorySessionStore` for flexible persistence.
- **Observability**: Added structured `AgentEvent` streaming, allowing real-time monitoring of planning, tool calls, and reflection.
- **Dependency Injection**: Refactored core components (`Orchestrator`, `Executor`, `ToolManager`) to support dependency injection, eliminating global static state.

### Changed
- **CLI Refactor**: Completely decoupled CLI/JSON-RPC logic from the core library; `main.rs` now acts as a thin wrapper around `ZeneEngine`.
- **Async Sessions**: Transitioned `SessionManager` to an asynchronous interface powered by `async-trait`.

## [0.3.1] - 2026-02-17

### Added
- **Testing Infrastructure**: Added `MockUserInterface` and `MockAgentClient` to `src/testing/mod.rs` for robust unit and integration testing.
- **Integration Tests**: Added `tests/it_agent_flow.rs` ensuring end-to-end verification of the agent loop (Planning -> Execution -> Reflection).
- **Unit Tests**: comprehensive unit tests for `Executor` and `Orchestrator` core components.

### Changed
- **Refactor**: Significant refactoring of `src/main.rs` to properly utilize the `zene` library crate, improving modularity and testability.
- **Agent Architecture**: Stabilized the P3 architecture (Orchestrator pattern) with reinforced testing.
- **Mocking**: Improved mock response handling in `AgentClient` to support JSON-structured responses matching real LLM outputs.

## [0.3.0] - 2026-02-16

### Added
- **Agent Architecture (P3)**: Decoupled monolithic runner into `Orchestrator`, `Planner`, `Executor`, and `Reflector`.
- **Context Memory**: Added tiered memory system with vector search (`fastembed`, `usearch`) and session compaction.
- **MCP Support**: Integrated Model Context Protocol via `zene-mcp`.
- **Python Execution V3**: Enhanced sandboxing and environment management for Python tasks.

### Changed
- **Documentation**: Overhauled documentation structure and added comprehensive guides.
