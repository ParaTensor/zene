import subprocess
import json
import time
import os
import sys

# Ensure Zene is built
# cargo build --release

ZENE_BIN = "./target/release/zene"
if not os.path.exists(ZENE_BIN):
    ZENE_BIN = "./target/debug/zene"
    if not os.path.exists(ZENE_BIN):
        print("Error: Zene binary not found. Please run 'cargo build' first.")
        exit(1)

def send_request(proc, method, params, req_id):
    req = {
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": req_id
    }
    json_str = json.dumps(req)
    print(f"\n[Client] -> Sending: {json_str[:200]}...") # Truncate for cleaner log
    proc.stdin.write(json_str + "\n")
    proc.stdin.flush()

def read_response(proc):
    line = proc.stdout.readline()
    if not line:
        return None
    print(f"[Client] <- Received: {line[:200]}...") # Truncate
    try:
        return json.loads(line)
    except json.JSONDecodeError:
        print(f"[Client] Error decoding JSON: {line}")
        return None

def main():
    # 1. Configuration (Injected via Environment Variables)
    # NOTE: These keys are provided by the user in the prompt and should be set when running this script.
    # For this demo script, we assume the environment is already set by the caller.
    
    env = os.environ.copy()
    env["RUST_LOG"] = "info"
    
    # Verify environment
    if "ZENE_PLANNER_API_KEY" not in env:
        print("Warning: ZENE_PLANNER_API_KEY not set. Demo might fail if not using global defaults.")

    print("Starting Zene Server with Multi-Role Configuration...")
    print(f"Planner Model: {env.get('ZENE_PLANNER_MODEL', 'default')}")
    print(f"Executor Model: {env.get('ZENE_EXECUTOR_MODEL', 'default')}")
    print(f"Reflector Model: {env.get('ZENE_REFLECTOR_MODEL', 'default')}")

    proc = subprocess.Popen(
        [ZENE_BIN, "server"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=sys.stderr, # Let stderr go to console for logs
        text=True,
        env=env
    )

    try:
        session_id = "demo_multi_agent_001"
        
        # 2. Task: Analyze the project and create a summary
        # This is a read-only task that requires planning (scanning files) and execution (reading files).
        instruction = (
            "Please analyze the current Rust project structure and create a new file named `ARCHITECTURE.md` "
            "that describes the key modules (agent, engine, config) and their responsibilities. "
            "Do not just list files, explain the architecture."
        )
        
        print(f"\n--- Sending Task: {instruction} ---")
        send_request(proc, "agent.run", {
            "instruction": instruction,
            "session_id": session_id
        }, 1)
        
        # 3. Wait for completion (this might take a while due to multiple steps)
        print("\nWaiting for Agent execution (this involves Planning -> Execution loop)...")
        resp = read_response(proc)
        
        if resp and "result" in resp:
            result = resp["result"]
            print("\n=== Agent Execution Result ===")
            print(json.dumps(result, indent=2))
            
            if result.get("status") == "completed":
                print("\n✅ Task Completed Successfully!")
                # Verify file creation
                if os.path.exists("ARCHITECTURE.md"):
                    print("\n[Verification] ARCHITECTURE.md created:")
                    with open("ARCHITECTURE.md", "r") as f:
                        print(f.read())
                else:
                    print("\n[Verification] Warning: ARCHITECTURE.md not found.")
            else:
                print(f"\n❌ Task Status: {result.get('status')}")
                if "error" in result:
                    print(f"Error: {result['error']}")

    finally:
        proc.terminate()

if __name__ == "__main__":
    main()
