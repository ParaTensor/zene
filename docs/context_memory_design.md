# Context Memory Design (RAG)

## Problem
Currently, Zene relies on:
1.  **Exact Keyword Search (`grep`)**: Fails if the user uses different terminology (e.g., "auth" vs "login").
2.  **Full File Reading**: Consumes massive tokens and hits context window limits.
3.  **Ephemeral Context**: Learnings from previous sessions or tasks are lost.

## Solution: Semantic Vector Memory
Implement a local Retrieval-Augmented Generation (RAG) system to allow Zene to find "conceptually related" code and history.

## Architecture: The Hybrid Tiered Strategy

Inspired by modern best practices (including Codex's 2026 evolution), Zene uses a **Tiered Context System** to balance precision, cost, and recall.

### Tier 1: Explicit Context (Lazy Loading) - *High Precision*
*   **Mechanism**: Only files explicitly requested by the Agent via `read_file` or `grep` are loaded.
*   **Storage**: Direct context window.
*   **Behavior**: Default mode. Keeps the context clean and focused on valid data.

### Tier 2: Semantic Reference (RAG) - *Adaptive Recall*
*   **Mechanism**: Background `fastembed` + `usearch` (or `lance`).
*   **Infrastructure**: **Embedded / Serverless**.
    *   **No Docker**: Does NOT require running a separate container (like Qdrant/Milvus).
    *   **No API**: Runs 100% locally on CPU.
    *   **File-based**: Data lives in `~/.zene/memory/`, just like a SQLite file.
*   **Usage**: When Planning, the Agent searches this layer to find "how we usually do things".

### Tier 3: Session Compaction - *Short-Term focus*
*   **Mechanism**: Summarize long conversations to keep the active window fresh.
### Tier 3: Session Compaction - *Short-Term focus*
*   **Mechanism**: Summarize long conversations to keep the active window fresh.
*   **Goal**: Ensure the Agent remembers the *current task's* decisions without getting confused by ancient history.

## Alternatives Analysis: Why not just `grep`?

We considered a pure Keyword Search (ripgrep) approach. Here is the trade-off:

| Feature | `grep` / `ripgrep` | `fastembed` (RAG) |
| :--- | :--- | :--- |
| **Principle** | Exact String Match | Semantic Meaning |
| **Query** | "find function `login`" | "how does auth work?" |
| **Success Case** | You know the *exact* name. | You have a vague *intent*. |
| **Failure Case** | Typo or synonym. | Subtle distinct concepts. |
| **Resource** | Near Zero. | ~200MB RAM, CPU burst. |
| **Role in Zene** | **Tool** (Execution). | **Memory** (Planning/Recall). |

**Decision**: We need **BOTH**.
*   **Grep**: For precise execution ("I need to edit `src/main.rs`").
*   **RAG**: For exploration ("Where is the error handling logic?").

## Implementation Plan

1.  **Dependencies**: Add `fastembed`, `usearch`.
2.  **Memory System**: Implement `src/engine/memory/` focusing purely on RAG (Indexing & Retrieval).
3.  **Integration**: Add `memory_search` tool.

## Performance & Resource Strategy

**Q: Will this kill my Cloud VM (e.g., AWS t3.medium)?**
**A: No, because we use a "Burst & Sleep" strategy.**

1.  **CPU Usage (Indexing Phase)**:
    *   **Intensity**: Embedding generation is CPU-intensive.
    *   **Mitigation**: We run indexing on a **low-priority background thread** (`nice`). We can also limit `rayon` concurrency to 1 thread to prevent UI/Terminal lag.
    *   **Cost**: `all-MiniLM-L6-v2` is extremely fast (~20ms per sentence). Indexing a medium repo (200 files) takes seconds, not minutes.

2.  **RAM Usage**:
    *   **Model**: ~80MB (quantized) or ~200MB (full).
    *   **Vector Index**: `usearch` is disk-based with memory mapping. For <10k vectors (typical repo), RAM usage is negligible (<50MB).

3.  **Cloud Compatibility**:
    *   Works fine on 2 vCPU / 4GB RAM instances.
    *   Compiles to static binary (no dependency hell).


## Implementation Plan

1.  **Dependencies**: Add `fastembed`, `usearch`.
2.  **Memory Manager**: Create `src/engine/memory/` for RAG.
3.  **Rule Engine**: Add `AGENTS.md` loader in `ContextEngine`.
4.  **Integration**:
    *   Update `AgentRunner` to support Tier 3 (Rules) immediately.
    *   Implement Tier 2 (RAG) as a tool.
    *   Design Tier 4 (Compaction) for future optimization.
