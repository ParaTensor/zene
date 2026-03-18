---
layout: home

hero:
  name: "$ zene start"
  text: "Embeddable Agent Execution Engine"
  tagline: "A Rust runtime for planning, tool execution, session state, and event streaming in coding workflows."
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
  - title: Embeddable Runtime
    details: Use Zene as a CLI runtime, Rust library, or backend for your own developer tools.
    icon: ⚙️
  - title: Observable Execution
    details: Stream structured agent events for logs, UIs, dashboards, and host-side control planes.
    icon: 📡
  - title: Tool Runtime
    details: Execute shell commands, file edits, Python tasks, and repo operations behind a single engine facade.
    icon: ⚡
  - title: Extensibility (MCP)
    details: Connect to any external tool via Model Context Protocol. Git, Postgres, Filesystem, and more.
    icon: 🔌
  - title: Strategy-Friendly
    details: Keep orchestration policies above the runtime so products can choose direct, planned, or workflow-driven execution.
    icon: 🧭
---

<div align="center" style="margin: 40px auto; max-width: 900px;">
  <img src="/images/demo-terminal.svg" alt="Zene Terminal Demo" style="border-radius: 12px; box-shadow: 0 8px 30px rgba(0,0,0,0.5);" />
</div>

# Runtime Model

**Zene** is the execution substrate behind coding agents and developer workflows. It provides the runtime primitives that higher-level products need.

1.  **State**: Maintain session history, environment, and execution progress.
2.  **Execute**: Run tools, shell commands, and file operations through one runtime.
3.  **Observe**: Emit events so the host can inspect, persist, or steer the run.

## Installation

```bash
cargo install zene
```

```bash
# Verify installation
zene --version
```

[Read the Documentation →](/guide/getting-started)
