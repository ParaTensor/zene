<div align="center">

# Zene



**A High-Performance Multi-Agent Coding Engine with Autonomous Planning and Reflection**

[![Crates.io](https://img.shields.io/crates/v/zene.svg)](https://crates.io/crates/zene)
[![Build Status](https://github.com/lipish/zene/actions/workflows/release.yml/badge.svg)](https://github.com/lipish/zene/actions)
[![License](https://img.shields.io/crates/l/zene.svg)](https://github.com/lipish/zene/blob/main/LICENSE)

[Installation](#installation) • [Usage](#usage) • [Documentation](https://zene.run) • [Contributing](#contributing)

<img src="https://zene.run/images/demo-terminal.svg" alt="Zene Terminal Demo" width="800" style="border-radius: 10px; box-shadow: 0 4px 8px rgba(0,0,0,0.5);" />

</div>

---

## Philosophy

**Zene** combines **"Zen"** and **"Engine"**.

*   **Zen**: Minimalist, focused, and distraction-free. We avoid bloated frameworks to provide a pure coding flow.
*   **Engine**: High-performance, reliable, and powerful. Written in Rust to be the robust core driving your intelligent agents.

We believe in:
*   **Single Binary**: No complex runtime dependencies.
*   **Explicit Configuration**: No magic, just standard environment variables.
1.  **Plan**: Generates a DAG of tasks.
2.  **Execute**: Runs real commands in your shell (Async/Non-blocking).
3.  **Reflect**: Analyzes stdout/stderr and file changes.

## Features

- **Async Native**: Built on `tokio`, Zene is 100% non-blocking and ready for high-concurrency web integration.
- **Streaming Output**: Real-time "Thought Delta" events allow for immersive, typewriter-style UI experiences.
- **Self-Healing**: The Reflector loop automatically fixes linting errors and failed tests.

## Key Features

*   **Model Agnostic**: Built on `llm-connector`, supporting OpenAI, Anthropic, DeepSeek, Google Gemini, and more.
*   **Context Aware**: Uses `tree-sitter` for syntax-level code analysis and efficient file walking to understand project structure.
*   **Safe Execution**: Features an OODA (Observe-Orient-Decide-Act) loop with "Dry Run" capabilities and atomic file operations.
*   **JSON-RPC Server**: Functions as a standard server, exposing its capabilities to IDEs and other tools.
*   **Blazing Fast**: Written in pure Rust with async I/O.

## Installation

### Pre-built Binaries
Download the latest release for your platform from the [Releases Page](https://github.com/lipish/zene/releases).

### From Crates.io
```bash
cargo install zene
```

### Build from Source
```bash
git clone https://github.com/lipish/zene.git
cd zene
cargo build --release
```

## Usage

### 1. Set Environment Variables
Zene prioritizes DeepSeek but supports OpenAI as a fallback.

### 2. Run a Task (One-Shot)
Execute a single instruction directly from the command line.

```bash
# Create a file
zene run "Create a hello.txt with content 'Hello Zene'"

# Refactor code (Context aware)
zene run "Refactor src/main.rs to extract the CLI logic into a separate module"

# Fetch Web Content
zene run "Fetch https://example.com and summarize it in README.md"
```

### 3. Server Mode
Start Zene as a JSON-RPC server (over Stdio). This mode supports persistent sessions and multi-turn conversations.

```bash
zene server
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

Sessions are automatically persisted to `~/.zene/sessions/<session_id>.json`.

## Documentation

Detailed documentation is available at [zene.run](https://zene.run) (or in the `www/` directory):

*   [Architecture Guide](https://zene.run/guide/architecture)
*   [Context & Memory](https://zene.run/guide/memory)
*   [Technical Design Specs](https://zene.run/guide/design/multi-user)
*   [Project Roadmap](https://zene.run/roadmap)

## Contributing

Contributions are welcome! Please read our architecture documentation to understand the core philosophy before submitting PRs.

## License

MIT
