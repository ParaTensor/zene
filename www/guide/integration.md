# How to Call Zene

Zene is designed to be highly flexible, supporting integration via CLI, standard protocols, or direct library embedding.

## 1. Command Line Interface (CLI)
The fastest way to use Zene for one-off tasks.

- **Command**: `zene run "<instruction>"`
- **Options**:
  - `--session <id>`: Continue a previous session or start a named one.
  - `-v`: Enable verbose logging.
- **Workflow**: Ideal for shell scripts, aliases, or simple terminal-based development.

## 2. JSON-RPC (Stdio Mode)
The standard way to integrate Zene into IDEs (VS Code, Cursor) or other applications.

- **Start Channel**: `zene server`
- **Protocol**: JSON-RPC 2.0 over `stdin`/`stdout`.
- **Methods**:
  - `agent.run`: Execute a task.
  - `session.get`: Retrieve session history.
  - `session.list`: List active sessions.
- **Why use it?**: It provides a stable, language-agnostic interface with support for real-time event streaming (`AgentEvent`).

## 3. Rust Library API
Directly embed the Zene engine into your Rust projects for maximum performance and control.

- **Crate**: `zene`
- **Key Component**: `ZeneEngine`
- **Example**:
  ```rust
  use zene::ZeneEngine;
  use zene::agent::engine::RunRequest;

  let engine = ZeneEngine::new(config).await?;
  let mut stream = engine.run(RunRequest {
      instruction: "Analyze the current project".into(),
      session_id: Some("analysis-01".into()),
  }).await?;
  ```
- **Why use it?**: Lowest latency and zero overhead. Best for high-performance Rust tools.

## Summary: Which one to choose?

| Pattern | Best For | Complexity | Isolation |
| :--- | :--- | :--- | :--- |
| **CLI** | Quick scripts | Low | Process-level |
| **JSON-RPC** | IDEs, Apps | Medium | Protocol-level |
| **Rust Lib** | Native Tools | High | Internal |
