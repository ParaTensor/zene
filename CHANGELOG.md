# Changelog

All notable changes to this project will be documented in this file.

## [0.6.1] - 2026-03-01

### Changed
- **Dependency Upgrade**: Upgraded `tree-sitter` to v0.24, `tree-sitter-rust` to v0.23, and `tree-sitter-typescript` to v0.23 to resolve version conflicts with downstream projects.
- **API Alignment**: Updated `ContextEngine` to adapt to new `tree-sitter` API.

### Fixed
- **Configuration**: Fixed missing `region` field initialization in `RoleConfig` tests.

## [0.6.0] - 2026-03-01

### Added
- **Library Integration**: Exposed `ZeneEngine`, `EventEnvelope`, and `SessionStore` in `lib.rs` to support embedding Zene as a library.
- **Event Streaming**: Added `run_envelope_stream` to `ZeneEngine` for consuming structured agent events.

### Removed
- **JSON-RPC Server**: Removed `jsonrpsee` based server implementation to simplify the core library.

## [0.5.5] - 2026-02-22

### Added
- **LLM Providers Integration**: Integrated `llm_providers` crate to support a wide range of LLM providers (e.g., Zhipu, MiniMax, DeepSeek, etc.) via dynamic configuration. `AgentClient::new` now queries `llm_providers` for base URLs before falling back to hardcoded defaults.

## [0.5.4] - 2026-02-22

### Changed
- **Tool Descriptions**: Enhanced tool descriptions for `read_file` and `search_code` with "CRITICAL" warnings to force LLMs (especially DeepSeek) to provide required parameters (`path`, `pattern`).

## [0.5.3] - 2026-02-22

### Fixed
- **DeepSeek Streaming Bug**: Fixed a critical bug where streaming tool calls from certain providers (like DeepSeek) caused the tool name to be duplicated endlessly (e.g., `list_fileslist_files...`). The executor now intelligently handles tool name chunks, preventing redundant appending.

## [0.5.2] - 2026-02-22

### Fixed
- **Critical Async Panic Fix**: Replaced all remaining `tokio::fs` calls in `ToolManager` (`read_file`, `write_file`, `apply_patch`) with `tokio::task::spawn_blocking`. This eliminates the possibility of blocking the Tokio runtime worker threads during heavy file I/O, solving the "Cannot block the current thread" panic once and for all.
- **Context Tools**: Updated `search_code` and `list_files` in `ToolManager` to also use `spawn_blocking`, ensuring consistent non-blocking behavior across all tools.

## [0.5.1] - 2026-02-22

### Changed
- **Async Heavy Context**: Offloaded CPU-intensive context tools (`search_code`, `list_files`) to `tokio::task::spawn_blocking`, ensuring the main event loop remains responsive even during large codebase scans.
- **Dependency Upgrade**: Upgraded `reqwest` to 0.12 and removed `blocking` feature, fully eliminating synchronous HTTP client code from the dependency tree.

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
