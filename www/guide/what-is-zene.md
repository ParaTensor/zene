# What is Zene?

Zene is a minimalist, high-performance **autonomous coding engine** written in Rust. Unlike standard AI coding assistants that simply autocomplete code, Zene operates as a continuous loop of **Planning**, **Execution**, and **Reflection**.

## Core Philosophy

"Build. Break. Fix. Repeat."

Zene assumes that code generation is imperfect. Instead of trying to get it right the first time, Zene relies on a fast feedback loop to verify its own work.

1.  **Plan**: Break down a high-level goal into a Directed Acyclic Graph (DAG) of tasks.
2.  **Execute**: Perform actions (edit files, run commands) to complete a task.
3.  **Reflect**: Analyze the outcome. Did the build fail? Did tests pass? If not, create a fix task.

## Why Rust?

- **Speed**: Instant startup and low memory footprint compared to Python/Node.js agents.
- **Safety**: Robust error handling and type safety.
- **Concurrency**: Efficiently manage multiple agent roles (Planner, Executor, Reflector) in parallel.
