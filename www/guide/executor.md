# The Executor

The **Executor** is the hands-on engineer of the Zene loop. While the Planner decides *what* to do, the Executor decides *how* to do it and performs the actual work.

## Responsibilities

- **Tool Use**: Utilizes Zene's toolset (File I/O, Shell Commands, Grep, MCP tools) to modify the codebase.
- **Implementation**: Translates high-level tasks into specific code changes.
- **Safety**: Operates within the safety constraints defined by the environment (e.g., sandboxed execution, confirmation prompts).

## Capabilities

- **Command Execution**: Runs build commands, tests, and other shell utilities.
- **File Manipulation**: Reads, writes, and patches files.
- **MCP Integration**: Connects to external tools via the Model Context Protocol to expand its capabilities.
