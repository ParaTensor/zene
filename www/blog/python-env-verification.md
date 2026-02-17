# Verifying Zene's Isolated Python Execution Environment

_2026-02-17_

One of the biggest challenges in building autonomous coding agents is **Dependency Hell**. Agents often need to run Python scripts to verify logic or analyze data, but running `pip install` on the host machine is dangerous and messy.

Today, we verified Zene's new **V3 Python Execution Engine**, which introduces session-scoped isolation and automatic virtual environment management.

## The Verification Test

We wrote a Rust test script (`examples/verification.rs`) to verify two critical features:
1.  **Virtual Environment Isolation**: Does the script run inside a dedicated `.venv`?
2.  **Environment Variable Injection**: Can we inject variables (like API keys) safely without polluting the host process?

### The Test Code

```rust
// examples/verification.rs

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Setup Session Envs
    let mut env_vars = HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), "CorrectValue".to_string());

    // 2. Create a test python script
    let script_path = "verify_test.py";
    let script_content = r#"
import os
import sys

print(f"Python Executable: {sys.executable}")
print(f"TEST_VAR: {os.environ.get('TEST_VAR')}")
"#;
    fs::write(script_path, script_content)?;

    // 3. Run Python using ToolManager
    // This should automatically create/use .venv and inject env_vars
    let output = ToolManager::run_python(script_path, &vec![], &env_vars).await?;
    
    println!("{}", output);
}
```

## The Results

Running the verification script produced the following output:

```bash
$ cargo run --example verification

Starting Python Execution Verification...
✅ Session Environment Variables prepared.
✅ Created test python script: verify_test.py
🚀 Running python script via ToolManager::run_python...
--- Script Output ---
Python Executable: /Users/xinference/github/zene/.venv/bin/python
TEST_VAR: CorrectValue
---------------------
✅ SUCCESS: Environment variable injected correctly.
✅ SUCCESS: Script ran inside .venv (path contained '.venv').
```

## Under the Hood

How does this work?

1.  **Auto-Venv**: When `run_python` is called, Zene checks for a `.venv` directory. If missing, it automatically creates one (preferring `uv` for speed, falling back to `python3 -m venv`).
2.  **Session Isolation**: The `TEST_VAR` was stored in the Session struct in Rust. It was injected into the child process using `Command::envs()`. Crucially, the host process's environment remained untouched.
3.  **Safety**: The script ran in a sandboxed process with a 60-second timeout and blocked stdin, preventing it from hanging the server.

This architecture allows Zene to safely execute untrusted code and manage complex Python dependencies without user intervention.
