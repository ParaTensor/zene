# Context Memory
2: 
3: Zene employs a sophisticated tiered memory system to handle large codebases and long-running sessions without overwhelming the LLM's context window.
4: 
5: ## Tiered Architecture
6: 
7: ### Tier 1: Hot Context
8: - **What**: The immediate conversation history and currently open files.
9: - **Mechanism**: Direct injection into the LLM prompt.
10: - **Limit**: Restricted by the model's context window (e.g., 128k tokens).
11: 
12: ### Tier 2: vector Search (RAG)
13: - **What**: Semantic search over the entire project codebase.
14: - **Mechanism**:
15:     - Uses `fastembed-rs` to generate embeddings for code chunks.
16:     - Uses `usearch` for high-performance local vector searching.
17:     - Enables the agent to find relevant code snippets based on natural language queries ("Find the authentication logic") rather than just regex keywoards.
18: - **Tools**: `memory_search` and `context_search`.
19: 
20: ### Tier 3: Session Compaction
21: - **What**: Summarization of older conversation history.
22: - **Mechanism**:
23:     - When the session history exceeds a threshold (default: 20 messages), the `SessionCompactor` activates.
24:     - It summarizes the middle portion of the conversation, preserving the initial system prompt and the most recent context.
25:     - This ensures the agent retains "long-term memory" of decisions without wasting tokens on old chit-chat.
26: 
27: ## Usage
28: 
29: The Context Engine manages these tiers automatically. When you run Zene:
30: 1. It indexes your project files (Tier 2).
31: 2. It monitors your session length (Tier 3).
32: 3. It proactively retrieves relevant context for tasks.
