# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0] - 2026-02-17

### 🚀 Major Features

*   **Python V3 Execution Engine**: 
    *   **Isolated Virtual Environment**: Automatically manages `.venv` for projects, prioritizing `uv` for speed.
    *   **Session-Scoped Isolation**: Environment variables are stored per-session and injected only into child processes, preventing global state pollution.
    *   **Safe Execution**: Added timeouts (60s) and zombie process prevention for shell commands.

### ✨ Enhancements

*   **New Tools**: Added `run_python`, `set_env`, `get_env` to the toolset.
*   **Examples**: Added 7 comprehensive scenarios covering Data Analysis, DevOps, Testing, and Web Scraping.
*   **Documentation**: Added technical blog posts and architecture docs (`docs/python_env.md`, `docs/architecture_isolation.md`).

### 🛠️ Fixes

*   **Reflector**: Robust JSON extraction logic to handle LLM `<think>` tags (fixing "fail open" bug).
*   **Git**: Properly ignore `.venv` directories recursively.

## [0.2.2] - 2026-02-16

### 🛠️ Fixes

*   Cleaned up compiler warnings (unused imports, dead code).
*   Refined `Reflector` implementation details.

## [0.2.1] - 2026-02-16

### ✨ Enhancements

*   **Native Minimax Support**: Added explicit support for "minimax" provider with correct API endpoints.
*   **Reflector Improvements**: Enhanced `Reflector` logic to better integrate with Executor and Planner, enabling a true "Self-Healing Loop".
*   **Documentation**: Added social media content templates (Blog, X, Xiaohongshu) to `docs/` (excluded from git).

## [0.2.0] - 2026-02-16

### 🚀 Major Features

*   **Multi-Agent Architecture**: Introduced `Planner` and `Executor` roles. Zene can now break down complex tasks into sequential steps and execute them autonomously.
*   **Multi-Provider Support**: Added support for configuring different LLM providers (DeepSeek, Zhipu, Minimax, etc.) for different roles via environment variables.
*   **Persistent Sessions**: Sessions now store execution plans and task status, allowing for stateful interactions and recovery.

### ✨ Enhancements

*   **Smart Planning**: The Planner Agent analyzes project context (file structure) before creating a task list.
*   **Robust Tooling**: Enhanced `read_file`, `write_file`, `list_files` tools. Added `search_code` (grep) for efficient code navigation.
*   **Fuzzy Patching**: Implemented a fuzzy-match algorithm for `apply_patch`, making code modifications more resilient to minor context mismatches.
*   **Documentation**: Added comprehensive architecture documentation in `docs/` and updated `README.md` with the project philosophy.

### 🛠️ Fixes

*   Fixed various compilation warnings and unused imports.
*   Improved error handling in tool execution.
*   Resolved path resolution issues in file operations.

## [0.1.2] - 2026-02-14

### Initial Release

*   Basic CLI and Server mode (JSON-RPC).
*   Single-turn agent execution loop.
*   Core tools: `read_file`, `write_file`, `run_command`.
*   Integration with `llm-connector` for basic OpenAI/DeepSeek support.
