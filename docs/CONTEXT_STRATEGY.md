# Zene Context Strategy

Zene adopts a multi-layered, structured context retrieval and management strategy designed to solve the "limited context window" problem of LLMs when dealing with large codebases, providing precise code understanding capabilities.

## 1. Context Layers

Zene divides context into three depth layers, which the Agent loads dynamically based on task requirements:

### L1: Project Structure
*   **Tool**: `ignore` (File walking library similar to ripgrep)
*   **Content**: List of file names, directory tree structure.
*   **Purpose**: To quickly give the Agent a macro understanding of the project and locate potentially relevant modules.
*   **Trigger**: When the session initializes, or when the user asks about "project structure".

### L2: Syntax Structure - **Core Capability**
*   **Tool**: `tree-sitter` (Incremental parser)
*   **Content**:
    *   **Definitions**: Function signatures, struct/enum definitions, Trait declarations.
    *   **References**: Cross-file symbol reference relationships.
    *   **Imports**: File dependency graph.
*   **Purpose**: To understand the Interface Contract without reading the full source code, significantly saving Tokens and improving accuracy.
*   **Trigger**: When analyzing code dependencies, finding definitions, or understanding API usage.

### L3: Semantic Content
*   **Tool**: Direct file reading, (Planned) Vector Database
*   **Content**: Full source code of files, comments, docstrings.
*   **Purpose**: To provide the specific details needed for modifying code.
*   **Trigger**: When a specific file to be modified is identified, or when detailed logic needs to be read.

## 2. Dynamic Context Management

The Agent maintains a dynamic Context Window to avoid irrelevant information interfering with the model:

*   **Expansion (On-Demand)**:
    *   When the Agent sees `use auth::Login;` but doesn't know the definition of `Login`, it triggers the `read_definition` tool.
    *   The system grabs only the definition part of the `Login` struct from `auth.rs` and injects it into the context, rather than reading the entire file.
*   **Pruning (Smart Pruning)**:
    *   Automatically removes AST nodes or file contents unrelated to the current task.
    *   Prioritizes retaining L3-level core code and L2-level related interface definitions.

## 3. External Integration

Zene's context capabilities can be exposed externally:

*   **Resources**: Provides files as resources (`file://...`).
*   **Prompts**: Provides preset Context templates (e.g., "Code Review Context", "Refactor Context").
*   **Tools**: Exposes advanced context tools like `get_definitions`, `find_references` to other Agents.

## 4. Context Optimization

*   **Token Counting**: Real-time monitoring of context Token count, triggering compression or summarization when thresholds are exceeded.
*   **Summarization**: Generates summaries for long files, retaining only key logic descriptions.
*   **Ranking**: Sorts retrieved code snippets by relevance to ensure the most important context appears earlier in the Prompt.
