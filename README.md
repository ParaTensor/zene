# Zene

> A minimalist, high-performance AI coding engine written in Rust.

**Zene** is a headless AI coding agent designed to understand your codebase and execute complex programming tasks. It acts as an intelligent backend that can be integrated into CLI tools, IDEs, or other agentic workflows via standard JSON-RPC.

## � Philosophy

**Zene** combines **"Zen"** and **"Engine"**.

*   **Zen**: Minimalist, focused, and distraction-free. We avoid bloated frameworks to provide a pure coding flow.
*   **Engine**: High-performance, reliable, and powerful. Written in Rust to be the robust core driving your intelligent agents.

We believe in:
*   **Single Binary**: No complex runtime dependencies.
*   **Explicit Configuration**: No magic, just standard environment variables.
*   **Agentic Composition**: Specialized roles (Planner, Executor, Reflector) working in harmony.

## �🚀 Key Features

*   **Model Agnostic**: Built on `llm-connector`, supporting OpenAI, Anthropic, DeepSeek, Google Gemini, and more.
*   **Context Aware**: Uses `tree-sitter` for syntax-level code analysis and efficient file walking to understand project structure.
*   **Safe Execution**: Features an OODA (Observe-Orient-Decide-Act) loop with "Dry Run" capabilities and atomic file operations.
*   **JSON-RPC Server**: Functions as a standard server, exposing its capabilities to IDEs and other tools.
*   **Blazing Fast**: Written in pure Rust with async I/O.

## 📦 Installation

### Prerequisites
*   Rust toolchain (cargo)

### Build from Source
```bash
git clone https://github.com/lipish/zene.git
cd zene
cargo build --release
```

## 🛠️ Usage

### 1. Set Environment Variables
Zene prioritizes DeepSeek but supports OpenAI as a fallback. Semantic memory (RAG) is optional to keep baseline memory low (~200MB savings).

```bash
# API Keys
export DEEPSEEK_API_KEY="sk-..."
export OPENAI_API_KEY="sk-..."

# Optional: Enable Semantic Memory (RAG)
# Default is false. When false, fastembed models are not loaded.
export ZENE_USE_SEMANTIC_MEMORY=true
```

### 2. Run a Task (One-Shot)
Execute a single instruction directly from the command line.

```bash
# Create a file
cargo run -- run "Create a hello.txt with content 'Hello Zene'"

# Refactor code (Context aware)
cargo run -- run "Refactor src/main.rs to extract the CLI logic into a separate module"

# Fetch Web Content
cargo run -- run "Fetch https://example.com and summarize it in README.md"
```

### 3. Server Mode
Start Zene as a JSON-RPC server (over Stdio). This mode supports persistent sessions and multi-turn conversations.

```bash
cargo run -- server
```

#### JSON-RPC API Example

**Request (Start Session):**
```json
{
  "jsonrpc": "2.0",
  "method": "agent.run",
  "params": {
    "instruction": "Analyze the project structure",
    "session_id": "my-session-001"
  },
  "id": 1
}
```

**Request (Follow-up):**
```json
{
  "jsonrpc": "2.0",
  "method": "agent.run",
  "params": {
    "instruction": "Based on that, generate a README",
    "session_id": "my-session-001"
  },
  "id": 2
}
```

Sessions are automatically persisted to `~/.zene/sessions/<session_id>.json`.

## 📚 Documentation

Detailed documentation is available at [zene.dev](https://zene.dev) (or in the `www/` directory):

*   [Architecture Guide](https://zene.dev/guide/architecture)
*   [Context & Memory](https://zene.dev/guide/memory)
*   [MCP Extensions](https://zene.dev/guide/mcp)
*   [Technical Design Specs](https://zene.dev/guide/design/multi-user)
*   [Project Roadmap](https://zene.dev/roadmap)

## 🤝 Contributing

Contributions are welcome! Please read our architecture documentation to understand the core philosophy before submitting PRs.

## 📄 License

MIT
