# Zene Core Optimization Plan

Based on feedback from the Celadon team, Zene is undergoing a major architectural refactor to become a cloud-native, high-performance AI coding engine suitable for embedding in async web frameworks (like Axum).

## 1. Async Native (Non-blocking IO)
**Goal**: Eliminate all blocking operations to prevent thread starvation in async runtimes (Tokio).

- [x] **Replace `std::fs` with `tokio::fs`**:
    - `src/engine/tools.rs`: `read_file`, `write_file`, `apply_patch` converted to async.
    - `src/engine/session/store.rs`: `load`, `save`, `load_all` converted to async using `tokio::fs`.
- [x] **Replace `std::process` with `tokio::process`**:
    - `src/engine/tools.rs`: `run_command` validated/updated to use `tokio::process::Command`.
- [x] **Mutex Migration**:
    - Replaced `std::sync::Mutex` with `tokio::sync::Mutex` in `InMemorySessionStore` and `ToolManager` calls.

## 2. Streaming Event Architecture
**Goal**: Provide real-time feedback (Typewriter effect) for DevPage.

- [x] **Enhanced `AgentEvent` Enum**:
    - Added `ThoughtDelta`, `ToolOutputDelta`, `FileStateChanged`.
- [x] **Streaming Interface**:
    - Implemented `ZeneEngine::run_stream` returning `mpsc::UnboundedReceiver<AgentEvent>`.
- [x] **Event Emission**:
    - Wired up `Executor` to emit `ThoughtDelta` using `llm-connector` v0.6.1 streaming capabilities.
    - Wired up `ToolManager` to emit `ToolOutputDelta` (via `ToolResult` event).

## 3. Structured File State (Workspace Awareness)
**Goal**: Enable "IDE-like" file tree updates in the frontend.

- [x] **File Change Tracking**:
    - Defined `FileChange` struct.
- [ ] **Tool Instrumentation**:
    - Modify `write_file` and `apply_patch` in `ToolManager` to broadcast `FileStateChanged`.

## Execution Strategy

1.  **Phase 1 (Completed)**: Refactored `tools.rs` and `session/store.rs` to be fully async.
2.  **Phase 2 (Completed)**: Implemented `run_stream` and event definitions. Integrated `llm-connector` streaming to support real-time `ThoughtDelta` emission.
3.  **Phase 3**: Implement file state tracking in `ToolManager`.
