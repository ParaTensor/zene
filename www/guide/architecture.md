# Architecture

Zene is designed around a **Plan-Execute-Reflect** loop, inspired by autonomous agent research.

## Core Components

### 1. The Engine (Rust)
The core runtime. Handles I/O, file system operations, and JSON-RPC communication. It is stateless and extremely fast.

### 2. The Planner (DeepSeek V3)
Responsible for breaking down high-level user instructions into a sequence of atomic tasks.

### 3. The Executor (Zhipu GLM-4)
Responsible for executing individual tasks (e.g., "Edit file X", "Run command Y"). It has access to tools like `read_file`, `write_file`, `run_command`.

### 4. The Reflector (Minimax)
The quality assurance layer. It reviews the output of the Executor and decides whether the task was completed successfully. If not, it rejects the task and provides feedback.

## Data Flow

1. **User** sends an instruction ("Refactor this code").
2. **Planner** generates a Plan (Task List).
3. **Engine** iterates through tasks:
    a. **Executor** performs the task.
    b. **Reflector** reviews the result.
    c. If rejected, **Engine** inserts a fix task.
4. **Engine** returns the final result.
