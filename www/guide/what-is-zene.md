# What is Zene?

Zene is a minimalist, high-performance **agent execution engine** written in Rust. Its main job is not to be an end-user copilot UI. Its main job is to provide a reusable runtime for coding agents, developer tools, and internal automation systems.

## Core Philosophy

"Make execution a product surface, not an implementation detail."

Zene assumes that orchestration patterns will evolve, but runtime primitives should remain stable. A host should be able to reuse the same engine for direct execution, plan-driven execution, or external workflow orchestration.

1.  **Execution Kernel**: Own runtime state, tool calls, persistence, and event emission.
2.  **Decision Strategy**: Decide whether to plan, route, parallelize, reflect, or stop.
3.  **Product Policy**: Choose the right strategy for a given workflow or integration surface.

This keeps Zene small at the core and flexible at the edges.

## What Zene Is For

Zene is useful when you need:

- a Rust-native execution runtime for coding tasks
- structured event streaming for a host UI or service
- session-aware tool execution with persistence and recovery
- a base engine that can sit under a CLI, IDE integration, or internal platform

Zene can still expose higher-level workflows such as plan-execute-reflect, but those workflows should be treated as strategies on top of the runtime rather than as the runtime itself.

## Why Rust?

- **Speed**: Instant startup and low memory footprint compared to Python/Node.js agents.
- **Safety**: Robust error handling and type safety.
- **Concurrency**: Efficiently manage tool execution, streaming events, and host integrations.
