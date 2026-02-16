# 🤖 3 AI Models Walk into a Bar... and Build a React App

**"What happens when you let DeepSeek plan, Zhipu execute, and Minimax reflect?"**

Today, I ran an experiment with **Zene** (my Rust-based AI Coding Engine) that felt less like coding and more like conducting an orchestra. Instead of relying on a single monolithic model, I assembled a "Dream Team" of Chinese LLMs to build a React application from scratch.

Here is the story of how **DeepSeek (Planner)**, **Zhipu GLM-4 (Executor)**, and **Minimax (Reflector)** collaborated to write code, make mistakes, and fix them.

## 🏗️ The Architecture: Multi-Agent Collaboration

Zene is designed to be model-agnostic and role-based. I configured it with three distinct roles:

1.  **🧠 Planner (DeepSeek V3)**: The Architect.
    *   *Role*: Understands the high-level goal, breaks it down into sequential tasks, and understands the project structure.
    *   *Why*: DeepSeek has shown incredible reasoning capabilities, perfect for planning.

2.  **⚡ Executor (Zhipu GLM-4 Flash)**: The Engineer.
    *   *Role*: Takes a single task from the plan and executes it using tools (shell, read_file, write_file).
    *   *Why*: Fast, cost-effective, and follows instructions well. Perfect for the heavy lifting.

3.  **🧐 Reflector (Minimax 2.5)**: The QA Lead.
    *   *Role*: Reviews the output, checks for hallucinations, and ensures quality.
    *   *Why*: Strong logical consistency and long-context understanding.

## 🚀 The Experiment: "Build a React App"

I gave Zene a simple but broad instruction:
> *"Initialize a new Vite React TypeScript project, create a Counter component with CSS modules, and wire it all up."*

### Phase 1: The Plan (DeepSeek)

DeepSeek immediately analyzed the request and output a precise 5-step plan:
1.  Run `npm create vite@latest` to scaffold the project.
2.  Run `npm install` to install dependencies.
3.  Create `src/components/Counter.tsx` with the logic.
4.  Create `src/components/Counter.module.css` for styling.
5.  Modify `src/App.tsx` to integrate the component.

*Verdict: Spot on. No fluff, just actionable steps.*

### Phase 2: The Execution (Zhipu GLM-4)

Zhipu took the baton. It executed the shell commands, waited for `npm install` (which took a while!), and then started writing code.

It generated a beautiful functional component:
```tsx
// Counter.tsx
import { useState } from 'react';
import styles from './Counter.module.css';

export default function Counter() {
  const [count, setCount] = useState(0);
  // ...
}
```

*Verdict: Fast and syntactically correct.*

### Phase 3: The Glitch (The "Human" Element)

Then came `App.tsx`. Zhipu, in its enthusiasm, tried to import a logo that didn't exist in the path it assumed:
```tsx
import viteLogo from "../assets/vite.svg"; // ❌ Wrong path!
```

When I tried to run the app, Vite exploded: `Failed to resolve import`.

This is where the **Multi-Agent system shines**. In a single-shot generation, this would be a dead end. But here, it's just a ticket for the Reflector (or a follow-up task).

I simply told Zene: *"Fix the bug in App.tsx"*. The Agent identified the unused/broken import and removed it, simplifying the code to just show the Counter.

## 💡 The Philosophy: Zen & Engine

This experiment validates the core philosophy of **Zene** ("Zen" + "Engine"):

*   **Zen**: The developer stays in the flow. I didn't write a single line of React code; I just orchestrated the intent.
*   **Engine**: The Rust core handled the context, file I/O, and tool execution with blazing speed, while the LLMs provided the intelligence.

## 🔮 What's Next?

We are moving towards a **Self-Healing Loop**.
Imagine if the **Reflector (Minimax)** automatically intercepted that `viteLogo` error by reading the `npm run build` output, and instructed **Zhipu** to fix it—all before I even saw the error.

That's the future we are building.

---

*Zene is open source. Star it on GitHub and build your own AI Dream Team.*
👉 [github.com/lipish/zene](https://github.com/lipish/zene)
