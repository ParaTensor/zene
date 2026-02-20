---
layout: home

hero:
  name: "$ zene start"
  text: "Self-Healing AI Agent"
  tagline: "Build. Break. Fix. Repeat. <br/> A Rust-powered autonomous coding engine."
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/lipish/zene
    - theme: alt
      text: Download
      link: https://github.com/lipish/zene/releases

features:
  - title: Planner
    details: Breaks down complex tasks into structured JSON execution plans. Zero hallucination architecture.
    icon: 🧠
  - title: Executor
    details: High-performance Rust runtime for shell execution and file manipulation.
    icon: ⚡
  - title: Reflector
    details: Automated code review and error correction loop. Verifies logic before commit.
    icon: 🛡️
  - title: Extensibility (MCP)
    details: Connect to any external tool via Model Context Protocol. Git, Postgres, Filesystem, and more.
    icon: 🔌
  - title: Minimalist Footprint
    details: Opt-in Semantic Memory (RAG) saves ~200MB of RAM. High performance by design.
    icon: 🍃
---

<div align="center" style="margin: 40px auto; max-width: 900px;">
  <img src="/images/demo-terminal.svg" alt="Zene Terminal Demo" style="border-radius: 12px; box-shadow: 0 8px 30px rgba(0,0,0,0.5);" />
</div>

# The Loop

**Zene** is not just another copilot. It's an autonomous loop that **guarantees** code quality through self-reflection.

1.  **Plan**: Generates a DAG of tasks.
2.  **Execute**: Runs real commands in your shell.
3.  **Reflect**: Analyzes stdout/stderr and file changes.

## Installation

```bash
cargo install zene
```

```bash
# Verify installation
zene --version
```

[Read the Documentation →](/guide/getting-started)
