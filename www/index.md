---
layout: home

hero:
  name: "Zene"
  text: "The Self-Healing AI Engine"
  tagline: "Plan. Execute. Reflect. An autonomous coding agent that writes, runs, and verifies code using Rust."
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/lipish/zene

features:
  - title: Plan (DeepSeek)
    details: Breaks down complex tasks into a structured, actionable plan. No hallucinations, just pure logic.
    icon: 🧠
  - title: Execute (Zhipu)
    details: Runs shell commands, edits files, and interacts with the OS. The hands that do the work.
    icon: ⚡
  - title: Reflect (Minimax)
    details: Reviews the output. Catches errors. Rejects bad code. The "Senior Engineer" that ensures quality.
    icon: 🧐
---

# Why Zene?

Traditional AI coding assistants are "fire and forget". They give you code, and you hope it works.

**Zene is different.**

It implements a **Plan-Execute-Reflect** loop. It doesn't just guess; it **verifies**.
- If the code doesn't compile, it fixes it.
- If the tests fail, it rewrites them.
- If the logic is flawed, it reflects and corrects.

Built with **Rust** for blazing speed and memory safety.

## Quick Start

```bash
# Install Zene
cargo install zene

# Set up your keys
export ZENE_PLANNER_API_KEY="..."
export ZENE_EXECUTOR_API_KEY="..."
export ZENE_REFLECTOR_API_KEY="..."

# Run it
zene run "Build a React app with a Counter component"
```

[Read the Documentation →](/guide/getting-started)
