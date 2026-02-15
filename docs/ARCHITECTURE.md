# Zene Architecture

## 1. Core Philosophy
- **Headless Core**: The core logic is a standalone process, decoupled from any UI.
- **Protocol First**: Communicates via standard protocols (JSON-RPC) over Stdio/HTTP.
- **High Performance**: Rust-based, async I/O, resident daemon for instant context.
- **Model Agnostic**: Support OpenAI, Anthropic, etc. via standard traits.

## 2. Tech Stack (Revised)
- **Runtime**: `tokio` (Async I/O, Task Scheduling)
- **Communication**: `serde_json` (JSON-RPC over Stdio)
- **LLM Client**: `llm-connector` (Unified interface for OpenAI, Anthropic, DeepSeek, etc.)
- **Context**: `tree-sitter` (Code Analysis), `ignore` (File Walking)
- **CLI Args**: `clap` (For launching the daemon or one-shot commands)
- **Logging**: `tracing` (Structured logging to file/stderr)

## 3. Module Structure
```rust
src/
├── main.rs          // Entry point (CLI parsing -> dispatch)
├── server.rs        // JSON-RPC Server Loop (Stdio/Socket)
├── api.rs           // Request/Response Definitions
├── engine/          // Core Logic
│   ├── mod.rs
│   ├── context.rs   // Codebase Indexing & Retrieval (Tree-sitter)
│   ├── tools.rs     // File I/O, Git, Command Execution
│   └── planner.rs   // High-level Task Planning
└── agent/           // LLM Interaction
    ├── mod.rs
    ├── client.rs    // LLM Client Wrapper
    └── prompt.rs    // Dynamic Prompt Engineering
```

## 4. Operational Modes
Zene operates in two primary modes:

### A. One-Shot CLI (Direct Execution)
For quick tasks in the terminal.
```bash
$ zene "Refactor main.rs to use async/await"
# Spawns a transient instance, executes the prompt, applies changes (or diffs), then exits.
```

### B. Daemon / Server Mode (Interactive)
For IDEs or complex workflows.
```bash
$ zene server --stdio
# Starts the long-running process.
# Client sends: {"jsonrpc": "2.0", "method": "agent.run", "params": {...}}
# Server responds: {"jsonrpc": "2.0", "result": {...}}
```

## 5. Interfaces (The "Face" of Zene)
Since Zene is headless, it exposes its capabilities through:
1.  **CLI**: Human-friendly wrapper around the API.
2.  **JSON-RPC Server**: For IDEs and other tools to integrate.

## 6. Data Flow (Request/Response)
1.  **Input**: Client sends a Task (Instruction + Context).
2.  **Analysis**: Engine analyzes codebase (Tree-sitter) to gather relevant context.
3.  **Planning**: Agent formulates a plan (Chain of Thought).
4.  **Execution**: Agent invokes Tools (Edit File, Run Test).
5.  **Output**: Returns result (Diff, Message, or Error) to Client.
