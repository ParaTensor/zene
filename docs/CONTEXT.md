# Zene Context & Memory System

Zene uses a tiered approach to codebase understanding, combining precise syntax analysis with semantic recall to solve the "limited context window" problem.

## 1. The Tiered Context Strategy

Context is loaded dynamically based on task depth to save tokens and improve accuracy.

### Tier 1: Project Structure (L1)
- **Tool**: `ignore` (File walking).
- **Content**: Directory tree and filenames.
- **Purpose**: Macro understanding and file location.
- **Trigger**: Session initialization.

### Tier 2: Syntax Structure (L2) - *Core*
- **Tool**: `tree-sitter` (Parsing).
- **Content**: Function signatures, struct/enum definitions, and cross-file references.
- **Purpose**: Interface contracts without reading full source code.
- **Trigger**: Dependency analysis and definition lookups.

### Tier 3: Semantic Content (L3)
- **Tool**: Direct file reading.
- **Content**: Full source code and comments.
- **Purpose**: Specific details needed for code modification.
- **Trigger**: Identifying exact target code for editing.

---

## 2. Semantic Vector Memory (RAG)

While Tier 1-3 focus on the *current* state of the code, the Memory system provides "Adaptive Recall" for concepts and historical decisions.

### Hybrid Strategy
| Feature | `grep` / `ripgrep` | Semantic RAG (`fastembed`) |
| :--- | :--- | :--- |
| **Principle** | Exact String Match | Semantic Meaning |
| **Use Case** | Precise Execution | Exploration ("How does auth work?") |
| **Infrastructure**| Near Zero | Local (100% CPU, Serverless) |

### Memory Implementation
- **Local Embedding**: Uses `fastembed` (`all-MiniLM-L6-v2`) for local, fast vectorization.
- **Vector Store**: `usearch` for disk-based, memory-mapped vector search.
- **Storage**: Data persists in `~/.zene/memory/`.

## 3. Dynamic Management
- **Smart Pruning**: Automatically removes AST nodes unrelated to the current task to keep the context window fresh.
- **Token Monitoring**: Real-time counting and summarization when thresholds are exceeded.
- **Relevance Ranking**: Retreived snippets are sorted so the most important context appears earlier in the prompt.
