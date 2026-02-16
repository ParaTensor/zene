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
WORKSPACE_DIR = os.path.abspath("./temp_workspace/full_react_app")

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
    while True:
        line = proc.stdout.readline()
        if not line:
            return None
        # Print raw log lines from stderr/stdout mixed stream if any
        # But here we only capture stdout for JSON-RPC
        
        try:
            # Try to parse as JSON-RPC response
            data = json.loads(line)
            if "jsonrpc" in data:
                print(f"[Client] <- Received: {line[:200]}...")
                return data
        except json.JSONDecodeError:
            # Not a JSON line (maybe logs), print it
            print(f"[Server Log] {line.strip()}")

def main():
    setup_workspace()

    env = os.environ.copy()
    env["RUST_LOG"] = "info"
    
    # Ensure npm is in PATH for the agent
    # The agent inherits the current PATH, so it should be fine.

    print("Starting Zene Server for Full React App Generation...")
    print(f"Working Directory: {WORKSPACE_DIR}")

    # Launch Zene in the workspace directory
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
        session_id = "demo_full_react_001"
        
        # Task: Full Project Initialization
        # This requires:
        # 1. Shell command execution (npm create)
        # 2. File creation (Components)
        # 3. Modification (App.tsx)
        instruction = (
            "I want to build a React application from scratch.\n"
            "1. Initialize a new Vite project in the current directory using `npm create vite@latest . -- --template react-ts`.\n"
            "2. Install dependencies with `npm install`.\n"
            "3. Create a `Counter` component in `src/components/Counter.tsx` with CSS modules.\n"
            "4. Modify `src/App.tsx` to use the Counter component.\n"
            "5. Do NOT start the server yet, just set everything up."
        )
        
        print(f"\n--- Sending Task: {instruction} ---")
        send_request(proc, "agent.run", {
            "instruction": instruction,
            "session_id": session_id
        }, 1)
        
        print("\nWaiting for Agent execution (this will take longer due to npm install)...")
        resp = read_response(proc)
        
        if resp and "result" in resp:
            result = resp["result"]
            print("\n=== Agent Execution Result ===")
            print(json.dumps(result, indent=2))
            
            if result.get("status") == "completed":
                print("\n✅ Project Setup Completed!")
                print(f"Go to: {WORKSPACE_DIR}")
                print("Run `npm run dev` to start the app.")
            else:
                print(f"\n❌ Task Status: {result.get('status')}")

    finally:
        proc.terminate()

if __name__ == "__main__":
    main()
