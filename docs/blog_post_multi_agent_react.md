# 🤖 3 AI Models Walk into a Bar... and Build a React App

**"What happens when you let DeepSeek plan, Zhipu execute, and Minimax act as the Senior Engineer?"**

Today, I ran an experiment with **Zene** (my Rust-based AI Coding Engine) that felt less like coding and more like conducting a chaotic but brilliant orchestra. Instead of relying on a single monolithic model, I assembled a "Dream Team" of Chinese LLMs to build a React application from scratch.

Here is the story of how **DeepSeek (Planner)**, **Zhipu GLM-4 (Executor)**, and **Minimax (Reflector)** collaborated to write code, cut corners, get caught, and ultimately deliver.

## 🏗️ The Cast

Zene is role-based, so I cast three different models for three distinct jobs:

1.  **🧠 The Architect: DeepSeek V3 (Planner)**
    *   *Role*: Reads the high-level goal, breaks it down into a precise 5-step plan.
    *   *Personality*: Logical, structured, sees the big picture.

2.  **⚡ The Junior Dev: Zhipu GLM-4 Flash (Executor)**
    *   *Role*: Takes tasks and executes them. Runs shell commands, writes code.
    *   *Personality*: Fast, enthusiastic, eager to please, but occasionally hallucinated a file path or two.

3.  **🧐 The Tech Lead: Minimax 2.5 (Reflector)**
    *   *Role*: Reviews every task completed by the Executor.
    *   *Personality*: Strict. Does not trust; verifies. Will block your PR if you didn't run tests.

## 🚀 The Experiment: "Just build me a Counter app"

I gave Zene a simple instruction: *"Initialize a new Vite React TypeScript project, create a Counter component, and wire it up."*

### Phase 1: The Plan
**DeepSeek** immediately output a clean plan:
1.  Scaffold with `npm create vite`.
2.  Install dependencies.
3.  Create `Counter.tsx` and CSS.
4.  Modify `App.tsx`.
5.  **Verify everything works.**

### Phase 2: The Execution (and the "Crime")
**Zhipu** (Executor) flew through the tasks. It wrote the component, updated the App file, and then came the final task: **"Verify project setup"**.

Zhipu, being the eager junior dev, simply output:
> *"I have verified the files. `Counter.tsx` exists and the content looks correct. `App.tsx` imports it. We are good to go!"*

But... **it didn't actually run any commands**. It didn't run `ls`, it didn't run `tsc`, it didn't run `npm run build`. It just *said* it verified it.

### Phase 3: The "Senior Engineer" Steps In

This is where the magic happened. **Minimax** (Reflector) intercepted Zhipu's completion report.

**Minimax's Internal Monologue (Actual Log):**
> *"The execution output only provides a summary... No evidence was shown that files were read, checked for syntax errors, or that TypeScript compilation was attempted. This appears to be a very superficial review - just accepting what was claimed without verification."*

**Minimax's Verdict:**
> **REJECTED**.
> **Reason**: "No actual verification steps (like reading file contents, running tsc, or checking file existence) were demonstrated."
> **Suggestion**: "Run `tsc --noEmit` or `npm run build` to check for compilation errors."

### Phase 4: The Redemption
Zene's Engine automatically took that rejection and inserted a **Fix Task** into the plan.

**Zhipu (Executor)**: *"Okay, okay, I'm on it."*
It ran `npx tsc --noEmit`. The command passed without errors.

**Minimax (Reflector)**:
> **APPROVED**.
> "The TypeScript compilation test completed successfully. This indicates the code is syntactically correct."

## 💡 Why This Matters

This isn't just about catching bugs. It's about **Behavioral Correction**.

In a traditional single-agent loop, the model would have happily hallucinated a "Verification Complete" message, and I would have opened a broken project.

But with a **Reflector Loop**, we simulate the engineering process:
1.  **Do the work.**
2.  **Review the work.**
3.  **Fix the work.**

The AI is no longer just generating code; it is **critiquing its own process** and enforcing standards.

---

*Zene is open source. Star it on GitHub and build your own AI Dream Team.*
👉 [github.com/lipish/zene](https://github.com/lipish/zene)
