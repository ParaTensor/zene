# The Self-Healing Compiler: When AI Refuses to Accept "Good Enough"

We've all been there: You ask an AI to write some Rust code. It looks perfect. You copy-paste it into your IDE, run `cargo build`, and the borrow checker immediately screams at you.

Then begins the tedious copy-paste dance. You paste the error back to the AI. It apologizes, gives you a "fix". You try again. Maybe it works, maybe it introduces a new error.

This loop is broken. The AI is guessing, not verifying. It's hallucinating a world where that lifetime parameter `'a` makes sense, while the compiler lives in reality.

Today, I ran an experiment with **Zene** (my Rust-based AI Agent Engine) to see if we could close this loop. I didn't want a chatbot that *suggests* fixes; I wanted an engineer that *verifies* them.

## The Experiment: A Broken Rust Project

I created a deliberately broken Rust project. It had everything a junior Rustacean nightmares about: a dangling reference (lifetime error) and a type mismatch.

### The Broken Code
```rust
fn get_longest<'a>(s1: &'a str, s2: &str) -> &'a str {
    if s1.len() > s2.len() {
        s1
    } else {
        s2 // ❌ ERROR: s2 does not live long enough ('a)
    }
}

fn calculate(val: i32) -> Result<i32, String> {
    if val < 0 {
        return Err("Negative value".to_string());
    }
    val * 2 // ❌ ERROR: expected Result, found i32
}
```

I gave Zene a simple instruction: **"Fix this code."**

In a traditional RAG or Chat loop, the model would look at the code and say, "Oh, you need to clone `s2` here." It might be right, or it might just be guessing.

But Zene is different. It's built on a **Plan-Execute-Reflect** architecture.

## The "Lazy" Executor

First, the **Planner (DeepSeek)** correctly identified that it needed to run `cargo build` to see the errors. Smart.

Then, the **Executor (Zhipu GLM-4)** ran the build and saw the errors:
```
error[E0312]: lifetime of reference outlives lifetime of borrowed content...
error[E0308]: mismatched types... expected `Result<i32, String>`, found `i32`
```

It attempted a fix. It modified `get_longest` to return a `String` (owned type) instead of a reference, which resolved the lifetime issue.

**But here is where things usually go wrong.** In most agent frameworks, the agent would stop here. "I fixed the code," it would say. "Here is the result."

And if I had run that result, it might have failed on the *next* error (the type mismatch in `calculate`). Or maybe the fix introduced a warning.

## Enter The Reflector

This is where **Minimax (the Reflector)** stepped in.

The Executor tried to mark the task as complete. But the Reflector intercepted the result. It looked at the execution log and noticed something missing: **The Executor hadn't run `cargo build` *after* the fix.**

It rejected the task.

> **Reflector Verdict**: "You claimed to fix the code, but you didn't verify it. Run `cargo build` again to prove it compiles."

This interaction is subtle but profound. The AI is enforcing engineering discipline on itself. It is no longer just generating text; it is adhering to a process.

## The Self-Healing Loop

Forced by the Reflector, the Executor went back to work. It ran `cargo build` again.

Surprise! There was still a type mismatch error in `calculate`. The first fix wasn't enough.

Because the task was rejected, Zene's engine automatically generated a **Fix Task**. The Executor read the new error message, adjusted the return type to `Ok(val * 2)`, and ran `cargo build` a third time.

### The Fixed Code
```rust
fn get_longest(s1: &str, s2: &str) -> String { // ✅ Changed to owned String
    if s1.len() > s2.len() {
        s1.to_string()
    } else {
        s2.to_string()
    }
}

fn calculate(val: i32) -> Result<i32, String> {
    if val < 0 {
        return Err("Negative value".to_string());
    }
    Ok(val * 2) // ✅ Wrapped in Ok()
}
```

**Success.** The project compiled.

Only then did the Reflector sign off: "Approved. The code compiles without errors."

## Why This Matters

We often talk about "Agentic Workflows" as if they are just chains of prompts. But true agency comes from **feedback loops**.

In software engineering, the compiler is the ultimate source of truth. By giving the AI access to that truth—and, more importantly, *forcing* it to respect that truth via a Reflector—we move from "AI that guesses" to "AI that engineers."

Zene is open source. You can try this self-healing workflow today.

[github.com/lipish/zene](https://github.com/lipish/zene)
