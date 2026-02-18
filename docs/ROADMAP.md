# Zene Roadmap

This roadmap outlines the evolution of Zene from a high-performance engine into a production-grade AI coding platform.

## 1. Advanced Knowledge & Context
- **LSP Integration**: Connect to Language Servers for "Go to Definition" and real-time diagnostics before running code.
- **Context Graph**: Build a cross-file knowledge graph to automatically include relevant model/schema definitions during edits.
- **Context Compaction**: Implement smart summarization of long session histories to maintain focus.

## 2. Interactive Human-in-the-Loop (HITL)
- **Proactive Clarification**: Agent pauses and asks for hints when stuck on a repetitive error.
- **Safety Gateways**: Explicit confirmation for high-risk actions (`rm -rf`, `git push`).
- **Web Dashboard**: An interactive UI to review diffs and guide the agent mid-task.

## 3. Toolchain & Environment
- **Multi-language Sandboxing**: Extend Python-style isolation to Node.js and Rust environments.
- **Docker Tooling**: Ephemeral containers for complex services (DBs, Redis) during integration testing.
- **AST-based Editing**: Move beyond text-based patches to precise AST manipulation for zero-conflict edits.

## 4. Observability & Developer Experience
- **TUI Dashboard**: A rich terminal UI showing task progress, DAG status, and real-time logs.
- **Artifact Summaries**: Automated reports on latency, token costs, and generated assets per session.
- **Integrated Benchmarking**: Automated evaluation suites to measure Agent success rates on real-world coding tasks.
