# Architecture Optimization: Session-Scoped Isolation & Environment Management

**Status**: Proposed / In-Progress
**Target**: Solves multi-user concurrency issues and Python dependency management.

## 1. The Problem: Global State is Unsafe

Currently, Zene executes shell commands using the parent process's environment. This presents two critical risks:

1.  **Concurrency Risk**: In a multi-threaded Rust server, using `std::env::set_var` is globally unsafe. If User A sets `API_KEY=123`, User B (executing concurrently) might accidentally use that key.
2.  **Environment Pollution**: Installing Python packages via `pip install` affects the host system globally, leading to dependency conflicts ("Dependency Hell") and potential system instability.

## 2. Solution: Session-Scoped Isolation

We propose shifting from **Process-Level State** to **Session-Level State**.

### A. Data Layer (The Session)
Environment variables will be stored inside the `Session` struct, which is unique to each user interaction and persisted to disk.

```rust
pub struct Session {
    // ... existing fields
    pub env_vars: HashMap<String, String>, // New: Isolated environment state
}
```

### B. Execution Layer (Transient Injection)
Tools like `run_command` will be refactored to accept this map. Variables are injected *only* into the spawned child process, leaving the main server process untouched.

```rust
// Pseudocode
Command::new("sh")
    .envs(&session.env_vars) // CRITICAL: Applies only to this child
    .spawn()
```

### C. Persistence
Since `Session` is serialized to JSON, environment configurations (e.g., API keys, paths) persist across server restarts, allowing for long-running, interruptible tasks.

## 3. Python Execution Strategy (Virtual Environments)

Building on the isolation layer, we will implement robust Python support.

### The `.venv` Standard
Zene will treat the current working directory as a Project.
1.  **Detection**: Check for `.venv` directory.
2.  **Creation**: If missing, auto-run `python -m venv .venv`.
3.  **Execution**: All Python scripts will be executed via `.venv/bin/python`, ensuring isolation from the host system.

### Dependency Intelligence
*   **Auto-Install**: Before execution, check `requirements.txt`.
*   **Smart Caching**: Hash `requirements.txt` to avoid redundant `pip install` runs, significantly speeding up iteration cycles.

## 4. Implementation Roadmap

1.  **Refactor Session**: Add `env_vars: HashMap<String, String>` to `src/engine/session.rs`.
2.  **Update Tool Interface**: Modify `ToolManager::run_command` to accept an optional environment map.
3.  **Agent Integration**: Update `AgentRunner` to pass `session.env_vars` when invoking tools.
4.  **New Tools**:
    *   `set_env(key, value)`: Safely updates the session state.
    *   `run_python(script)`: A high-level tool that manages the `.venv` lifecycle and executes code safely.

## 5. Security & Future Considerations

*   **Secret Masking**: Future versions should detect sensitive keys in `env_vars` and mask them in logs/UI.
*   **Inheritance Control**: We currently default to `Inherit + Override`. For stricter security, we may offer a `Clean` mode (using `Command::env_clear()`) to strip host variables entirely.
