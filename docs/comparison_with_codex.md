# Zene vs OpenAI Codex CLI

This document provides a detailed comparison of the architecture, workflow, and features of **Zene** (this project) and **OpenAI Codex CLI**, aiming to clarify Zene's positioning and unique advantages.

## 1. Core Philosophy

| Feature | OpenAI Codex CLI | Zene |
| :--- | :--- | :--- |
| **Positioning** | General AI coding assistant, focused on Natural Language to Shell Commands | **High-Performance AI Coding Engine**, focused on Codebase Understanding & Complex Task Execution |
| **Form Factor** | Command Line Interface (CLI) | **Headless Daemon** + Multiple Clients (CLI, IDE) |
| **Model** | Bound to OpenAI Codex Model (Closed source) | **Model Agnostic**, supports OpenAI, Anthropic, DeepSeek, etc. |
| **Ecosystem** | Closed ecosystem, mainly serving the OpenAI platform | **Open Ecosystem**, connects to any tool via standard JSON-RPC |

## 2. Architecture Comparison

### OpenAI Codex CLI
*   **Interaction Mode**: `Natural Language -> Bash Command`. User inputs natural language, Codex generates the corresponding Shell command for user confirmation and execution.
*   **Context**: Mainly relies on the file list of the current directory (`ls`) and Shell history. Context window is limited, making it difficult to handle large projects.
*   **Execution**: Executes commands directly in the user's Shell, dependent on the system's Shell environment.

### Zene
*   **Interaction Mode**: **OODA Loop (Observe-Orient-Decide-Act)**. User inputs a task, and the Agent autonomously performs context analysis, planning, tool invocation, and verification.
*   **Context**:
    *   **Tree-sitter**: Statically analyzes code AST to precisely extract function definitions and dependencies.
    *   **File Walking**: Efficiently scans project structure.
    *   **Vector Search** (Planned): Semantically retrieves code snippets.
*   **Execution**: Executes atomic operations (e.g., `read_file`, `write_file`) via **Internal Tools**, featuring sandboxing and Dry Run mechanisms.
*   **Communication**: Based on JSON-RPC over Stdio, can run as an IDE Language Server (LSP-like).

## 3. Key Features

### A. Safety
*   **Codex**: Relies on manual user confirmation for every generated command.
*   **Zene**:
    *   **Dry Run**: Generates a Diff first for high-risk operations (like file writing).
    *   **Human-in-the-loop**: Sensitive operations require explicit authorization.
    *   **Atomic Edits**: Ensures atomicity of file modifications to prevent intermediate states from breaking code.

### B. Extensibility
*   **Codex**: Features are relatively fixed.
*   **Zene**:
    *   **External Integrations**: Connects to external tools (e.g., Database, Git, Slack) via standard protocols.
    *   **Wasm Plugins**: (Optional) Supports user-written WebAssembly plugins to extend capabilities.

### C. Performance
*   **Codex**: Python/Node.js implementation (presumed), startup speed affected by runtime.
*   **Zene**: **Rust Native**, lightning-fast startup, low memory footprint, suitable for running as a resident daemon.

## 4. Conclusion

Zene is not just a "better CLI"; it is an **intelligent programming backend**. Borrowing from Codex's ease of use, Zene goes a step further by deeply understanding code structure (Tree-sitter) and utilizing standardized protocols, aiming to become the core intelligent engine in developer IDEs and workflows.
