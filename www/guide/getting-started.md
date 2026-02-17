# Getting Started with Zene

Welcome to **Zene**, the minimalist AI coding engine. This guide will walk you through installing, configuring, and running your first agentic workflow.

## Installation

### Prerequisites
Zene is written in Rust, so you'll need the Rust toolchain installed.

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build from Source
Currently, the best way to install Zene is to build it from source.

```bash
git clone https://github.com/lipish/zene.git
cd zene
cargo install --path .
```

Verify the installation:
```bash
zene --version
```

## Configuration

Zene follows the "12-factor app" philosophy and uses environment variables for configuration. No complex YAML files required.

### Set your API Keys

Zene uses a multi-model architecture. You can mix and match providers.

```bash
# Planner (DeepSeek V3 is recommended)
export ZENE_PLANNER_PROVIDER="deepseek"
export ZENE_PLANNER_API_KEY="sk-..."

# Executor (Zhipu GLM-4 Flash is fast and cheap)
export ZENE_EXECUTOR_PROVIDER="zhipu"
export ZENE_EXECUTOR_API_KEY="sk-..."

# Reflector (Minimax is excellent for critique)
export ZENE_REFLECTOR_PROVIDER="minimax"
export ZENE_REFLECTOR_API_KEY="sk-..."
```

## Usage

### One-Shot Mode
Execute a single instruction directly from the command line. Perfect for quick scripts or refactoring.

```bash
# Create a file
zene run "Create a hello.txt with content 'Hello Zene'"

# Refactor code (Context aware)
zene run "Refactor src/main.rs to extract the CLI logic into a separate module"
```

### Server Mode (JSON-RPC)
Start Zene as a persistent server. This is how IDEs and other tools integrate with Zene.

```bash
zene server
```

The server communicates via Stdio using JSON-RPC 2.0.

**Example Request:**
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

## Next Steps

- Learn about the [Architecture](/guide/architecture)
- See [Examples](/examples/)
- Read the [Blog](/blog/)
