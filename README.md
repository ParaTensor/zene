<div align="center">

# Zene



**An Embeddable Agent Execution Engine for Coding Workflows**

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
*   **Engine**: High-performance, reliable, and powerful. Written in Rust to act as the execution core behind coding agents and developer tools.

We believe in:
*   **Embeddable Runtime**: Zene should be usable from a CLI, a Rust library, or another host application.
*   **Single Binary**: No complex runtime dependencies.
*   **Explicit Configuration**: No magic, just standard environment variables.
*   **Observable Execution**: Structured events, session state, and tool boundaries should be visible to the host.

## Features

- **Embeddable Engine**: Use Zene as a Rust library, a CLI runtime, or an execution backend for other developer tools.
- **Async Native**: Built on `tokio`, Zene is non-blocking and suitable for concurrent integrations.
- **Streaming Events**: Real-time agent events make it easier to build UIs, logs, and execution dashboards on top.
- **Workspace Awareness**: `FileStateChanged` events enable IDE-like file tree updates in frontend integrations.
- **Structured Agent Loop**: Planning, execution, and reflection are implemented as explicit runtime stages instead of hidden prompt behavior.

## Why Zene

*   **Embeddable by Design**: The `ZeneEngine` facade exposes a host-friendly runtime API.
*   **Model Agnostic**: Built on `llm-connector`, supporting OpenAI, Anthropic, DeepSeek, Google Gemini, and more.
*   **Context Aware**: Uses `tree-sitter` and fast file walking to understand project structure.
*   **Blazing Fast**: Written in Rust with async I/O and structured concurrency.

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

### 3. Embed Zene in Your Own Host
Zene can also be embedded as a Rust library or consumed through its Python bindings. This is the recommended direction if you want to build your own IDE integration, automation service, or internal coding platform on top of the runtime.

Sessions are persisted under `~/.zene/sessions/<session_id>.json` when using the default file-backed session store.

## Documentation

Detailed documentation is available at [zene.run](https://zene.run) (or in the `www/` directory):

*   [Architecture Guide](https://zene.run/guide/architecture)
*   [Execution Kernel vs Strategy](https://zene.run/guide/design/execution-kernel)
*   [Context & Memory](https://zene.run/guide/memory)
*   [Technical Design Specs](https://zene.run/guide/design/multi-user)
*   [Project Roadmap](https://zene.run/roadmap)

## Contributing

Contributions are welcome! Please read our architecture documentation to understand the core philosophy before submitting PRs.

## License

MIT
