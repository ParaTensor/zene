# Analysis: Spawn-per-Request vs. Long-running Daemon

This analysis compares the "Spawn-per-Request" (CGI Model) with the "Long-running Daemon" model for managing Zene's lifecycle in a web context.

## 1. How it works
1. **User Request**: Web UI sends task to Backend.
2. **Spawn**: Backend runs `zene run "..." --session <id>`.
3. **Pipe**: Backend pipes Zene's `stdout` to the Web UI.
4. **Exit**: Zene completes task and exits.

## 2. Advantages (The "Simple" Case)
- **Extreme Isolation**: Fresh OS process per task.
- **Lower Complexity**: No multi-threading or complex state management in the engine.
- **Stateless Backend**: Simple management.

## 3. The Bottlenecks (The "Heavy" Reality)

| Task | Time Cost | Impact |
| :--- | :--- | :--- |
| **Model Load** | 100ms - 500ms | Loading local embedding models (`fastembed`). |
| **MCP Connect**| 200ms - 1s | Handshaking with sub-processes. |
| **Context Index**| 500ms - 2s | Parsing `tree-sitter` symbols. |

**Result**: A "latency tax" of ~2 seconds before the Agent starts.

## 4. Compromise: The "Warm Worker" Pool
Instead of "Spawn-on-Demand," the outer app can maintain a **Pool of Zene Daemons**:
- Backend keeps 5 Zene instances running in `server` mode.
- Backend assigns idle Zene instance to session.
- Once done, backend sends a `session.clear` command.

## 5. Security Comparison

| Feature | Spawn-per-Request | Long-running Server |
| :--- | :--- | :--- |
| **Zombie Risk** | Low | High |
| **Secret Safety**| High | Low |
| **Resource Cap** | Easy | Hard |

## Conclusion
If your goal is **simplicity and safety**, the "Spawn-per-Request" model is the winner. If you need **instant response**, Zene needs to stay "Warm."
