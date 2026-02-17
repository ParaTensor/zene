# Future Roadmap & Optimization Insights

Based on the execution of 7 core scenarios (Refactoring, Self-Healing, Data Analysis, API, DevOps, Testing, Scraping), we have identified critical areas for Zene's evolution. This document outlines the roadmap to transform Zene from an MVP into a production-grade AI coding engine.

## 1. Dependency Management & Isolation (The "Environment" Problem)

**Current Issue**: Zene relies on the host environment. Missing dependencies (e.g., `pandas`, `requests`) cause failures unless the Reflector "self-heals" by installing them, which is slow and pollutes the user's global environment.

**Optimization Strategy**:
*   **Automatic Virtual Environments**: Detect project type (Python/Node/Rust) and automatically create/activate isolated environments (e.g., `.venv`, `node_modules`).
*   **Smart Toolchain Detection**: Use modern, fast package managers like `uv` (Python), `pnpm` (Node), or `cargo` (Rust) for instant dependency resolution.
*   **Docker Integration**: For complex tasks (e.g., databases), launch ephemeral Docker containers to run code safely without affecting the host.

## 2. Enhanced Tooling (Beyond grep/sed)

**Current Issue**: Basic tools like `read_file` and `write_file` are blunt instruments. Editing large files via full rewrite is inefficient and error-prone. `apply_patch` is fragile.

**Optimization Strategy**:
*   **AST-based Editing**: Implement language-aware tools (for Python/JS/Rust) that can insert imports, add functions, or rename variables by manipulating the Abstract Syntax Tree, not just text strings.
*   **LSP Integration**: Connect to Language Servers (LSP) to provide the Agent with "IDE-like" powers: Go to Definition, Find References, and real-time Diagnostics (linting errors) *before* running the code.

## 3. Context Management (The "Memory" Problem)

**Current Issue**: In multi-file projects (e.g., FastAPI), the Executor often loses track of cross-file dependencies or consumes excessive tokens reading unrelated files.

**Optimization Strategy**:
*   **Smart Context Window**: maintain a "Knowledge Graph" of the codebase. When editing `models.py`, automatically include relevant snippets from `crud.py` and `schemas.py` in the context, but hide unrelated code.
*   **Vector Memory**: Use a lightweight local vector DB to persist the Agent's "train of thought" and key decisions across long sessions.

## 4. Human-in-the-Loop (Interactive Feedback)

**Current Issue**: Zene can get stuck in a loop trying to fix a persistent error (e.g., "port 5000 is occupied") when a simple human hint would solve it instantly.

**Optimization Strategy**:
*   **Proactive Clarification**: If the Reflector rejects a task twice with similar errors, the Agent should pause and ask the user: "I'm stuck on X. Do you have a suggestion?"
*   **Safety Confirmation**: Require explicit user approval for high-risk actions like `rm -rf`, `git push`, or modifying critical configuration files.

## 5. Observability & UX

**Current Issue**: The current CLI output is a stream of text logs. It's hard to see the "big picture" of the task progress.

**Optimization Strategy**:
*   **TUI Dashboard**: A rich terminal UI showing the Task DAG, current step status, and real-time logs.
*   **Artifact Management**: Automatically collect and present generated outputs (charts, reports, binaries) in a clean summary at the end of the run.
