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
---

<div class="terminal-demo">
  <div class="terminal-header">
    <span class="dot red"></span>
    <span class="dot yellow"></span>
    <span class="dot green"></span>
  </div>
  <div class="terminal-body">
    <div class="line"><span class="prompt">$</span> zene run "Fix the bug in src/main.rs"</div>
    <div class="line text-dim">Analyzing codebase...</div>
    <div class="line text-info">ℹ Planner: Identified syntax error on line 42</div>
    <div class="line text-success">✔ Executor: Applied fix</div>
    <div class="line text-warning">⚠ Reflector: Compilation failed. Retrying...</div>
    <div class="line text-success">✔ Executor: Adjusted lifetime parameters</div>
    <div class="line text-success">✔ Reflector: All tests passed.</div>
    <div class="line"><span class="prompt">$</span> <span class="cursor">_</span></div>
  </div>
</div>

<style>
.terminal-demo {
  background: #1e1e1e;
  border-radius: 6px;
  box-shadow: 0 4px 20px rgba(0,0,0,0.5);
  margin: 40px auto;
  max-width: 800px;
  font-family: 'JetBrains Mono', monospace;
  overflow: hidden;
  border: 1px solid #333;
}
.terminal-header {
  background: #2d2d2d;
  padding: 10px;
  display: flex;
  gap: 8px;
}
.dot { width: 12px; height: 12px; border-radius: 50%; }
.red { background: #ff5f56; }
.yellow { background: #ffbd2e; }
.green { background: #27c93f; }
.terminal-body {
  padding: 20px;
  color: #f8f8f2;
}
.line { margin-bottom: 8px; }
.prompt { color: #ff79c6; margin-right: 8px; }
.text-dim { color: #6272a4; }
.text-info { color: #8be9fd; }
.text-success { color: #50fa7b; }
.text-warning { color: #ffb86c; }
.cursor { animation: blink 1s step-end infinite; }
@keyframes blink { 50% { opacity: 0; } }
</style>

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
