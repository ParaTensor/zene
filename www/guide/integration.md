# How to Call Zene

Zene is designed to be embedded into other developer experiences. Today, the runtime is available through the CLI, a spawnable worker process, the Rust library API, and Python bindings.

## 1. Command Line Interface (CLI)
The fastest way to use Zene for one-off tasks.

- **Command**: `zene run "<instruction>"`
- **Workflow**: Ideal for shell scripts, aliases, or lightweight local automation.

## 2. Worker Process
Use worker mode when you want process isolation but do not want to embed Rust directly.

- **Command**: `zene worker`
- **Input**: one JSON `RunRequest` on `stdin`
- **Output**: newline-delimited JSON `WorkerMessage` values on `stdout`
- **Why use it?**: Best first step for host applications that want a spawn-per-request execution model.

See [Worker Protocol](/guide/worker) for the full contract.

## 3. Rust Library API
Directly embed Zene into your Rust projects for maximum control over sessions, events, and runtime composition.

- **Crate**: `zene`
- **Key Component**: `ZeneEngine`
- **Example**:
  ```rust
  use zene::{ExecutionStrategy, RunRequest, ZeneEngine};

  let engine = ZeneEngine::new(config, session_store).await?;
  let result = engine.run(RunRequest {
      prompt: "Analyze the current project".into(),
      session_id: "analysis-01".into(),
      env_vars: None,
      strategy: Some(ExecutionStrategy::Planned),
  }).await?;
  ```
- **Why use it?**: Best when Zene is part of a larger Rust-native tool, IDE backend, or automation system.

### Host Control Surface

The embedded runtime now exposes a minimal host-facing control model:

- `run`: execute and await a final `RunResult`
- `run_stream`: receive incremental `AgentEvent` values
- `submit`: receive a `RunHandle` for tracked execution
- `get_run_snapshot`: inspect the latest known run state
- `cancel_run`: request cooperative cancellation

## 4. Event Streaming
Hosts that need incremental visibility can consume the event stream rather than waiting for a single final result.

- **API**: `run_stream` and `run_envelope_stream`
- **Use Cases**:
  - render live progress in a UI
  - persist structured run events
  - build dashboards, logs, or supervisory control loops

When using `submit`, the host can stream events while also tracking the run by `run_id`.

## 5. Python Bindings
Python bindings are available for hosts that want to integrate Zene without embedding Rust directly.

- **Location**: `python/zene`
- **Use Case**: internal tools, automation services, and experimentation from Python code

## Summary: Which one to choose?

| Pattern | Best For | Control | Notes |
| :--- | :--- | :--- | :--- |
| **CLI** | Quick scripts | Low | Fastest way to run one task |
| **Worker** | Spawn-per-request hosts | Medium | Good isolation with a tiny protocol |
| **Rust Lib** | Native tools and services | High | Primary embeddable surface |
| **Event Stream** | UIs and observability | High | Best for live control and monitoring |
| **Python Bindings** | Python-based tooling | Medium | Useful when Rust embedding is not desired |
