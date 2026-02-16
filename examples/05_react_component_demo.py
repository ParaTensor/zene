import subprocess
import json
import os
import sys
import shutil

# Ensure Zene is built
ZENE_BIN = os.path.abspath("./target/release/zene")
if not os.path.exists(ZENE_BIN):
    ZENE_BIN = os.path.abspath("./target/debug/zene")
    if not os.path.exists(ZENE_BIN):
        print("Error: Zene binary not found. Please run 'cargo build' first.")
        exit(1)

# Workspace Configuration
WORKSPACE_DIR = os.path.abspath("./temp_workspace/react_demo")

def setup_workspace():
    if os.path.exists(WORKSPACE_DIR):
        print(f"Cleaning existing workspace: {WORKSPACE_DIR}")
        shutil.rmtree(WORKSPACE_DIR)
    os.makedirs(WORKSPACE_DIR)
    print(f"Created workspace: {WORKSPACE_DIR}")

def send_request(proc, method, params, req_id):
    req = {
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": req_id
    }
    json_str = json.dumps(req)
    print(f"\n[Client] -> Sending: {json_str[:200]}...") 
    proc.stdin.write(json_str + "\n")
    proc.stdin.flush()

def read_response(proc):
    line = proc.stdout.readline()
    if not line:
        return None
    print(f"[Client] <- Received: {line[:200]}...") 
    try:
        return json.loads(line)
    except json.JSONDecodeError:
        print(f"[Client] Error decoding JSON: {line}")
        return None

def main():
    setup_workspace()

    env = os.environ.copy()
    env["RUST_LOG"] = "info"
    
    # Check if user updated the model to Minimax 2.5
    reflector_model = env.get("ZENE_REFLECTOR_MODEL", "default")
    print("Starting Zene Server for React Component Generation Task...")
    print(f"Planner Model: {env.get('ZENE_PLANNER_MODEL', 'default')}")
    print(f"Executor Model: {env.get('ZENE_EXECUTOR_MODEL', 'default')}")
    print(f"Reflector Model: {reflector_model} (Minimax)")
    print(f"Working Directory: {WORKSPACE_DIR}")

    # Launch Zene in the workspace directory
    proc = subprocess.Popen(
        [ZENE_BIN, "server"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=sys.stderr, 
        text=True,
        env=env,
        cwd=WORKSPACE_DIR # Crucial: Run in the temp directory
    )

    try:
        session_id = "demo_react_001"
        
        # Task: Create a React Component
        instruction = (
            "Create a modern, functional React component named `Counter` using TypeScript and CSS Modules. "
            "It should have increment, decrement, and reset functionality. "
            "Also create a corresponding CSS module file for styling. "
            "Finally, create a dummy `src/App.tsx` showing how to use it. "
            "Create files in `src/components` (create dirs if needed)."
        )
        
        print(f"\n--- Sending Task: {instruction} ---")
        send_request(proc, "agent.run", {
            "instruction": instruction,
            "session_id": session_id
        }, 1)
        
        print("\nWaiting for Agent execution...")
        resp = read_response(proc)
        
        if resp and "result" in resp:
            result = resp["result"]
            print("\n=== Agent Execution Result ===")
            print(json.dumps(result, indent=2))
            
            if result.get("status") == "completed":
                print("\n✅ Task Completed Successfully!")
                print(f"Check the generated files in: {WORKSPACE_DIR}/src/components/")
            else:
                print(f"\n❌ Task Status: {result.get('status')}")

    finally:
        proc.terminate()

if __name__ == "__main__":
    main()
