import subprocess
import time
import json
import os
import sys
import shutil

# Configuration
ZENE_BIN = os.path.abspath("./target/release/zene")
WORKSPACE_DIR = os.path.abspath("./temp_workspace/bug_fixing_rust")

# Sample Broken Rust Code
# Contains:
# 1. Lifetime error (borrow checker)
# 2. Type mismatch (returning i32 instead of Result)
# 3. Unused variable warning
MAIN_RS = """
fn get_longest<'a>(s1: &'a str, s2: &str) -> &'a str {
    if s1.len() > s2.len() {
        s1
    } else {
        s2 // Error: s2 does not live long enough ('a)
    }
}

fn calculate(val: i32) -> Result<i32, String> {
    if val < 0 {
        return Err("Negative value".to_string());
    }
    val * 2 // Error: expected Result, found i32
}

fn main() {
    let string1 = String::from("long string");
    let result;
    {
        let string2 = String::from("xyz");
        result = get_longest(string1.as_str(), string2.as_str());
        // string2 goes out of scope here, but result borrows it
    }
    println!("The longest string is {}", result);
    
    let calc = calculate(10);
    println!("Calc: {:?}", calc);
}
"""

CARGO_TOML = """
[package]
name = "broken_app"
version = "0.1.0"
edition = "2021"

[dependencies]
"""

def setup_workspace():
    if os.path.exists(WORKSPACE_DIR):
        print(f"Cleaning existing workspace: {WORKSPACE_DIR}")
        shutil.rmtree(WORKSPACE_DIR)
    os.makedirs(WORKSPACE_DIR)
    os.makedirs(os.path.join(WORKSPACE_DIR, "src"))
    
    # Write Cargo.toml
    with open(os.path.join(WORKSPACE_DIR, "Cargo.toml"), "w") as f:
        f.write(CARGO_TOML)

    # Write broken main.rs
    with open(os.path.join(WORKSPACE_DIR, "src/main.rs"), "w") as f:
        f.write(MAIN_RS)
    
    print(f"Created broken Rust project at: {WORKSPACE_DIR}")

def main():
    setup_workspace()

    env = os.environ.copy()
    env["RUST_LOG"] = "info"
    
    print("Starting Zene Server for Bug Fixing Task...")
    print(f"Working Directory: {WORKSPACE_DIR}")

    proc = subprocess.Popen(
        [ZENE_BIN, "server"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=sys.stderr, 
        text=True,
        env=env,
        cwd=WORKSPACE_DIR
    )

    try:
        session_id = f"demo_fix_rust_{int(time.time())}"
        print(f"Session ID: {session_id}")
        
        # Task: Fix Compilation Errors
        instruction = """
I have a Rust project in the current directory that fails to compile.
Please:
1. Run `cargo build` to see the errors.
2. Analyze the compiler output to understand the lifetime and type mismatch issues.
3. Fix the code in `src/main.rs`.
4. Run `cargo build` again to verify the fix.
5. Run `cargo run` to ensure it works correctly.
        """
        
        request = {
            "jsonrpc": "2.0",
            "method": "agent.run",
            "params": {
                "instruction": instruction.strip(),
                "session_id": session_id
            },
            "id": 1
        }

        print(f"\n--- Sending Fix Task ---\n{instruction}\n---")
        
        json_req = json.dumps(request)
        proc.stdin.write(json_req + "\n")
        proc.stdin.flush()

        print("Waiting for Agent execution...")
        
        while True:
            line = proc.stdout.readline()
            if not line:
                break
            
            try:
                resp = json.loads(line)
                if "result" in resp:
                    print("\n=== Agent Execution Result ===")
                    print(json.dumps(resp["result"], indent=2))
                    break
                elif "error" in resp:
                    print(f"Error: {resp['error']}")
                    break
            except json.JSONDecodeError:
                print(f"[Server Log] {line.strip()}")

    except KeyboardInterrupt:
        print("\nStopping...")
    finally:
        proc.terminate()

if __name__ == "__main__":
    main()
