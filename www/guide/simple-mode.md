# Simple Mode (Direct Execution)

Sometimes you don't need a complex plan. You just want Zene to execute a command or make a small change directly. **Simple Mode** bypasses the Planner and Reflector loops, giving you direct access to the Executor.

## When to use it?

- **Quick Fixes**: "Fix the typo in README.md"
- **Refactoring**: "Rename variable x to y in execution.rs"
- **Information**: "List all functions in main.rs"
- **MCP Tools**: "Use git to commit changes"

## How to Enable

Set the `ZENE_SIMPLE_MODE` environment variable to `true` or `1`.

```bash
export ZENE_SIMPLE_MODE=true
zene run "List files in src/"
```

## How it Works

In Simple Mode:
1.  **Skip Planning**: Zene immediately acts on your prompt.
2.  **Direct Execution**: The Executor uses available tools to satisfy the request.
3.  **No Reflection**: Zene will not self-correct or run verification loops automatically (unless you ask it to).

This mode is faster and uses fewer tokens, making it ideal for straightforward tasks.
