# Changelog

All notable changes to this project will be documented in this file.

## [0.5.0] - 2026-02-22

### Added
- **Workspace State Diff**: Added `AgentEvent::FileStateChanged` (Created/Modified) to the event stream, enabling IDE-like real-time file tree updates in frontend integrations (like Celadon).
- **Architecture Stabilization**: Fully validated and stabilized the Async Native and Streaming Event architecture introduced in 0.4.9.

## [0.4.9] - 2026-02-22

### Added
- **Async Native Architecture**: Completely refactored the engine core to be 100% async/non-blocking. Replaced all usages of `std::fs` and `std::process` with `tokio::fs` and `tokio::process`, making Zene suitable for embedding in high-concurrency async web frameworks (like Axum).
- **Streaming Events**: Added full support for real-time feedback. The `Executor` now emits `ThoughtDelta` events via a streaming interface, enabling "typewriter" effects in UIs.
- **Release Workflow**: Refactored GitHub Actions to separate build and release jobs, resolving race conditions during multi-platform artifact uploads.

### Changed
- **LLM Integration**: Upgraded to `llm-connector` 0.6.1, leveraging its new streaming capabilities.
- **Documentation**: Updated all links to point to the new [zene.run](https://zene.run) domain.
- **Visuals**: Updated README and homepage with new, clearer SVG terminal demos.

## [0.4.3] - 2026-02-19

### Added
- **Optional Semantic Memory**: Semantic memory (RAG) is now opt-in via `ZENE_USE_SEMANTIC_MEMORY=true`, saving ~200MB of RAM by default.

### Changed
- **Shared Context Engine**: `ContextEngine` is now initialized once and shared across all components via `Arc<Mutex<ContextEngine>>`, preventing redundant model loading.
- **Documentation Consolidation**: All documentation has been unified into the `www/` directory and is served via the VitePress site at [zene.run](https://zene.run). The redundant `docs/` folder has been removed.

## [0.4.2] - 2026-02-18

### Changed
- **Documentation Overhaul**: Consolidated 10 fragmented documentation files into 5 high-quality, focused guides (`ARCHITECTURE.md`, `CONTEXT.md`, `EXTENSIONS.md`, `PYTHON.md`, `ROADMAP.md`).
- **Cleanup**: Removed redundant architecture guides, outdated roadmaps, and fragmented strategy documents.

## [0.4.1] - 2026-02-18

### Added
- **Parallel Tool Execution**: Enabled concurrent execution of multiple independent tool calls in the `Executor`, significantly lowering task latency.
- **Typed Error Hierarchy**: Migrated from `anyhow` to a structured `ZeneError` system using `thiserror` for better programmatic error handling.
- **Advanced Observability (xtrace v0.0.13)**:
    - Integrated `XtraceLayer` for automated span timing and metric capture.
    - **Trace ID Propagation**: Automatic injection of Trace IDs into outbound HTTP requests (`X-Trace-Id`) and shell commands (`ZENE_TRACE_ID`) for end-to-end distributed tracing.
- **Token Usage Aggregation**: Precise tracking and propagation of token usage across all LLM interactions in a session.

### Fixed
- **History Consistency**: Refactored `execute_task` to use mutable history references, ensuring all intermediate tool results are correctly persistent in the session.
- **Test Stability**: Updated unit and integration tests to align with new async signatures and tuple return types.

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
