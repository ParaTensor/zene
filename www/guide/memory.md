# Context Memory

Zene employs a sophisticated tiered memory system to handle large codebases and long-running sessions without overwhelming the LLM's context window.

## Tiered Architecture

Context is loaded dynamically based on task depth to save tokens and improve accuracy.

### Tier 1: Project Structure (L1) / Hot Context
- **Tool**: `ignore` (File walking).
- **Content**: Directory tree, filenames, and immediate conversation history.
- **Purpose**: Macro understanding and immediate dialogue.
- **Trigger**: Session initialization.

### Tier 2: Syntax Structure (L2) - *Core*
- **Tool**: `tree-sitter` (Parsing).
- **Content**: Function signatures, struct/enum definitions, and cross-file references.
- **Purpose**: Interface contracts without reading full source code.
- **Trigger**: Dependency analysis and definition lookups.

### Tier 3: Semantic Vector Memory (RAG)
- **What**: Semantic search over the entire project codebase.
- **Status**: **Opt-in** (Disabled by default to save ~200MB RAM).
- **Mechanism**:
    - Uses `fastembed-rs` to generate embeddings for code chunks.
    - Uses `usearch` for high-performance local vector searching.
    - Enables the agent to find relevant code snippets based on natural language queries.
- **Tools**: `memory_search` and `context_search`.

### Tier 4: Session Compaction
- **What**: Summarization of older conversation history.
- **Mechanism**:
    - When history exceeds a threshold, the `SessionCompactor` summarizes the middle portion, preserving system prompt and recent context.

## Performance & Opt-in

By default, Zene aims for a minimalist footprint. Semantic memory requires loading transformer models which consume significant memory.

To enable Tier 3 Vector Search, set the following environment variable:

```bash
export ZENE_USE_SEMANTIC_MEMORY=true
```

## Dynamic Management
- **Smart Pruning**: Automatically removes AST nodes unrelated to the current task.
- **Token Monitoring**: Real-time counting and summarization.
- **Relevance Ranking**: Retreived snippets are sorted by importance.
